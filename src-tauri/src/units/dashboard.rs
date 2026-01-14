use std::fs;
use serde::Serialize;
use walkdir::WalkDir;
use std::process::Command;
use log::debug;

#[derive(Serialize)]
pub struct DashboardStats {
    backup_count: usize,
    total_size: u64,    // 字节单位
    is_ready: bool,     // 后端健康
}

#[tauri::command]
pub fn get_dashboard_stats() -> Result<DashboardStats, String> {
    let mut total_size : u64 = 0;
    let is_ready : bool = true;

    let backup_root = "./backups/".to_string();
    // 计算文件夹数量
    let count = fs::read_dir(&backup_root)
        .map_err(|e| e.to_string())?
        .count();

    // 递归计算总大小 
    total_size = WalkDir::new(&backup_root)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| entry.metadata().ok())
        .filter(|metadata| metadata.is_file())
        .map(|m| m.len())
        .sum();
    debug!("存档数:{} ，总大小：{}", total_size, count);

    Ok(
        DashboardStats {
            backup_count : count,
            total_size,
            is_ready,
        }
    )
}