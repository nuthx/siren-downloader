mod api;
mod config;
mod cover;
mod download;
mod ffmpeg;
mod menu;
mod models;
mod refresh;
mod utils;

use models::SongDetail;
use serde_json::Value;

// 获取平台名称
#[tauri::command]
fn get_platform() -> String {
    std::env::consts::OS.to_string()
}

// 检查更新
#[tauri::command]
async fn load_latest_version() -> Result<String, String> {
    api::fetch_latest_version().await.map_err(|e| e.to_string())
}

// 获取 FFmpeg 版本
#[tauri::command]
async fn get_ffmpeg_version(app: tauri::AppHandle) -> Result<String, String> {
    let ffmpeg_path = ffmpeg::get_ffmpeg_path(&app).map_err(|e| e.to_string())?;
    ffmpeg::check_ffmpeg_version(&app, &ffmpeg_path)
        .await
        .map_err(|e| e.to_string())
}

// 下载 FFmpeg
#[tauri::command]
async fn download_ffmpeg(app: tauri::AppHandle) -> Result<(), String> {
    ffmpeg::download_ffmpeg(&app)
        .await
        .map_err(|e| e.to_string())
}

// 删除 FFmpeg
#[tauri::command]
async fn delete_ffmpeg(app: tauri::AppHandle) -> Result<(), String> {
    ffmpeg::delete_ffmpeg(&app).await.map_err(|e| e.to_string())
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            #[cfg(target_os = "macos")]
            {
                let menu = menu::create_menu(app)?;
                app.set_menu(menu)?;
            }
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
            get_ffmpeg_version,
            get_platform,
            download_ffmpeg,
            delete_ffmpeg,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
