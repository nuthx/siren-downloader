use serde::{Deserialize, Serialize};
use serde_json::Value;

// 应用配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub download_path: String,
    pub concurrent_downloads: usize,
    pub download_instrumental: bool,
    pub download_lyrics: bool,
    pub id3_date_format: String,
    pub custom_album: String,
    pub show_cover: bool,
    #[serde(default = "default_audio_format")]
    pub audio_format: String,
    #[serde(default = "default_album_folder_template")]
    pub album_folder_template: String,
    #[serde(default = "default_file_name_template")]
    pub file_name_template: String,
}

fn default_audio_format() -> String {
    "flac".to_string()
}

fn default_album_folder_template() -> String {
    "{album}".to_string()
}

fn default_file_name_template() -> String {
    "{title}".to_string()
}

// 歌曲详情
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SongDetail {
    pub id: String,
    pub title: String,
    pub album_id: String,
    pub album_title: String,
    pub cover_siren: String,
    pub cover_ncm: String,
    pub track: u8,
    pub publish: String,
    pub instrumental: bool,
    pub download: bool,
}

impl SongDetail {
    pub fn from_siren_api(item: &Value) -> Option<Self> {
        Some(Self {
            id: item.get("cid")?.as_str()?.to_string(),
            title: item.get("name")?.as_str()?.to_string(),
            album_id: item.get("albumCid")?.as_str()?.to_string(),
            instrumental: item
                .get("name")?
                .as_str()?
                .to_lowercase()
                .contains("instrument"),
            ..Default::default()
        })
    }
}
