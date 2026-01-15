/// 路径相关写了四个前端可调用的函数，读、写、验证、选择文件路径。
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::{env, fs};
use chrono::Local;
use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;
use log::{info, debug, error};
use serde::{Deserialize, Serialize};


// 定义配置结构体，自动支持序列化
#[derive(Debug, Serialize, Deserialize, Default)]
struct AppConfig {
    // 使用 Option，如果是 None 代表用户还没设置
    save_path: Option<String>,
}

pub struct ConfigManager;

impl ConfigManager {
    fn get_config_file_path() -> PathBuf {
        let exe_path = env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
        let exe_dir = exe_path.parent().unwrap_or_else(|| Path::new("."));
        exe_dir.join("config.json")
    }

    /// 读取配置
    fn load() -> AppConfig {
        let path = Self::get_config_file_path();
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
                Err(e) => {
                    error!("读取配置文件失败: {}", e);
                    AppConfig::default()
                }
            }
        } else {
            AppConfig::default()
        }
    }

    /// 保存配置
    fn save(config: &AppConfig) -> Result<(), String> {
        let path = Self::get_config_file_path();
        let content = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;

        fs::write(path, content).map_err(|e| e.to_string())?;
        Ok(())
    }
}

// --- Tauri Commands 改造 ---

#[tauri::command]
pub fn save_path_to_env(path: &str) -> Result<(), String> {
    debug!("[save_config] 保存路径: {}", path);

    // 1. 读取现有配置（保留其他可能存在的字段）
    let mut config = ConfigManager::load();

    // 2. 修改路径
    config.save_path = Some(path.to_string());

    // 3. 写入文件（serde 自动处理 json 格式）
    ConfigManager::save(&config)?;

    info!("配置已更新");
    Ok(())
}

#[tauri::command]
pub fn get_save_path() -> Result<String, String> {
    let config = ConfigManager::load();

    // 如果配置文件里有，直接返回
    if let Some(path) = config.save_path {
        debug!("[get_save_path] 从配置加载: {}", path);
        return Ok(path);
    }

    // 如果没有，生成默认路径
    let default_path = if let Some(home) = dirs::home_dir() {
        // 手动拼接比较稳妥
        home.join("AppData").join("LocalLow").join("Nolla_Games_Noita").join("save00")
    } else {
        return Err("无法获取系统用户目录".to_string());
    };

    let path_str = default_path.to_string_lossy().to_string();

    // 保存默认值到配置文件
    save_path_to_env(&path_str)?;

    info!("[get_save_path] 使用默认路径: {}", path_str);
    Ok(path_str)
}

#[tauri::command]
pub async fn select_save_path(app: AppHandle) -> Option<String> {
    debug!("[select_save_path] {}", Local::now());
    // 使用同步方式获取文件夹路径
    if let Some(path) = app
        .dialog()
        .file()
        .set_title("选择存档目录")
        .blocking_pick_folder()
    {
        Some(path.to_string())
    } else {
        None
    }
}


#[tauri::command]
pub async fn verify_validation() -> Result<(), String> {
    debug!("[verify_validation] {}", Local::now());
    let current_path = get_save_path().map_err(|e| e.to_string())?;
    let path = Path::new(&current_path);

    // 检查路径是否存在
    if !path.exists() {
        let e = format!("路径不存在: {}", current_path);
        error!("{}",e);
        return Err(e);
    }

    // 检查是否为目录
    if !path.is_dir() {
        let e = format!("路径不是目录: {}", current_path);
        error!("{}",e);
        return Err(e);
    }

    // 读取目录内容
    let entries =
        fs::read_dir(path).map_err(|e| {
            let msg = format!("无法读取目录 {}: {}", current_path, e);
            error!("{}",msg);
            msg
        })?;

    // 收集所有子目录名称
    let mut subdirs = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| format!("读取目录项失败: {}", e))?;
        let path = entry.path();
        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                subdirs.push(name.to_string());
            }
        }
    }

    // 检查是否包含必需的三个存档目录
    let required_dirs = ["persistent", "stats", "world"];
    for required_dir in &required_dirs {
        if !subdirs.contains(&required_dir.to_string()) {
            error!("无法找到存档文件");
            return Err("无法找到存档文件".parse().unwrap());
        }
    }

    let success_msg = "路径验证成功".to_string();
    info!("时间：{},{}",Local::now() ,success_msg);
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::units::path::verify_validation;

    #[tokio::test]
    async fn test_path() {
        let result = verify_validation().await;
        assert!(result.is_ok(), "验证应该成功: {:?}", result);
    }
}
