use crate::db::{Backup, Db};
use crate::units::path;
use anyhow::Result;
use sha2::{Digest, Sha256};
use std::fs;

use std::path::Path;
use std::process::Command;
use log::{debug, error, info};
use chrono::Local;
use serde::de::Unexpected::Option;
use time::OffsetDateTime;


/// 计算目录指纹（基于文件元数据，不读取文件内容）
fn calculate_hash(path: &Path) -> Result<String> {
    let mut hasher = Sha256::new();

    if path.is_file() {
        // 对单个文件，使用元数据 + 文件名
        let metadata = fs::metadata(path)?;
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        hasher.update(file_name.as_bytes());
        hasher.update(&metadata.len().to_le_bytes());

        // 添加修改时间（如果可用）
        if let Ok(modified) = metadata.modified() {
            if let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH) {
                hasher.update(&duration.as_secs().to_le_bytes());
            }
        }
    } else if path.is_dir() {
        // 收集所有文件和子目录的元数据
        let mut entries = Vec::new();
        collect_directory_entries(path, &mut entries)?;

        // 按路径排序确保一致性
        entries.sort_by(|a, b| a.path.cmp(&b.path));

        // 哈希所有条目的元数据
        for entry in entries {
            hasher.update(entry.path.as_bytes());
            hasher.update(&entry.size.to_le_bytes());
            hasher.update(&entry.modified_secs.to_le_bytes());
            hasher.update(&[entry.is_dir as u8]);
        }
    }

    let result = hasher.finalize();
    Ok(result
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>())
}

/// 文件/目录条目信息
#[derive(Debug)]
struct DirectoryEntry {
    path: String,
    size: u64,
    modified_secs: u64,
    is_dir: bool,
}

/// 递归收集目录中所有文件和子目录的元数据
fn collect_directory_entries(dir: &Path, entries: &mut Vec<DirectoryEntry>) -> Result<()> {
    let read_dir = fs::read_dir(dir)?;

    for entry in read_dir {
        let entry = entry?;
        let path = entry.path();
        let metadata = entry.metadata()?;

        let relative_path = path
            .strip_prefix(dir)
            .unwrap_or(&path)
            .to_string_lossy()
            .to_string();

        let modified_secs = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);

        entries.push(DirectoryEntry {
            path: relative_path,
            size: metadata.len(),
            modified_secs,
            is_dir: metadata.is_dir(),
        });

        // 递归处理子目录
        if metadata.is_dir() {
            collect_directory_entries(&path, entries)?;
        }
    }

    Ok(())
}

/// 使用系统调用计算目录大小（字节）
fn calculate_directory_size(path: &Path) -> Result<i64> {
    #[cfg(target_os = "windows")]
    {
        // 使用PowerShell获取目录大小
        let output = Command::new("powershell")
            .args([
                "-Command",
                &format!(
                    "(Get-ChildItem -Path '{}' -Recurse -Force | Measure-Object -Property Length -Sum).Sum",
                    path.display()
                )
            ])
            .output()?;

        if output.status.success() {
            let size_str = String::from_utf8_lossy(&output.stdout).to_string();
            if let Ok(size) = size_str.parse::<i64>() {
                return Ok(size);
            }
        }
    }

    // 回退到原生方法
    let mut total_size = 0i64;

    if path.is_file() {
        return Ok(fs::metadata(path)?.len() as i64);
    }

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            total_size += calculate_directory_size(&path)?;
        } else {
            total_size += fs::metadata(&path)?.len() as i64;
        }
    }

    Ok(total_size)
}

