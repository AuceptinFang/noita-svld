use std::path::PathBuf;
use anyhow::Result;
use log::info;

/// 获取数据库文件的标准路径
/// Windows: C:\Users\{username}\AppData\Roaming\{app_name}\data\backups.db
/// 
/// 此路径固定，用户不可配置
pub fn get_db_path() -> Result<String> {
    let app_name = "noita-svld"; // 应用名称
    
    // 获取 Windows AppData\Roaming 目录
    let app_data = std::env::var("APPDATA")
        .map_err(|_| anyhow::anyhow!("无法获取 APPDATA 环境变量"))?;
    
    // 构建完整路径: AppData\Roaming\noita-svld\data\backups.db
    let db_dir = PathBuf::from(app_data)
        .join(app_name)
        .join("data");
    
    // 确保目录存在
    std::fs::create_dir_all(&db_dir)?;
    info!("数据库目录已创建/确认: {}", db_dir.display());
    
    let db_path = db_dir.join("backups.db");
    let db_path_str = db_path.to_string_lossy().to_string();
    
    info!("数据库文件路径: {}", db_path_str);
    
    Ok(db_path_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_db_path() {
        let path = get_db_path().unwrap();
        println!("数据库路径: {}", path);
        assert!(path.contains("AppData"));
        assert!(path.contains("noita-svld"));
        assert!(path.ends_with("backups.db"));
    }
}
