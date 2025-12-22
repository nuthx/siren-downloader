use anyhow::{bail, Context, Result};
use reqwest::get;
use std::fs::{create_dir_all, remove_file, rename, File};
use std::io::{copy, Write};
use std::path::Path;
use tauri::{AppHandle, Manager};
use tauri_plugin_shell::ShellExt;

// 获取 FFmpeg 路径
pub fn get_ffmpeg_path(app: &AppHandle) -> Result<String> {
    if cfg!(target_os = "windows") {
        let config_dir = app.path().app_data_dir().context("无法访问应用目录")?;
        let ffmpeg_path = config_dir.join("bin").join("ffmpeg.exe");
        if ffmpeg_path.exists() {
            Ok(ffmpeg_path.to_string_lossy().to_string())
        } else {
            bail!("请先下载 FFmpeg")
        }
    } else if cfg!(target_os = "macos") {
        let paths = ["/opt/homebrew/bin/ffmpeg", "/usr/local/bin/ffmpeg"];
        for p in paths {
            if Path::new(p).exists() {
                return Ok(p.to_string());
            }
        }
        bail!("请先通过 Homebrew 安装 FFmpeg: brew install ffmpeg");
    } else {
        Ok("ffmpeg".to_string())
    }
}

// 检查 FFmpeg 版本
pub async fn check_ffmpeg_version(app: &AppHandle, ffmpeg_path: &str) -> Result<String> {
    let output = app
        .shell()
        .command(ffmpeg_path)
        .args(["-version"])
        .output()
        .await
        .context("FFmpeg 执行失败")?;

    if !output.status.success() {
        if cfg!(target_os = "macos") {
            bail!("FFmpeg 未安装，请先通过 Homebrew 安装: brew install ffmpeg");
        } else {
            bail!("FFmpeg 执行失败");
        }
    }

    let version = String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .unwrap_or("FFmpeg 版本获取失败")
        .to_string();

    println!("[FFmpeg] {}", version);
    Ok(version)
}

// 下载 FFmpeg
pub async fn download_ffmpeg(app: &AppHandle) -> Result<()> {
    let bin_dir = app
        .path()
        .app_data_dir()
        .context("无法访问应用目录")?
        .join("bin");
    create_dir_all(&bin_dir).context("创建 bin 目录失败")?;

    let ffmpeg_path = bin_dir.join("ffmpeg.exe");
    let ffmpeg_new_path = bin_dir.join("ffmpeg_new.exe");
    let temp_7z_path = bin_dir.join("ffmpeg_temp.7z");

    // 下载 7z 到临时文件
    println!("[FFmpeg] 开始下载 FFmpeg");
    let bytes = get("https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.7z")
        .await
        .context("下载 FFmpeg 失败")?
        .bytes()
        .await
        .context("读取下载数据失败")?;

    File::create(&temp_7z_path)
        .context("创建临时文件失败")?
        .write_all(&bytes)
        .context("写入临时文件失败")?;

    // 从临时文件解压出 ffmpeg.exe
    println!("[FFmpeg] 开始解压 FFmpeg");
    let temp_file = File::open(&temp_7z_path).context("打开临时文件失败")?;
    sevenz_rust::decompress_with_extract_fn(temp_file, &bin_dir, |entry, reader, _| {
        if entry.name().ends_with("ffmpeg.exe") {
            copy(reader, &mut File::create(&ffmpeg_new_path)?)?;
            Ok(false)
        } else {
            Ok(false)
        }
    })
    .context("解压 FFmpeg 失败")?;

    // 清理临时文件并替换旧版本
    remove_file(&temp_7z_path).ok();
    remove_file(&ffmpeg_path).ok();
    rename(&ffmpeg_new_path, &ffmpeg_path).context("重命名新文件失败")?;

    println!("[FFmpeg] FFmpeg 下载成功");
    Ok(())
}

// 删除 FFmpeg
pub async fn delete_ffmpeg(app: &AppHandle) -> Result<()> {
    let config_dir = app.path().app_data_dir().context("无法访问应用目录")?;
    let ffmpeg_path = config_dir.join("bin").join("ffmpeg.exe");

    if !ffmpeg_path.exists() {
        bail!("FFmpeg 不存在");
    }

    remove_file(&ffmpeg_path).context("删除 FFmpeg 失败")?;
    println!("[FFmpeg] FFmpeg 已删除");
    Ok(())
}
