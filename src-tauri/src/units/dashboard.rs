use std::fs;
use crate::db;
use serde::Serialize;
use walkdir::WalkDir;
use std::process::Command;
use log::{debug, error};
use crate::backup::commands::get_all_backups;
use crate::db::Db;
use crate::units::path;

#[derive(Serialize, Debug)]
pub struct DashboardStats {
    backup_count: usize,
    total_size: i64,    // 单位字节
    is_ready: bool,     // 后端健康
}

#[tauri::command]
pub async fn get_dashboard_stats() -> Result<DashboardStats, String> {
    let mut total_size : i64 = 0;
    let is_ready : bool = true;

    let backup_root = path::get_data_path()?;

    let db_path = "./db/backups.db".to_string();
    let mut conn = match Db::new(db_path).await{
        Ok(db) => db,
        Err(e) => {
            error!("建立数据库连接出错: {}",e);
            return Err(e.to_string());
        }
    };

    let backups = db::Db::get_all_backup(&mut conn).await.map_err(|e| {
        error!("查询存档错误");
        "查询数据库失败".to_string()
    })?;
    // 计算文件夹数量
    let count = backups.len();

    // 计算总大小
    for backup in backups {
        total_size += backup.size;
    }

    debug!("存档数:{} ，总大小：{}", total_size, count);

    Ok(
        DashboardStats {
            backup_count : count,
            total_size,
            is_ready,
        }
    )
}

#[cfg(test)]
mod tests{
    use super::*;
    #[tokio::test]
    async fn test_dashboard(){
        println!("{:?}", get_dashboard_stats().await.unwrap());
    }
}