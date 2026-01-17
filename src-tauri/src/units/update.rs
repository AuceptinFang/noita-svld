use tauri_plugin_updater::UpdaterExt;

#[tauri::command]
async fn check_update(app: tauri::AppHandle) -> Result<String, String> {
    // 1. 获取更新实例
    let updater = app.updater().map_err(|e| e.to_string())?;

    // 2. 检查是否有新版本
    // 根据 tauri.conf.json 里的 endpoints 去请求更新源
    let update_result = updater.check().await.map_err(|e| e.to_string())?;

    if let Some(update) = update_result {
        // 3. 发现新版本，开始下载并安装
        // 注意：Tauri 会自动校验签名 (Signature)
        update
            .download_and_install(
                |chunk_length, content_length| {
                    // content_length 是可选的，有些服务器不返回总大小
                    if let Some(total) = content_length {
                        println!("下载进度: {}/{}", chunk_length, total);
                    }
                },
                || {
                    println!("下载完成，正在安装...");
                },
            )
            .await
            .map_err(|e| e.to_string())?;

        // 4. 安装成功后重启应用
        println!("更新成功，应用即将重启");
        app.restart();

        Ok("Update installed successfully, restarting...".to_string())
    } else {
        // 5. 已经是最新版本
        Ok("Already up to date".to_string())
    }
}