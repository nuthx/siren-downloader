use crate::api::fetch_song_detail;
use crate::config::{load_app_config, load_music_data, save_music_data};
use crate::cover::get_cover_path;
use crate::ffmpeg::get_ffmpeg_path;
use crate::utils::{fix_folder_name, render_song_template};
use anyhow::{bail, Context, Result};
use futures::StreamExt;
use image::ImageReader;
use metaflac::block::PictureType as FlacPictureType;
use metaflac::Tag as FlacTag;
use reqwest::get;
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::fs::{create_dir_all, remove_file, write, File};
use std::io::BufReader;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_shell::ShellExt;
use tokio::fs as async_fs;
use tokio::io::AsyncWriteExt;

fn output_formats(audio_format: &str) -> (bool, bool) {
    match audio_format {
        "wav" => (true, false),
        "wav_flac" => (true, true),
        _ => (false, true),
    }
}

fn song_base_path(
    download_path: &str,
    config: &crate::models::AppConfig,
    song: &SongDetail,
) -> PathBuf {
    let folder_template = if config.album_folder_template.trim().is_empty() {
        "{album}"
    } else {
        &config.album_folder_template
    };
    let file_template = if config.file_name_template.trim().is_empty() {
        "{title}"
    } else {
        &config.file_name_template
    };
    let folder_name = fix_folder_name(&render_song_template(folder_template, song));
    let folder_name = if folder_name.is_empty() {
        "未命名专辑".to_string()
    } else {
        folder_name
    };
    let file_name = fix_folder_name(&render_song_template(file_template, song));
    let file_name = if file_name.is_empty() {
        song.id.clone()
    } else {
        file_name
    };

    PathBuf::from(download_path)
        .join(folder_name)
        .join(file_name)
}

