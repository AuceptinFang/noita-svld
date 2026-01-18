use log::info;
use tauri_plugin_updater::UpdaterExt;
use tauri_plugin_process::*;
#[tauri::command]
async fn check_update(app: tauri::AppHandle) -> Result<String, String> {
    let updater = app.updater().map_err(|e| e.to_string())?;

    // 检查是否有新版本
    // 根据 tauri.conf.json 里的 endpoints 去请求更新源
    let update_result = updater.check().await.map_err(|e| e.to_string())?;

    if let Some(update) = update_result {
        update
            .download_and_install(
                |chunk_length, content_length| {
                    if let Some(total) = content_length {
                        info!("下载进度: {}/{}", chunk_length, total);
                    }
                },
                || {
                    info!("下载完成，正在安装...");
                },
            )
            .await
            .map_err(|e| e.to_string())?;

        info!("更新成功，应用即将重启");
        app.restart();

        Ok("Update installed successfully, restarting...".to_string())
    } else {
        Ok("Already up to date".to_string())
    }
}