use log::{debug, error, info};
use chrono::Local;

#[tauri::command]
async fn write_log(msg: String) {
    info!("Yew: {}", msg);
}