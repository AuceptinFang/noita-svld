/// 路径相关写了四个前端可调用的函数，读、写、验证、选择文件路径。
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::{env, fs};
use chrono::Local;
use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;
use log::{info, debug, error, warn, trace};

pub fn find_env_file() -> Result<PathBuf, String> {
    let possible_paths = [".env", "../.env", "../../.env"];
    possible_paths
        .iter()
        .find(|&&p| Path::new(p).exists())
        .map(|&p| PathBuf::from(p))
        .ok_or_else(|| {
            error!("没有找到.env文件");
            "没有找到 .env 文件".to_string()
        })
}

///读env文件保存的path字段
pub fn load_save_path_from_env() -> Option<String> {
    //莫名其妙的路径寻找，但没关系可以多找找
    let possible_paths = [".env", "../.env", "../../.env"];
    
    for env_file_path in &possible_paths {
    
        if let Ok(content) = fs::read_to_string(env_file_path) {
            debug!("[load_save_path_from_env] 成功读取.env文件");
            
            // 逐行解析，查找SAVE_PATH
            for line in content.lines() {
                let line = line.trim();
                if line.starts_with("SAVE_PATH=") {
                    let path = line.strip_prefix("SAVE_PATH=").unwrap_or("").trim();
                    if !path.is_empty() {
                        debug!("[load_save_path_from_env] 找到SAVE_PATH: {}", path);
                        return Some(path.to_string());
                    }
                }
            }
            error!("[load_save_path_from_env] .env文件中未找到SAVE_PATH");
        } else {
            error!("[load_save_path_from_env] 无法读取.env文件: {}", env_file_path);
        }
    }
    
    None
}

/// 输入字符串（切片），写入.env文件保存
#[tauri::command]
pub fn save_path_to_env(path: &str) -> Result<(), String> {
    debug!("[save_path_to_env] {} 存入 {}",Local::now(), path);

    let env_file = find_env_file().map_err(|e| e.to_string())?;

    // 读取现有的.env内容
    let content = if env_file.exists() {
        fs::read_to_string(&env_file).unwrap_or_default()
    } else {
        String::new()
    };

    let lines: Vec<String> = content
        .lines()
        .filter(|line| !line.starts_with("SAVE_PATH="))
        .map(|s| s.to_string())
        .collect();

    // 新的内容，首先加入原有内容，再添加SAVE_PATH
    let mut new_content = lines.join("\n");
    if !new_content.is_empty() {
        new_content.push('\n'); // 保证每行之间有换行符
    }
    new_content.push_str(&format!("SAVE_PATH={}", path));

    // 写入新内容
    fs::write(&env_file, new_content).map_err(|e| {
        format!("写入.env文件失败: {}", e)
    })?;
    
    info!("[save_path_to_env] 成功保存路径到.env: {}", path);
    Ok(())
}

/// 如果env文件里已经写入，则返回；如果没有，则默认，返回路径
#[tauri::command]
pub fn get_save_path() -> Result<String, String> {
    debug!("[get_save_path] {}",Local::now());
    let path = load_save_path_from_env();
    let dir = if let Some(path) = path {
        // 如果从env文件读取到路径，直接使用，不要重新保存
        PathBuf::from(&path)
    } else {
        // 只有在没有env设置时，才使用默认路径并保存到env
        let username =
            env::var("USERNAME").map_err(|_| "自动获取路径失败，请手动输入存档路径".to_string())?;
        let mut dir = PathBuf::from("C:/Users/");
        dir.push(username);
        dir.push("AppData/LocalLow/Nolla_Games_Noita/save00");

        // 只在使用默认路径时才保存到env文件
        save_path_to_env(dir.to_str().unwrap()).map_err(|e| e.to_string())?;
        dir
    };
    info!("[get_save_path:]{:?}", dir);
    Ok(dir.to_string_lossy().to_string())
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
    use super::*;
    use crate::units::path::{get_save_path, save_path_to_env, verify_validation};
    /*#[test]
    fn test_get_path() {
        save_path_to_env("./src");
        let result = get_save_path();
        assert_eq!(result.unwrap(), "./src");
    }*/

    #[tokio::test]
    async fn test_path() {
        let result = verify_validation().await;
        assert!(result.is_ok(), "验证应该成功: {:?}", result);
    }
}
