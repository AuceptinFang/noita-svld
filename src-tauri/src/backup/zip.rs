
/*use std::fs::{self, File};
use std::io::{Write, Read};
use std::path::{Path, PathBuf};
use std::time::Instant;
use rand::{Rng, thread_rng};
use zip::write::FileOptions;
use rayon::prelude::*;
use jwalk::WalkDir;

// ==========================================
// 这里放你那两个要对比的函数实现 (模拟引入)
// ==========================================

// 1. 旧方案：Bzip2
fn compress_bzip2(src: &Path, dst: &Path) -> anyhow::Result<()> {
    let file = File::create(dst)?;
    let mut zip = zip::ZipWriter::new(file);
    // 模拟你的旧代码逻辑
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Bzip2);

    for entry in walkdir::WalkDir::new(src) {
        let entry = entry?;
        if entry.path().is_file() {
            let name = entry.path().strip_prefix(src)?.to_str().unwrap();
            zip.start_file(name, options)?;
            let mut f = File::open(entry.path())?;
            let mut buffer = Vec::new();
            f.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
        }
    }
    zip.finish()?;
    Ok(())
}

// 2. 推荐方案：Deflate (或者 Zstd 如果你引入了的话)
fn compress_deflate(src: &Path, dst: &Path) -> anyhow::Result<()> {
    let file = File::create(dst)?;
    let mut zip = zip::ZipWriter::new(file);
    // 仅仅改了这个枚举值
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflate);

    for entry in walkdir::WalkDir::new(src) {
        let entry = entry?;
        if entry.path().is_file() {
            let name = entry.path().strip_prefix(src)?.to_str().unwrap();
            zip.start_file(name, options)?;
            let mut f = File::open(entry.path())?;
            let mut buffer = Vec::new();
            f.read_to_end(&mut buffer)?; // 注意：这里还可以优化为 std::io::copy
            zip.write_all(&buffer)?;
        }
    }
    zip.finish()?;
    Ok(())
}

// ==========================================
// 这才是你要的：正规测试模块
// ==========================================
#[cfg(test)]
mod tests {
    use super::*;

    // 辅助函数：生成测试数据 (在测试开始前运行)
    fn setup_test_data(root: &Path) {
        if root.exists() { fs::remove_dir_all(root).unwrap(); }
        fs::create_dir_all(root).unwrap();

        // 并行生成 2000 个小文件 + 5 个大文件
        // 这里用 Rayon 加速生成，避免生成数据本身太慢
        (0..2000).into_par_iter().for_each(|i| {
            let p = root.join(format!("file_{}.bin", i));
            let mut rng = thread_rng();
            let data: Vec<u8> = (0..2048).map(|_| rng.gen()).collect(); // 2KB
            let mut f = File::create(p).unwrap();
            f.write_all(&data).unwrap();
        });

        // 生成大文件
        for i in 0..5 {
            let p = root.join(format!("large_{}.bin", i));
            let data = vec![0u8; 5 * 1024 * 1024]; // 5MB
            File::create(p).unwrap().write_all(&data).unwrap();
        }
    }

    // 清理函数
    fn teardown(root: &Path) {
        if root.exists() { fs::remove_dir_all(root).unwrap(); }
    }

    #[test]
    fn benchmark_compression_performance() {
        let test_root = Path::new("target/test_data_compress");
        let output_dir = Path::new("target/test_output");
        fs::create_dir_all(output_dir).unwrap();

        println!("\n[Setup] 正在生成测试数据...");
        setup_test_data(test_root);

        // --- 测试 Bzip2 ---
        let bzip_path = output_dir.join("result_bzip2.zip");
        let start = Instant::now();
        compress_bzip2(test_root, &bzip_path).expect("Bzip2 compression failed");
        let duration_bzip = start.elapsed();

        // --- 测试 Deflate ---
        let deflate_path = output_dir.join("result_deflate.zip");
        let start = Instant::now();
        compress_deflate(test_root, &deflate_path).expect("Deflate compression failed");
        let duration_deflate = start.elapsed();

        // --- 结果输出 ---
        let size_bzip = fs::metadata(&bzip_path).unwrap().len();
        let size_deflate = fs::metadata(&deflate_path).unwrap().len();

        println!("\n================ 性能对比结果 ================");
        println!("| 方案\t\t| 耗时\t\t| 文件大小\t|");
        println!("|--------------|---------------|---------------|");
        println!("| Bzip2 (旧)\t| {:?}\t| {:.2} MB\t|", duration_bzip, size_bzip as f64 / 1024.0 / 1024.0);
        println!("| Deflate (新)\t| {:?}\t| {:.2} MB\t|", duration_deflate, size_deflate as f64 / 1024.0 / 1024.0);
        println!("==============================================\n");

        assert!(duration_deflate < duration_bzip, "Deflate 应该比 Bzip2 快！");

        // 清理
        teardown(test_root);
        teardown(output_dir);
    }
}