use crate::models::{AppConfig, SongDetail};
use anyhow::{anyhow, Context, Result};
use serde_json::{from_str, from_value, to_string_pretty};
use std::collections::HashMap;
use std::fs::{self, write};
use tauri::{AppHandle, Manager};
use tauri_plugin_store::StoreExt;

// 读取应用配置
pub async fn load_app_config(app: &AppHandle) -> Result<AppConfig> {
    app.store("config.json")
        .context("无法读取配置文件")?
        .get("app_config")
        .ok_or_else(|| anyhow!("配置不存在"))
        .and_then(|v| from_value(v).context("配置解析失败"))
}

// 读取专辑映射
pub async fn load_album_match(app: &AppHandle) -> HashMap<String, String> {
    app.store("config.json")
        .ok()
        .and_then(|s| s.get("album_match"))
        .and_then(|v| from_value(v).ok())
        .unwrap_or_default()
}

// 读取歌曲列表
pub async fn load_music_data(app: &AppHandle) -> Result<Vec<SongDetail>> {
    let music_file = app
        .path()
        .app_data_dir()
        .context("无法访问应用目录")?
        .join("music.json");

    // 如果不存在则返回空列表
    if !music_file.exists() {
        return Ok(Vec::new());
    }

    let data = fs::read_to_string(&music_file).context("无法读取歌曲列表")?;
    let list: Vec<SongDetail> = from_str(&data).context("歌曲列表解析失败")?;
    Ok(list)
}

// 保存歌曲列表
pub async fn save_music_data(app: &AppHandle, songs: &[SongDetail]) -> Result<()> {
    let music_file = app
        .path()
        .app_data_dir()
        .context("无法访问应用目录")?
        .join("music.json");

    write(&music_file, to_string_pretty(&songs).unwrap())?;
    Ok(())
}
