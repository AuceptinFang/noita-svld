use crate::units::path;
use std::fs;
use std::path::{Path, PathBuf};
use crate::backup::fs_ops::*;

/// 把目标存档直接复制到本地
/// 返回 (backup_name, digest)
pub async fn save_local() -> Result<(String, String), String> {
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
    let backup_root = PathBuf::from(path::get_data_path()?);
    if !backup_root.exists() {
        fs::create_dir_all(&backup_root).map_err(|e| e.to_string())?;
    }

    let backup_name = format!("backup_{}", digest_prefix);
    let backup_path = backup_root.join(&backup_name);

    // 如果已经存在相同 digest 的备份，直接返回
    if backup_path.exists() {
        println!("备份已存在: {}", backup_path.display());
        return Ok((backup_name, digest));
    }

    // 复制存档目录
    copy_directory(source_path, &backup_path).map_err(|e| e.to_string())?;

    println!("存档已复制到: {}", backup_path.display());
    Ok((backup_name, digest))
}

