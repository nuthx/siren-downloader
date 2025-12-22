use anyhow::{Context, Result};
use reqwest::get;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::LazyLock;
use tauri::{AppHandle, Manager};
use tokio::fs;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

// 正在下载的封面集合，防止并发重复下载
static DOWNLOADING_COVERS: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

// 获取封面路径（本地已存在则直接返回，不存在则下载后返回）
pub async fn get_cover_path(app: &AppHandle, album_id: &str, cover_url: &str) -> Result<PathBuf> {
    let cover_dir = app
        .path()
        .app_data_dir()
        .context("无法访问应用目录")?
        .join("covers");
    let cover_path = cover_dir.join(format!("{}.jpg", album_id));

    // 如果已存在则直接返回
    if cover_path.exists() {
        return Ok(cover_path);
    }

    // 检查是否正在下载，如果是则等待
    {
        let mut downloading = DOWNLOADING_COVERS.lock().await;
        if downloading.contains(album_id) {
            drop(downloading);
            // 等待其他任务完成下载
            loop {
                sleep(Duration::from_millis(50)).await;
                if cover_path.exists() {
                    return Ok(cover_path);
                }
                let downloading = DOWNLOADING_COVERS.lock().await;
                if !downloading.contains(album_id) {
                    break;
                }
            }
            // 再次检查文件是否存在
            if cover_path.exists() {
                return Ok(cover_path);
            }
            // 重新获取锁并标记为正在下载
            DOWNLOADING_COVERS.lock().await.insert(album_id.to_string());
        } else {
            downloading.insert(album_id.to_string());
        }
    }

    // 确保下载完成后移除标记
    let result = async {
        // 创建封面目录
        fs::create_dir_all(&cover_dir)
            .await
            .context("无法创建封面目录")?;

        // 下载封面
        let bytes = get(cover_url)
            .await
            .context("下载封面失败")?
            .bytes()
            .await
            .context("解析封面失败")?;

        // 保存到本地
        fs::write(&cover_path, &bytes)
            .await
            .context("保存封面失败")?;

        println!("[Cover] 封面已保存: {}", album_id);
        Ok(cover_path.clone())
    }
    .await;

    // 移除下载标记
    DOWNLOADING_COVERS.lock().await.remove(album_id);

    result
}
