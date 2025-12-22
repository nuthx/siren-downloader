use crate::api::fetch_song_detail;
use crate::config::{load_app_config, load_music_data, save_music_data};
use crate::cover::get_cover_path;
use crate::ffmpeg::get_ffmpeg_path;
use crate::utils::{fix_filename, fix_folder_name};
use anyhow::{bail, Context, Result};
use id3::frame::{Frame, Picture, PictureType};
use id3::{Tag, TagLike, Version};
use image::ImageReader;
use metaflac::block::PictureType as FlacPictureType;
use metaflac::Tag as FlacTag;
use reqwest::get;
use serde_json::json;
use std::collections::HashMap;
use std::fs::{create_dir_all, remove_file, write, File};
use std::io::BufReader;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter};
use tauri_plugin_shell::ShellExt;
use tokio::fs as async_fs;

// 下载单首歌曲
pub async fn download_music(app: &AppHandle, song_id: &str) -> Result<()> {
    let config = load_app_config(app).await?;

    // 检查下载路径
    let download_path = Path::new(&config.download_path);
    if download_path.as_os_str().is_empty() {
        bail!("请先在设置中填写下载路径");
    }

    // 检查 FFmpeg
    let ffmpeg_path = get_ffmpeg_path(app)?;

    // 获取本地歌曲数据并构建索引
    let mut songs = load_music_data(app).await?;
    let song_map: HashMap<&str, usize> = songs
        .iter()
        .enumerate()
        .map(|(i, s)| (s.id.as_str(), i))
        .collect();
    let song_idx = *song_map
        .get(song_id)
        .context("本地不存在该歌曲，请先更新歌曲列表")?;
    let song = songs[song_idx].clone();

    // 获取下载链接、歌词链接和扩展名
    println!("[Download] 获取歌曲信息: {}", song.title);
    let song_detail = fetch_song_detail(song_id).await?;
    let lyric_url = song_detail["lyricUrl"].as_str().unwrap_or("");
    let source_url = song_detail["sourceUrl"]
        .as_str()
        .context("获取下载连接失败")?;
    let mut format = source_url
        .rsplit('.')
        .next()
        .expect("获取扩展名失败")
        .to_lowercase();

    // 检查网易云封面，必须存在，否则说明歌曲信息获取有误
    println!("[Download] 获取歌曲封面: {}", song.title);
    let url = &song.cover_ncm;
    if url.is_empty() {
        bail!("网易云封面为空，请重新更新歌曲列表");
    }

    // 获取封面尺寸
    let cover_path = get_cover_path(app, &song.album_id, url).await?;
    let file = File::open(&cover_path).context("打开封面文件失败")?;
    let reader = ImageReader::new(BufReader::new(file))
        .with_guessed_format()
        .context("解析封面格式失败")?;
    let (width, height) = reader.into_dimensions().context("获取封面尺寸失败")?;

    // 分辨率大于 1500 则缩小，保存到当前封面目录
    let cover_data = if width > 1500 || height > 1500 {
        let resized_cover_path = cover_path.with_file_name(format!("{}_resized.jpg", song.album_id));
        let output = app
            .shell()
            .command(&ffmpeg_path)
            .args([
                "-i",
                cover_path.to_str().context("封面路径无效")?,
                "-vf",
                "scale=1500:1500:force_original_aspect_ratio=decrease",
                "-q:v",
                "3",
                "-y",
                resized_cover_path.to_str().context("缩放封面路径无效")?,
                "-loglevel",
                "error",
            ])
            .output()
            .await
            .context("封面缩放失败")?;
        if !output.status.success() {
            bail!("封面缩放失败: {}", String::from_utf8_lossy(&output.stderr));
        }
        
        // 读取缩放后的文件数据
        let data = async_fs::read(&resized_cover_path)
            .await
            .context("读取缩放封面数据失败")?;
        
        // 删除缩放后的文件
        let _ = remove_file(&resized_cover_path);
        
        data
    } else {
        async_fs::read(&cover_path)
            .await
            .context("读取封面数据失败")?
    };

    // 构建文件路径
    let folder_path = PathBuf::from(&config.download_path).join(fix_folder_name(&song.album_title));
    create_dir_all(&folder_path).context("无法创建下载目录")?;
    let base_path = folder_path.join(fix_filename(&song.title));

    // 下载歌曲并保存
    println!("[Download] 开始下载歌曲: {}", song.title);
    write(
        base_path.with_extension(&format),
        get(source_url)
            .await
            .context("下载歌曲文件失败")?
            .bytes()
            .await
            .context("解析歌曲字节失败")?,
    )
    .context("保存歌曲文件失败")?;

    // 如果开启歌词下载，且存在歌词链接，则下载并保存为 LRC 文件
    if config.download_lyrics && !lyric_url.is_empty() {
        println!("[Download] 开始下载歌词: {}", song.title);
        write(
            base_path.with_extension("lrc"),
            get(lyric_url)
                .await
                .context("下载歌词文件失败")?
                .text()
                .await
                .context("解析歌词文本失败")?,
        )
        .context("保存歌词文件失败")?;
    }

    // WAV 需要转换为 FLAC
    if format == "wav" {
        println!("[Download] 转换歌曲格式: {}", song.title);
        let output = app
            .shell()
            .command(&ffmpeg_path)
            .args([
                "-i",
                base_path
                    .with_extension("wav")
                    .to_str()
                    .context("WAV 路径无效")?,
                "-codec:a",
                "flac",
                "-level",
                "0",
                "-y",
                base_path
                    .with_extension("flac")
                    .to_str()
                    .context("FLAC 路径无效")?,
                "-loglevel",
                "error",
            ])
            .output()
            .await
            .context(format!("歌曲转换失败: {}", song.title))?;
        if !output.status.success() {
            bail!("歌曲转换失败: {}", String::from_utf8_lossy(&output.stderr));
        }
        format = "flac".to_string();
        if let Err(e) = remove_file(base_path.with_extension("wav")) {
            println!("[Download] 删除 WAV 失败，请手动删除: {}", e);
        }
    }

    // 写入元数据
    println!("[Download] 写入歌曲元数据: {}", song.title);
    let song_path = base_path.with_extension(&format);
    if format == "mp3" {
        let mut tag = Tag::new();
        tag.set_title(&song.title);
        tag.set_artist("塞壬唱片-MSR");
        tag.set_album(&song.album_title);
        tag.set_album_artist("塞壬唱片-MSR");
        tag.set_disc(1);
        tag.set_track(song.track as u32);
        tag.add_frame(Frame::text("TDRC", song.publish.get(..4).unwrap_or("")));
        tag.set_genre("Arknights");
        tag.add_frame(Picture {
            mime_type: "image/jpeg".to_string(),
            picture_type: PictureType::CoverFront,
            description: String::new(),
            data: cover_data,
        });
        tag.write_to_path(&song_path, Version::Id3v24)
            .context(format!("写入元数据失败: {}", song.title))?;
    } else if format == "flac" {
        let mut tag = FlacTag::read_from_path(&song_path).context("读取 FLAC 标签失败")?;
        let comments = tag.vorbis_comments_mut();
        comments.set_title(vec![song.title.clone()]);
        comments.set_artist(vec!["塞壬唱片-MSR".to_string()]);
        comments.set_album(vec![song.album_title.clone()]);
        comments.set_album_artist(vec!["塞壬唱片-MSR".to_string()]);
        comments.set("DISCNUMBER", vec!["1"]);
        comments.set_track(song.track as u32);
        comments.set("DATE", vec![song.publish.get(..4).unwrap_or("")]);
        comments.set_genre(vec!["Arknights".to_string()]);
        tag.add_picture("image/jpeg", FlacPictureType::CoverFront, cover_data);
        tag.write_to_path(&song_path)
            .context(format!("写入元数据失败: {}", song.title))?;
    } else {
        bail!(
            "当前版本不支持写入 {} 格式的元数据，请发起 Issues 反馈作者",
            format.to_uppercase()
        );
    }

    // 更新本地 JSON 中的下载状态
    songs[song_idx].download = true;
    save_music_data(app, &songs).await?;
    println!("[Download] 下载完成: {}", song.title);

    Ok(())
}

// 批量下载所有歌曲
pub async fn download_all_music(app: &AppHandle) -> Result<(usize, usize)> {
    let config = load_app_config(app).await?;
    let songs = load_music_data(app).await?;

    // 待下载歌曲
    let pending: Vec<_> = songs
        .iter()
        .filter(|s| !s.download && (config.download_instrumental || !s.instrumental))
        .collect();

    let total = pending.len();
    let mut success_count = 0;
    let mut fail_count = 0;

    for (i, song) in pending.iter().enumerate() {
        let success = download_music(app, &song.id).await.is_ok();
        if success {
            success_count += 1;
        } else {
            fail_count += 1;
        }

        // 发送事件
        let _ = app.emit(
            "download-progress",
            json!({
                "current": i + 1,
                "total": total,
                "song_id": song.id,
                "song_title": song.title,
                "success": success
            }),
        );
    }

    Ok((success_count, fail_count))
}
