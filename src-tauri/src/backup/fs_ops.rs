use anyhow::Result;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
use std::process::Command;
use log::error;

#[derive(Debug)]
struct DirectoryEntry {
    path: String,
    size: u64,
    modified_secs: u64,
    is_dir: bool,
}


/// 计算目录指纹（基于文件元数据，不读取文件内容）
pub fn calculate_hash(path: &Path) -> Result<String> {
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


/// 递归收集目录中所有文件和子目录的元数据
pub fn collect_directory_entries(dir: &Path, entries: &mut Vec<DirectoryEntry>) -> Result<()> {
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
pub fn calculate_directory_size(path: &Path) -> Result<i64> {
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
pub fn copy_directory_system(src: &Path, dst: &Path) -> Result<()> {
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
pub fn copy_directory_native(src: &Path, dst: &Path) -> Result<()> {
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
pub fn copy_directory(src: &Path, dst: &Path) -> Result<()> {
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
