use anyhow::{anyhow, Context, Result};
use reqwest::{get, Client};
use serde_json::Value;

// 获取 Github 上的最新版本号
pub async fn fetch_latest_version() -> Result<String> {
    let client = Client::new();
    let response = client
        .get("https://api.github.com/repos/nuthx/siren-downloader/releases/latest")
        .header("User-Agent", "siren-downloader")
        .send()
        .await?
        .json::<Value>()
        .await?;

    let version = response["tag_name"]
        .as_str()
        .context("获取最新版本失败")?
        .trim_start_matches('v')
        .to_string();

    println!("[API] 最新版本: {}", version);
    Ok(version)
}

// 从官网获取所有歌曲
pub async fn fetch_all_songs() -> Result<Vec<Value>> {
    let response = get("https://monster-siren.hypergryph.com/api/songs")
        .await?
        .json::<Value>()
        .await?;

    let songs: Vec<Value> = response["data"]["list"]
        .as_array()
        .context("/api/songs 解析失败")?
        .iter()
        .rev() // 翻转歌曲顺序，最新的在后面
        .cloned()
        .collect();

    println!("[API] 官网发现 {} 首歌曲", songs.len());
    Ok(songs)
}

// 从官网获取专辑详情
pub async fn fetch_album_detail(album_id: String) -> Result<Value> {
    let response = get(&format!(
        "https://monster-siren.hypergryph.com/api/album/{}/detail",
        album_id
    ))
    .await?
    .json::<Value>()
    .await?;

    let data = response["data"].clone();

    println!("[API] 专辑 {} 获取成功: {}", album_id, data["name"]);
    Ok(data)
}

// 从官网获取歌曲详情
pub async fn fetch_song_detail(song_id: &str) -> Result<Value> {
    let response = get(&format!(
        "https://monster-siren.hypergryph.com/api/song/{}",
        song_id
    ))
    .await?
    .json::<Value>()
    .await?;

    let data = response["data"].clone();

    println!("[API] 歌曲 {} 获取成功: {}", song_id, data["name"]);
    Ok(data)
}

// 从网易云获取专辑列表
pub async fn fetch_ncm_albums() -> Result<Vec<Value>> {
    let client = Client::new();
    let response = client
        .get("https://music.163.com/api/artist/albums/32540734?offset=0&limit=1000")
        .header(
            "User-Agent",
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36",
        )
        .header("Referer", "https://music.163.com/")
        .send()
        .await?
        .json::<Value>()
        .await?;

    let albums = response["hotAlbums"]
        .as_array()
        .cloned()
        .ok_or_else(|| anyhow!("网易云 API 请求失败"))?;

    println!("[API] 网易云发现 {} 张专辑", albums.len());
    Ok(albums)
}