async fn convert_to_flac(
    app: &AppHandle,
    ffmpeg_path: &str,
    input_path: &Path,
    output_path: &Path,
    song_title: &str,
) -> Result<()> {
    let output = app
        .shell()
        .command(ffmpeg_path)
        .args([
            "-loglevel",
            "error",
            "-i",
            input_path.to_str().context("音频路径无效")?,
            "-map",
            "0:a:0",
            "-codec:a",
            "flac",
            "-level",
            "0",
            "-y",
            output_path.to_str().context("FLAC 路径无效")?,
        ])
        .output()
        .await
        .context(format!("歌曲转换失败: {song_title}"))?;

    if !output.status.success() {
        bail!("歌曲转换失败: {}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(())
}

async fn write_wav_metadata(
    app: &AppHandle,
    ffmpeg_path: &str,
    input_path: &Path,
    output_path: &Path,
    song: &SongDetail,
    publish_date: &str,
) -> Result<()> {
    let audio_codec = if input_path.extension().and_then(|ext| ext.to_str()) == Some("wav") {
        "copy"
    } else {
        "pcm_s16le"
    };
    let track_number = song.track.to_string();
    let mut args = vec![
        "-loglevel".to_string(),
        "error".to_string(),
        "-i".to_string(),
        input_path.to_string_lossy().to_string(),
        "-map".to_string(),
        "0:a:0".to_string(),
        "-codec:a".to_string(),
        audio_codec.to_string(),
    ];
    for (key, value) in [
        ("title", song.title.as_str()),
        ("artist", "塞壬唱片-MSR"),
        ("album", song.album_title.as_str()),
        ("album_artist", "塞壬唱片-MSR"),
        ("track", track_number.as_str()),
        ("date", publish_date),
        ("genre", "Arknights"),
    ] {
        args.push("-metadata".to_string());
        args.push(format!("{key}={value}"));
    }
    args.extend([
        "-y".to_string(),
        output_path.to_string_lossy().to_string(),
    ]);

    let output = app
        .shell()
        .command(ffmpeg_path)
        .args(args)
        .output()
        .await
        .context(format!("写入 WAV 元数据失败: {}", song.title))?;
    if !output.status.success() {
        bail!(
            "写入 WAV 元数据失败: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(())
}

fn write_flac_metadata(
    song_path: &Path,
    song: &SongDetail,
    cover_data: &[u8],
    publish_date: &str,
) -> Result<()> {
    let mut tag = FlacTag::read_from_path(song_path).context("读取 FLAC 标签失败")?;
    let comments = tag.vorbis_comments_mut();
    comments.set_title(vec![song.title.clone()]);
    comments.set_artist(vec!["塞壬唱片-MSR".to_string()]);
    comments.set_album(vec![song.album_title.clone()]);
    comments.set_album_artist(vec!["塞壬唱片-MSR".to_string()]);
    comments.set("DISCNUMBER", vec!["1"]);
    comments.set_track(song.track as u32);
    comments.set("DATE", vec![publish_date]);
    comments.set_genre(vec!["Arknights".to_string()]);
    tag.add_picture(
        "image/jpeg",
        FlacPictureType::CoverFront,
        cover_data.to_vec(),
    );
    tag.write_to_path(song_path)
        .context("写入 FLAC 元数据失败")?;
    Ok(())
}

// 下载单首歌曲
async fn download_one_music(app: &AppHandle, song_id: &str) -> Result<()> {
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
    let source_format = source_url
        .split('?')
        .next()
        .and_then(|url| url.rsplit_once('.').map(|(_, extension)| extension))
        .filter(|extension| !extension.is_empty())
        .unwrap_or("wav")
        .to_ascii_lowercase();
    let source_format = if ["wav", "flac", "mp3"].contains(&source_format.as_str()) {
        source_format
    } else {
        "wav".to_string()
    };

    let (_, keep_flac) = output_formats(&config.audio_format);
    let url = &song.cover_ncm;
    if keep_flac && url.is_empty() {
        bail!("网易云封面为空，请重新更新歌曲列表");
    }

    let cover_data = if keep_flac {
        println!("[Download] 获取歌曲封面: {}", song.title);
        let cover_path = get_cover_path(app, &song.album_id, url).await?;
        let file = File::open(&cover_path).context("打开封面文件失败")?;
        let reader = ImageReader::new(BufReader::new(file))
            .with_guessed_format()
            .context("解析封面格式失败")?;
        let (width, height) = reader.into_dimensions().context("获取封面尺寸失败")?;

        if width > 1500 || height > 1500 {
            let resized_cover_path =
                cover_path.with_file_name(format!("{}_resized.jpg", song.album_id));
            let output = app
                .shell()
                .command(&ffmpeg_path)
                .args([
                    "-loglevel",
                    "error",
                    "-i",
                    cover_path.to_str().context("封面路径无效")?,
                    "-vf",
                    "scale=1500:1500:force_original_aspect_ratio=decrease",
                    "-q:v",
                    "3",
                    "-y",
                    resized_cover_path.to_str().context("缩放封面路径无效")?,
                ])
                .output()
                .await
                .context("封面缩放失败")?;
            if !output.status.success() {
                bail!("封面缩放失败: {}", String::from_utf8_lossy(&output.stderr));
            }

            let data = async_fs::read(&resized_cover_path)
                .await
                .context("读取缩放封面数据失败")?;
            let _ = remove_file(&resized_cover_path);
            data
        } else {
            async_fs::read(&cover_path)
                .await
                .context("读取封面数据失败")?
        }
    } else {
        Vec::new()
    };

    let base_path = song_base_path(&config.download_path, &config, &song);
    let folder_path = base_path.parent().context("歌曲目录无效")?;
    create_dir_all(folder_path).context("无法创建下载目录")?;

    // 先创建空的歌曲文件
    println!("[Download] 开始下载歌曲: {}", song.title);
    let response = get(source_url).await.context("下载歌曲文件失败")?;
    let total_size = response.content_length().unwrap_or(0);
    let source_file_path = base_path.with_extension(&source_format);
    let mut file = async_fs::File::create(&source_file_path)
        .await
        .context("创建歌曲文件失败")?;

    // 分块写入歌曲数据
    let mut downloaded: u64 = 0;
    let mut last_progress: u32 = 0;
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("下载数据块失败")?;
        file.write_all(&chunk).await.context("写入文件失败")?;
        downloaded += chunk.len() as u64;

        if total_size > 0 {
            let progress = (downloaded * 100 / total_size) as u32;
            if progress >= last_progress + 2 || progress == 100 {
                last_progress = progress;
                let _ = app.emit(
                    "download-single-progress",
                    json!({
                        "song_id": song_id,
                        "progress": progress
                    }),
                );
            }
        }
    }

    file.flush().await.context("刷新文件缓冲失败")?;

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

    let (keep_wav, keep_flac) = output_formats(&config.audio_format);
    let wav_path = base_path.with_extension("wav");
    let flac_path = base_path.with_extension("flac");
    let publish_date = match config.id3_date_format.as_str() {
        "full" => &song.publish,
        _ => song.publish.get(..4).unwrap_or(""),
    };

    if keep_flac {
        if source_file_path != flac_path {
            println!("[Download] 转换歌曲格式: {}", song.title);
            convert_to_flac(
                app,
                &ffmpeg_path,
                &source_file_path,
                &flac_path,
                &song.title,
            )
            .await?;
        }
        println!("[Download] 写入 FLAC 元数据: {}", song.title);
        write_flac_metadata(&flac_path, &song, &cover_data, publish_date)?;
    }

    if keep_wav {
        println!("[Download] 写入 WAV 元数据: {}", song.title);
        let tagged_wav_path = base_path.with_extension("tagged.wav");
        write_wav_metadata(
            app,
            &ffmpeg_path,
            &source_file_path,
            &tagged_wav_path,
            &song,
            publish_date,
        )
        .await?;
        if wav_path.exists() {
            remove_file(&wav_path).context("替换 WAV 文件失败")?;
        }
        rename(&tagged_wav_path, &wav_path).context("保存 WAV 文件失败")?;
    }

    let source_is_output = (keep_wav && source_file_path == wav_path)
        || (keep_flac && source_file_path == flac_path);
    if !source_is_output && source_file_path.exists() {
        if let Err(error) = remove_file(&source_file_path) {
            println!("[Download] 删除临时音频失败，请手动删除: {error}");
        }
    }

    // 更新本地 JSON 中的下载状态
    songs[song_idx].download = true;
    save_music_data(app, &songs).await?;
    println!("[Download] 下载完成: {}", song.title);

    Ok(())
}

// 下载歌曲
pub async fn download_music(
    app: &AppHandle,
    song_ids: Option<&[String]>,
) -> Result<(usize, usize)> {
    let songs = load_music_data(app).await?;
    let queue: Vec<(String, String)> = match song_ids {
        Some(song_ids) => {
            let song_titles: HashMap<&str, &str> = songs
                .iter()
                .map(|song| (song.id.as_str(), song.title.as_str()))
                .collect();
            let mut seen = HashSet::new();

            song_ids
                .iter()
                .filter_map(|song_id| {
                    let song_id = song_id.as_str();
                    if !seen.insert(song_id) {
                        return None;
                    }

                    song_titles
                        .get(song_id)
                        .map(|title| (song_id.to_string(), (*title).to_string()))
                })
                .collect()
        }
        None => {
            let config = load_app_config(app).await?;
            songs
                .iter()
                .filter(|song| {
                    !song.download && (config.download_instrumental || !song.instrumental)
                })
                .map(|song| (song.id.clone(), song.title.clone()))
                .collect()
        }
    };

    let total = queue.len();
    if total == 0 {
        return Ok((0, 0));
    }

    // 获取并发下载数量配置
    let config = load_app_config(app).await?;
    let concurrent_downloads = config.concurrent_downloads.clamp(1, 10);

    let mut success_count = 0;
    let mut fail_count = 0;

    // 将 queue 转换为 owned 数据
    let queue_owned: Vec<(usize, String, String)> = queue
        .iter()
        .enumerate()
        .map(|(i, (song_id, song_title))| (i, song_id.clone(), song_title.clone()))
        .collect();

    // 使用 futures 的 stream 实现并发下载
    use futures::stream;
    let results: Vec<_> = stream::iter(queue_owned)
        .map(|(i, song_id, song_title)| {
            let app = app.clone();

            async move {
                // 发送开始事件
                let _ = app.emit(
                    "download-progress",
                    json!({
                        "current": i + 1,
                        "total": total,
                        "song_id": song_id,
                        "song_title": song_title,
                        "stage": "start"
                    }),
                );

                // 执行下载
                let result = download_one_music(&app, &song_id).await;
                let success = result.is_ok();

                // 发送完成事件
                let _ = app.emit(
                    "download-progress",
                    json!({
                        "current": i + 1,
                        "total": total,
                        "song_id": song_id,
                        "song_title": song_title,
                        "success": success,
                        "stage": "complete"
                    }),
                );

                (i, success, result)
            }
        })
        .buffer_unordered(concurrent_downloads)
        .collect()
        .await;

    // 统计结果
    for (i, success, result) in results {
        if success {
            success_count += 1;
        } else {
            fail_count += 1;
        }

        // 如果只有一首歌且失败，抛出错误
        if total == 1 && i == 0 {
            result?;
        }
    }

    Ok((success_count, fail_count))
}

pub async fn local_song_path(app: &AppHandle, song_id: &str) -> Result<PathBuf> {
    let config = load_app_config(app).await?;
    let songs = load_music_data(app).await?;
    let song = songs
        .iter()
        .find(|song| song.id == song_id)
        .context("本地不存在该歌曲，请先更新歌曲列表")?;
    let base_path = song_base_path(&config.download_path, &config, song);
    let (keep_wav, keep_flac) = output_formats(&config.audio_format);

    let extensions = if keep_wav && !keep_flac {
        ["wav", "flac"]
    } else {
        ["flac", "wav"]
    };
    let path = extensions
        .into_iter()
        .map(|extension| base_path.with_extension(extension))
        .find(|path| path.is_file())
        .context("未找到本地音频文件，请先下载歌曲")?;
    let path = path.canonicalize().context("读取本地音频路径失败")?;

    app.asset_protocol_scope()
        .allow_file(&path)
        .context("无法打开本地音频文件")?;
    Ok(path)
}
