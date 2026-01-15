use std::fs;
use std::path::Path;
use super::{fs_ops, service};
use crate::db::{Backup, Db};
use crate::backup::service::*;
use crate::backup::fs_ops::*;
use chrono::Local;
use log::{debug, error, info};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use crate::units::path;

/// 在数据库里留档
#[tauri::command]
pub async fn save_backup(name : Option<&str>) -> Result<String, String> {
    debug!("[save_back_up] {}", Local::now());
    // 先保存到本地
    let backup_name = match save_local().await{
        Ok(name) => name,
        Err(e) => {
            error!("保存时出错: {}",e);
            return Err(e);
        }
    };

    // 计算备份信息
    let backup_path = Path::new("./backups").join(&backup_name);
    let digest = match calculate_hash(&backup_path).map_err(|e| e.to_string()){
        Ok(digest) => digest,
        Err(e) => {
            error!("计算哈希出错: {}", e);
            return Err(e);
        }
    };

    // 连接数据库
    let db_path = "./db/backups.db".to_string();
    let mut conn = match Db::new(db_path).await{
        Ok(db) => db,
        Err(e) => {
            error!("建立数据库连接出错: {}",e);
            return Err(e.to_string());
        }
    };

    // 检查是否已存在相同 digest 的备份
    if let Some(existing_backup) = Db::get_backup_by_digest(&mut conn, &digest).await.map_err(|e| {
        error!("查询数据库失败: {}", e);
        e.to_string()
    })? {
        let existing_name = existing_backup.name.as_deref().unwrap_or("未命名");
        let msg = format!("该存档内容已备份过，名称为: {}", existing_name);
        info!("{}", msg);
        return Err(msg);
    }

    let size = match calculate_directory_size(&backup_path).map_err(|e| e.to_string()){
        Ok(size) => size,
        Err(e) => {
            error!("计算文件大小出错: {}",e);
            return Err(e);
        }
    };
    let save_time = OffsetDateTime::now_utc();

    let slot_name: String = name
        .filter(|n| !n.trim().is_empty())
        .map(|n| n.to_string())
        .unwrap_or_else(|| {
            let time_str = save_time
                .format(&Rfc3339)
                .unwrap_or_default();
            format!("存档_{}", time_str)
        });

    let backup = Backup {
        id: 0,
        name: Some(slot_name),
        digest,
        size,
        save_time,
        more_info: None,
    };

    match Db::store_backup(&backup, &mut conn).await {
        Ok(_) => {},
        Err(e) => {
            error!("存储数据库失败: {}",e);
            return Err(e.to_string());
        }
    }

    info!(
        "[{}] 存档保存成功: {}",Local::now(),
        backup.name.as_ref().unwrap_or(&"未命名".to_string())
    );
    Ok("存档保存成功".to_string())
}

#[tauri::command]
pub async fn get_all_backups() -> Result<Vec<Backup>, String> {
    debug!("[get_all_backups] {}",Local::now());
    let db_path = "./db/backups.db".to_string();

    let mut conn = match Db::new(db_path).await{
        Ok(db) => db,
        Err(e) => {
            error!("建立数据库连接出错: {}",e);
            return Err(e.to_string());
        }
    };

    Db::get_all_backup(&mut conn).await.map_err(|e| {
        error!("获取已有存档失败: {}", e);  // 记录错误日志
        e.to_string()  // 将错误转换为字符串后返回
    })

}

#[tauri::command]
pub async fn load_backup(backup_id: i32) -> Result<String, String> {
    debug!("[load_backup] {} ", Local::now());
    // 连接数据库查找备份
    let db_path = "./db/backups.db".to_string();

    let mut conn = Db::new(db_path).await.map_err(|e| {
        error!("建立数据库连接出错: {}", e);
        e.to_string()
    })?;

    let backup = Db::get_backup_by_id(&mut conn, backup_id)
        .await
        .map_err(|e| {
            error!("获取存档出错: {}", e);
            e.to_string()
        })?;

    let backup = match backup {
        Some(b) => b,
        None => return Err(format!("未找到ID为{}的备份", backup_id)),
    };

    // 获取当前存档路径
    let save_path = path::get_save_path().map_err(|e| {
        error!("获取路径失败: {}",e);
        e.to_string()
    })?;
    let target_path = Path::new(&save_path);

    // 构建备份文件路径
    let digest_prefix = &backup.digest[..12]; // 使用前12位
    let backup_name = format!("backup_{}", digest_prefix);
    let backup_path = Path::new("./backups").join(&backup_name);

    // 验证备份文件是否存在
    if !backup_path.exists() {
        error!("备份文件不存在: {}",backup_path.display());
        return Err(format!("备份文件不存在: {}", backup_path.display()));
    }

    // 验证备份完整性
    let current_digest = calculate_hash(&backup_path).map_err(|e| {
        error!("完整性校验失败: {}",e);
        e.to_string()
    })?;

    if current_digest != backup.digest {
        error!("哈希校验失败");
        return Err(format!("备份文件已损坏，哈希值不匹配"));
    }

    // 删除目标目录（如果存在）
    if target_path.exists() {
        fs::remove_dir_all(target_path).map_err(|e| {
            error!("删除文件失败 {}",e);
            format!("无法删除目标目录 {}: {}", target_path.display(), e)
        })?;
    }

    // 确保父目录存在
    if let Some(parent) = target_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| {
                error!("创建父目录失败 {}",e);
                format!("无法创建父目录 {}: {}", parent.display(), e)
            })?;
        }
    }

    // 复制备份到目标位置
    copy_directory(&backup_path, target_path).map_err(|e| {
        error!("加载备份失败 {}", e);
        e.to_string()
    })?;

    let success_msg = format!(
        "成功加载备份: {} -> {}",
        backup.name.as_ref().unwrap_or(&"未命名".to_string()),
        target_path.display()
    );

    debug!("{}", success_msg);
    Ok(success_msg)
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn test_description(){
        let d1 : Option<&str> = Option::Some("d1");
        let d2 : Option<&str> = Option::None;
        assert!(d1.map(|s| s.to_string()).is_some());
        assert!(d2.map(|s| s.to_string()).is_none());
    }
}