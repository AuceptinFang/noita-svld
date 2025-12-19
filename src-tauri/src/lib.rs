// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod db;
pub mod units;
use anyhow::Result;
use units::backup::*;
use units::path::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> Result<()> {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            get_save_path,
            save_path_to_env,
            select_save_path,
            verify_validation,
            save_backup,
            get_all_backups,
            load_backup,
            write_log
        ])
        .run(tauri::generate_context!())?;

    Ok(())
}
