use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs};
use log::{debug, error, info};
use crate::db::Db;
use crate::units::db_path;

#[tauri::command]
pub async fn open_log() -> Result<(), String> {
    // 获取 LOCALAPPDATA 环境变量 (通常是 C:\Users\用户名\AppData\Local)
    let local_app_data = env::var("LOCALAPPDATA")
        .map_err(|_| "无法获取 LOCALAPPDATA 环境变量".to_string())?;

    // 拼接日志完整路径
    let log_path = Path::new(&local_app_data)
        .join("com.auceptin.noita-svld")
        .join("logs")
        .join("app.log");

    if !log_path.exists() {
        return Err(format!("日志文件不存在: {:?}", log_path));
    }

    // 在 Windows 资源管理器中打开并选中该文件
    // /select, 后面紧跟路径可以实现“在文件夹中显示”并高亮该文件
    Command::new("explorer")
        .arg("/select,")
        .arg(log_path)
        .spawn()
        .map_err(|e| format!("无法打开资源管理器: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn open_backup(id: i32) -> Result<(), String> {
    debug!("[open_backup] id = {}", id);

    let db_path = db_path::get_db_path().map_err(|e| {
        error!("获取数据库路径失败: {}", e);
        e.to_string()
    })?;

    let mut conn = match Db::new(db_path).await {
        Ok(db) => db,
        Err(e) => {
            error!("建立数据库连接出错: {}", e);
            return Err(e.to_string());
        }
    };

    let backup = Db::get_backup_by_id(&mut conn, id)
        .await
        .map_err(|e| {
            error!("获取存档出错: {}", e);
            e.to_string()
        })?;

    let backup = match backup {
        Some(b) => b,
        None => return Err(format!("未找到 ID 为 {} 的备份", id)),
    };

    // 获取备份文件所在的父目录，或者是备份路径本身
    let path = Path::new(&backup.path);

    // 如果 backup.path 指向的是文件，我们打开它的父目录并选中它
    // 如果是目录，直接打开该目录
    if path.is_file() {
        Command::new("explorer")
            .arg("/select,")
            .arg(path)
            .spawn()
            .map_err(|e| format!("打开文件夹失败: {}", e))?;
    } else if path.is_dir() {
        Command::new("explorer")
            .arg(path)
            .spawn()
            .map_err(|e| format!("打开文件夹失败: {}", e))?;
    } else {
        return Err("备份路径不存在或无效".to_string());
    }

    Ok(())
}