/// 使用系统调用复制目录到目标位置
fn copy_directory_system(src: &Path, dst: &Path) -> Result<()> {
    // 确保目标目录的父目录存在
    if let Some(parent) = dst.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    #[cfg(target_os = "windows")]
    {
        // 使用robocopy进行高性能复制
        let output = Command::new("robocopy")
            .args([
                src.to_str().unwrap(),
                dst.to_str().unwrap(),
                "/E",    // 复制子目录，包括空目录
                "/MT:8", // 多线程复制，使用8个线程
                "/R:3",  // 重试次数
                "/W:1",  // 重试等待时间（秒）
                "/NFL",  // 不记录文件名
                "/NDL",  // 不记录目录名
                "/NP",   // 不显示进度
            ])
            .output()?;

        // robocopy的退出码0-7都表示成功
        let exit_code = output.status.code().unwrap_or(-1);
        if exit_code >= 0 && exit_code <= 7 {
            return Ok(());
        } else {
            // 如果robocopy失败，尝试使用xcopy
            let output = Command::new("xcopy")
                .args([
                    src.to_str().unwrap(),
                    dst.to_str().unwrap(),
                    "/E", // 复制目录和子目录，包括空目录
                    "/I", // 如果目标不存在且复制多个文件，假定目标是目录
                    "/H", // 复制隐藏文件和系统文件
                    "/Y", // 覆盖现有文件而不提示
                ])
                .output()?;

            if !output.status.success() {
                return Err(anyhow::anyhow!(
                    "系统复制命令失败: {}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
        }
    }

    Ok(())
}

/// 回退的原生复制方法
fn copy_directory_native(src: &Path, dst: &Path) -> Result<()> {
    // 确保目录存在
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    // 遍历目录
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            copy_directory_native(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

/// 优先使用系统调用，失败时回退到原生方法
fn copy_directory(src: &Path, dst: &Path) -> Result<()> {
    // 首先尝试系统调用
    match copy_directory_system(src, dst) {
        Ok(()) => Ok(()),
        Err(_) => {
            // 系统调用失败时回退到原生方法
            error!("系统调用复制失败，回退到原生方法");
            copy_directory_native(src, dst)
        }
    }
}

/// 把目标存档直接复制到本地，保证开销最小

pub async fn save_local() -> Result<String, String> {
    let save_path = path::get_save_path().map_err(|e| e.to_string())?;
    let source_path = Path::new(&save_path);

    // 验证源路径
    if !source_path.exists() {
        return Err(format!("存档路径不存在: {}", save_path));
    }

    // 先计算 digest 用作文件名
    let digest = calculate_hash(source_path).map_err(|e| e.to_string())?;
    let digest_prefix = &digest[..12]; // 使用前12位作为文件名

    // 创建备份目录
    let backup_root = Path::new("./backup");
    if !backup_root.exists() {
        fs::create_dir_all(backup_root).map_err(|e| e.to_string())?;
    }

    let backup_name = format!("backup_{}", digest_prefix);
    let backup_path = backup_root.join(&backup_name);

    // 如果已经存在相同 digest 的备份，直接返回
    if backup_path.exists() {
        println!("备份已存在: {}", backup_path.display());
        return Ok(backup_name);
    }

    // 复制存档目录
    copy_directory(source_path, &backup_path).map_err(|e| e.to_string())?;

    println!("存档已复制到: {}", backup_path.display());
    Ok(backup_name)
}

/// 在数据库里留档
#[tauri::command]
pub async fn save_backup(slot_id : i8) -> Result<String, String> {
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
    let backup_path = Path::new("./backup").join(&backup_name);
    let digest = match calculate_hash(&backup_path).map_err(|e| e.to_string()){
        Ok(digest) => digest,
        Err(e) => {
            error!("计算哈希出错: {}", e);
            return Err(e);
        }
    };

    let size = match calculate_directory_size(&backup_path).map_err(|e| e.to_string()){
        Ok(size) => size,
        Err(e) => {
            error!("计算文件大小出错: {}",e);
            return Err(e);
        }
    };
    let save_time = OffsetDateTime::now_utc();

    // 创建Backup结构体
    let backup = Backup {
        id: 0, // 数据库自增，这里设为0
        name: Some(format!(
            "存档_{}",
            save_time
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_default()
        )),
        digest,
        size,
        save_time,
        more_info: None,
        slot_id: Some(slot_id), // 暂时设为None
    };

    // 连接数据库并保存
    let db_path = "./db/backups.db".to_string();
    let mut conn = match Db::new(db_path).await{
        Ok(db) => db,
        Err(e) => {
            error!("建立数据库连接出错: {}",e);
            return Err(e.to_string());
        }
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
    let backup_path = Path::new("./backup").join(&backup_name);

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


