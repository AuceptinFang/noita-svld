use anyhow::{Context, Result};
use jwalk::WalkDir;
use rayon::prelude::*;
use sha2::{Digest, Sha256};
use std::cmp::Ordering;
use std::fs;
use std::path::{Path, PathBuf};

// 定义一个中间结构体用于存储文件元数据，以便在内存中排序
struct FileMeta {
    rel_path: String,
    len: u64,
    modified: u64,
    is_dir: bool,
}

/// 计算目录指纹（基于文件元数据，不读取文件内容）
/// 优化：使用 jwalk 并行扫描 + rayon 并行排序 + 纯内存计算 Hash
pub fn calculate_hash(path: &Path) -> Result<String> {
    let mut hasher = Sha256::new();
    let root = path;

    if path.is_file() {
        // 单个文件
        let metadata = fs::metadata(path)?;
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        hasher.update(file_name.as_bytes());
        hasher.update(&metadata.len().to_le_bytes());

        if let Ok(modified) = metadata.modified() {
            if let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH) {
                hasher.update(&duration.as_secs().to_le_bytes());
            }
        }
    } else {
        // 1. 并行扫描目录 (IO 密集型优化)
        // jwalk 会利用多线程预取文件元数据
        let mut entries: Vec<FileMeta> = WalkDir::new(path)
            .skip_hidden(false) // 不跳过隐藏文件
            .follow_links(false)
            .into_iter()
            .filter_map(|entry| entry.ok()) // 忽略无法访问的文件
            .filter_map(|entry| {
                let path = entry.path();
                let metadata = entry.metadata().ok()?;

                // 计算相对路径，使用 '/' 作为分隔符
                let rel_path = path
                    .strip_prefix(root)
                    .ok()?
                    .to_string_lossy()
                    .replace('\\', "/");

                // 跳过根目录
                if rel_path.is_empty() {
                    return None;
                }

                let modified = metadata.modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(0);

                Some(FileMeta {
                    rel_path,
                    len: metadata.len(),
                    modified,
                    is_dir: metadata.is_dir(),
                })
            })
            .collect();

        // 2. 并行排序，否则多线程扫描的随机顺序会导致 Hash 每次都不一样
        entries.par_sort_unstable_by(|a, b| a.rel_path.cmp(&b.rel_path));

        // 3. 顺序计算 Hash
        for entry in entries {
            hasher.update(entry.rel_path.as_bytes());
            hasher.update(&entry.len.to_le_bytes());
            hasher.update(&entry.modified.to_le_bytes());
            // is_dir 转 u8
            hasher.update(&[entry.is_dir as u8]);
        }
    }

    let result = hasher.finalize();
    Ok(hex::encode(result))
}

/// 计算目录大小（字节）
/// 使用 jwalk 并行遍历
pub fn calculate_directory_size(path: &Path) -> Result<i64> {
    if path.is_file() {
        return Ok(fs::metadata(path)?.len() as i64);
    }

    // jwalk::WalkDir 默认启用并行（parallelism 自动设置为 CPU 核心数）
    let total_size: u64 = WalkDir::new(path)
        .skip_hidden(false)
        .into_iter()
        .filter_map(|entry| entry.ok()) // 忽略无权限错误
        .filter_map(|entry| entry.metadata().ok())
        .filter(|m| m.is_file()) // 只累加文件大小，忽略目录本身的大小占用
        .map(|m| m.len())
        .sum();

    // 强转为 i64 以匹配原始签名
    Ok(total_size as i64)
}

/// 复制目录到目标位置
/// jwalk 负责发现文件，rayon 负责并行复制。
pub fn copy_directory(src: &Path, dst: &Path) -> Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst).with_context(|| format!("无法创建目标根目录: {:?}", dst))?;
    }

    // 1. 阶段一：扫描 (Scanning)
    // 快速收集所有源文件路径，计算出目标路径。
    // 使用 Vec 收集是为了避免在复制文件的同时持有目录迭代器的锁，
    // 同时也为了先创建所有目录，再并行复制文件。
    let entries: Vec<(PathBuf, PathBuf, bool)> = WalkDir::new(src)
        .skip_hidden(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter_map(|entry| {
            let src_path = entry.path();
            // 跳过根目录本身
            if src_path == src {
                return None;
            }

            let relative = src_path.strip_prefix(src).ok()?;
            let dst_path = dst.join(relative);
            let is_dir = entry.file_type().is_dir();

            Some((src_path.to_path_buf(), dst_path, is_dir))
        })
        .collect();

    // 2. 阶段二：创建目录结构 (Structure Creation)
    for (_, dst_path, is_dir) in entries.iter() {
        if *is_dir {
            if let Err(e) = fs::create_dir_all(dst_path) {
                // 忽略 "目录已存在" 的错误
                if e.kind() != std::io::ErrorKind::AlreadyExists {
                    return Err(anyhow::anyhow!("创建目录失败 {:?}: {}", dst_path, e));
                }
            }
        }
    }

    // 3. 阶段三：并行复制文件 (Parallel Copying)
    // Rayon 会自动利用线程池进行复制。对于 SSD，这能极大提高吞吐量。
    // 遇到任何一个错误直接返回（Fail Fast），或者你可以改为收集错误。
    let errors: Vec<(PathBuf, std::io::Error)> = entries.into_par_iter()
        .filter(|(_, _, is_dir)| !*is_dir)
        .filter_map(|(src_path, dst_path, _)| -> Option<(PathBuf, std::io::Error)> {
            // 尝试复制，如果成功返回 None，如果失败返回 Some(错误信息)
            match fs::copy(&src_path, &dst_path) {
                Ok(_) => None,
                Err(e) => Some((src_path, e)),
            }
        })
        .collect(); // 这里会等待所有线程跑完，并把所有错误收集到一个 Vec 中

    // 4. 阶段四：错误汇报
    // 如果有错误，我们需要决定是打印日志还是返回 Err
    if !errors.is_empty() {
        // 打印前 5 个错误示例 (避免日志刷屏)
        for (path, err) in errors.iter().take(5) {
            eprintln!("警告: 复制失败 {:?} -> {}", path, err);
        }

        // 如果你希望“只要有文件失败就算整个任务失败”，则返回 Err
        return Err(anyhow::anyhow!(
            "备份完成，但有 {} 个文件复制失败 (详情请查看日志)",
            errors.len()
        ));

        // 如果你希望“容忍部分失败”，这里可以直接 return Ok(());
    }

    Ok(())
}