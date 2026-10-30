use anyhow::{bail, Context, Result};
use reqwest::get;
use serde::Serialize;
use std::env;
use std::fs::{create_dir_all, remove_file, rename, File};
use std::io::copy;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tauri::ipc::Channel;
use tauri::{AppHandle, Manager};
use tauri_plugin_shell::ShellExt;
use tokio::io::AsyncWriteExt;

#[derive(Clone, Serialize)]
pub struct DownloadProgress {
    stage: &'static str,
    downloaded: u64,
    total: Option<u64>,
}

fn find_ffmpeg_in_paths(paths: Option<std::ffi::OsString>) -> Option<PathBuf> {
    let paths = paths?;
    let executable = if cfg!(target_os = "windows") {
        "ffmpeg.exe"
    } else {
        "ffmpeg"
    };

    env::split_paths(&paths)
        .map(|directory| directory.join(executable))
        .find(|path| path.is_file())
}

// 获取 FFmpeg 路径
pub fn get_ffmpeg_path(app: &AppHandle) -> Result<String> {
    if cfg!(target_os = "windows") {
        let config_dir = app.path().app_data_dir().context("无法访问应用目录")?;
        let ffmpeg_path = config_dir.join("bin").join("ffmpeg.exe");
        if ffmpeg_path.is_file() {
            Ok(ffmpeg_path.to_string_lossy().to_string())
        } else if let Some(path) = find_ffmpeg_in_paths(env::var_os("PATH")) {
            Ok(path.to_string_lossy().to_string())
        } else {
            bail!("请先安装 FFmpeg，或通过应用下载 FFmpeg")
        }
    } else if cfg!(target_os = "macos") {
        if let Some(path) = find_ffmpeg_in_paths(env::var_os("PATH")) {
            return Ok(path.to_string_lossy().to_string());
        }

        let paths = ["/opt/homebrew/bin/ffmpeg", "/usr/local/bin/ffmpeg"];
        for p in paths {
            if Path::new(p).is_file() {
                return Ok(p.to_string());
            }
        }
        bail!("请先通过 Homebrew 安装 FFmpeg: brew install ffmpeg");
    } else {
        Ok(find_ffmpeg_in_paths(env::var_os("PATH"))
            .map(|path| path.to_string_lossy().to_string())
            .unwrap_or_else(|| "ffmpeg".to_string()))
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
pub async fn download_ffmpeg(
    app: &AppHandle,
    on_progress: Channel<DownloadProgress>,
) -> Result<()> {
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
    let mut response = get("https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.7z")
        .await
        .context("下载 FFmpeg 失败")?
        .error_for_status()
        .context("下载 FFmpeg 失败")?;
    let mut progress = DownloadProgress {
        stage: "downloading",
        downloaded: 0,
        total: response.content_length().filter(|total| *total > 0),
    };
    // 页面关闭时仍继续安装，进度通知失败不影响下载。
    let _ = on_progress.send(progress.clone());
    let mut temp_file = tokio::fs::File::create(&temp_7z_path)
        .await
        .context("创建临时文件失败")?;
    let mut last_update = Instant::now();
    while let Some(chunk) = response.chunk().await.context("读取下载数据失败")? {
        temp_file
            .write_all(&chunk)
            .await
            .context("写入临时文件失败")?;
        progress.downloaded += chunk.len() as u64;
        if last_update.elapsed() >= Duration::from_millis(100) {
            let _ = on_progress.send(progress.clone());
            last_update = Instant::now();
        }
    }
    temp_file.flush().await.context("写入临时文件失败")?;
    drop(temp_file);

    // 从临时文件解压出 ffmpeg.exe
    println!("[FFmpeg] 开始解压 FFmpeg");
    progress.stage = "extracting";
    let _ = on_progress.send(progress);
    let temp_file = File::open(&temp_7z_path).context("打开临时文件失败")?;
    sevenz_rust::decompress_with_extract_fn(temp_file, &bin_dir, |entry, reader, _| {
        if entry.name().ends_with("ffmpeg.exe") {
            copy(reader, &mut File::create(&ffmpeg_new_path)?)?;
        }
        Ok(false)
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

#[cfg(test)]
mod tests {
    use super::find_ffmpeg_in_paths;
    use std::env;
    use std::fs::{create_dir_all, remove_dir_all, File};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn finds_ffmpeg_in_path_directories() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after UNIX epoch")
            .as_nanos();
        let root = env::temp_dir().join(format!("siren-downloader-ffmpeg-test-{unique}"));
        let empty_dir = root.join("empty");
        let bin_dir = root.join("bin");
        create_dir_all(&empty_dir).expect("create empty PATH directory");
        create_dir_all(&bin_dir).expect("create FFmpeg PATH directory");

        let executable = if cfg!(target_os = "windows") {
            "ffmpeg.exe"
        } else {
            "ffmpeg"
        };
        let expected_path = bin_dir.join(executable);
        File::create(&expected_path).expect("create fake FFmpeg executable");
        let path = env::join_paths([&empty_dir, &bin_dir]).expect("build test PATH");

        assert_eq!(
            find_ffmpeg_in_paths(Some(path)),
            Some(expected_path.clone())
        );
        assert_eq!(
            find_ffmpeg_in_paths(Some(
                env::join_paths([&empty_dir]).expect("build empty PATH")
            )),
            None
        );

        remove_dir_all(root).expect("remove test directories");
    }
}
