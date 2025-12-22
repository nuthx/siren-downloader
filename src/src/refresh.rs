use crate::api::{fetch_album_detail, fetch_all_songs, fetch_ncm_albums};
use crate::config::{load_album_match, load_music_data, save_music_data};
use crate::models::SongDetail;
use anyhow::Result;
use chrono::DateTime;
use futures::stream::{self, StreamExt};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use tauri::AppHandle;

pub async fn refresh_music_data(app: &AppHandle) -> Result<Value> {
    // 加载歌曲列表
    let local_songs: Vec<SongDetail> = load_music_data(app).await?;
    let remote_songs: Vec<SongDetail> = fetch_all_songs()
        .await?
        .iter()
        .filter_map(SongDetail::from_siren_api)
        .collect();

    // 将本地歌曲转换为字典，便于下一步查找
    let mut local_songs_map: HashMap<String, SongDetail> =
        local_songs.into_iter().map(|s| (s.id.clone(), s)).collect();

    // 合并本地与远程歌曲，并按照远程顺序排序
    let mut final_songs: Vec<SongDetail> = remote_songs
        .into_iter()
        .map(|remote| local_songs_map.remove(&remote.id).unwrap_or(remote))
        .collect();

    // 特殊处理: Sanctuary Inside 这首歌曲的顺序有问题，需要将 125042 移动到 880300 前面
    if let (Some(a), Some(b)) = (
        final_songs.iter().position(|s| s.id == "125042"),
        final_songs.iter().position(|s| s.id == "880300"),
    ) {
        if a > b {
            let song = final_songs.remove(a);
            final_songs.insert(b, song);
        }
    }

    // 收集缺少专辑信息的专辑ID，用于下一步在API获取这些专辑信息，避免全部请求
    let album_ids: HashSet<String> = final_songs
        .iter()
        .filter(|song| song.album_title.is_empty())
        .map(|song| song.album_id.clone())
        .collect();

    if !album_ids.is_empty() {
        // 并发获取缺失的专辑信息，最多同时 10 个请求
        let albums: Vec<_> = stream::iter(album_ids)
            .map(fetch_album_detail)
            .buffer_unordered(10)
            .collect()
            .await;

        // 构建专辑信息映射表，用于下一步快速查询并写入歌曲
        let album_map: HashMap<String, (String, String)> = albums
            .into_iter()
            .flatten()
            .filter_map(|album| {
                let album_id = album.get("cid")?.as_str()?.to_string();
                let album_name = album
                    .get("name")
                    .and_then(|a| a.as_str())
                    .unwrap_or("")
                    .to_string();
                let cover = album
                    .get("coverUrl")
                    .and_then(|c| c.as_str())
                    .unwrap_or("")
                    .to_string();
                Some((album_id, (album_name, cover)))
            })
            .collect();

        // 写入专辑名和塞壬封面到SongDetail
        for song in final_songs.iter_mut() {
            if let Some((album_title, cover)) = album_map.get(&song.album_id) {
                song.album_title = album_title.clone();
                song.cover_siren = cover.clone();
            }
        }
    }

    // 获取网易云的塞壬唱片的所有专辑信息
    let ncm_albums = fetch_ncm_albums(app).await?;

    // 预读匹配列表，并构建专辑名到专辑信息的映射
    let match_list = load_album_match(app).await;
    let ncm_map: HashMap<_, _> = ncm_albums
        .iter()
        .filter_map(|a| a["name"].as_str().map(|n| (n.trim(), a)))
        .collect();

    for song in final_songs.iter_mut() {
        // 优先从匹配列表查询专辑名，没有则使用原专辑名
        let ncm_album_name = match_list
            .get(&song.album_title)
            .unwrap_or(&song.album_title)
            .trim();

        // 查询匹配并写入网易云的封面和发布时间到SongDetail
        if let Some(ncm_album) = ncm_map.get(&ncm_album_name) {
            song.publish =
                DateTime::from_timestamp(ncm_album["publishTime"].as_i64().unwrap() / 1000, 0)
                    .unwrap()
                    .format("%Y-%m-%d")
                    .to_string();
            song.cover_ncm = ncm_album["picUrl"].as_str().unwrap().to_string();
        }
    }

    // 计算曲目编号
    let mut track = 0;
    let mut current_album: &str = "";
    for song in final_songs.iter_mut() {
        if current_album == song.album_title.as_str() {
            track += 1;
        } else {
            current_album = song.album_title.as_str();
            track = 1;
        }
        song.track = track;
    }

    // 保存到本地
    save_music_data(app, &final_songs).await?;

    Ok(json!(final_songs))
}
