mod api;
mod config;
mod cover;
mod download;
mod models;
mod refresh;
mod utils;

use models::SongDetail;
use serde_json::Value;
use std::time::Instant;
use tauri_plugin_shell::ShellExt;

// 检查更新
#[tauri::command]
async fn load_latest_version() -> Result<String, String> {
    api::fetch_latest_version().await.map_err(|e| e.to_string())
}

// 拉取官网歌曲列表
#[tauri::command]
async fn load_remote_music() -> Result<Vec<Value>, String> {
    api::fetch_all_songs().await.map_err(|e| e.to_string())
}

// 加载本地歌曲列表
#[tauri::command]
async fn load_local_music(app: tauri::AppHandle) -> Result<Vec<SongDetail>, String> {
    config::load_music_data(&app)
        .await
        .map_err(|e| e.to_string())
}

// 更新歌曲列表并保存完整信息到本地
#[tauri::command]
async fn refresh_music_list(app: tauri::AppHandle) -> Result<Value, String> {
    refresh::refresh_music_data(&app)
        .await
        .map_err(|e| e.to_string())
}

// 获取封面路径（本地存在则直接返回，不存在则下载）
#[tauri::command]
async fn get_cover(
    app: tauri::AppHandle,
    album_id: String,
    cover_url: String,
) -> Result<String, String> {
    let path = cover::get_cover_path(&app, &album_id, &cover_url)
        .await
        .map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().to_string())
}

// 下载单首歌曲
#[tauri::command]
async fn download_music(app: tauri::AppHandle, song_id: String) -> Result<(), String> {
    download::download_music(&app, &song_id)
        .await
        .map_err(|e| e.to_string())
}

// 批量下载所有待下载歌曲
#[tauri::command]
async fn download_all_music(app: tauri::AppHandle) -> Result<(usize, usize), String> {
    download::download_all_music(&app)
        .await
        .map_err(|e| e.to_string())
}

// 预热 FFmpeg，避免首次调用耗时很长
async fn warmup_ffmpeg(app: tauri::AppHandle) {
    let start = Instant::now();
    match app.shell().sidecar("ffmpeg") {
        Ok(cmd) => {
            let _ = cmd.args(["-version"]).output().await;
            println!("[Init] FFmpeg 预热完成，耗时: {:?}", start.elapsed());
        }
        Err(e) => {
            println!("[Init] FFmpeg 预热失败: {}", e);
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                warmup_ffmpeg(handle).await;
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            load_latest_version,
            load_remote_music,
            load_local_music,
            refresh_music_list,
            get_cover,
            download_music,
            download_all_music,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
