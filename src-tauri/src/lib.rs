// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod db;
pub mod units;
pub mod backup;
use anyhow::Result;
use backup::commands::*;
use units::path::*;
use units::dashboard::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> Result<()> {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stdout),
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Webview),
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::LogDir {
                        file_name: Some("app.log".to_string())   // C:\Users\username\AppData\Roaming\myapp\logs\app.log
                    }),
                ])
                .level(log::LevelFilter::Debug)
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            get_save_path,
            get_data_path,
            save_path_to_env,
            save_data_path,
            select_save_path,
            verify_validation,
            save_backup,
            get_all_backups,
            load_backup,
            get_dashboard_stats,
            delete_backup,
            select_data_path,
        ])
        .run(tauri::generate_context!())?;

    Ok(())
}
