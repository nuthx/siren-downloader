use crate::models::SongDetail;

// 修复文件名中的非法字符
pub fn fix_filename(name: &str) -> String {
    name.replace("  ", " ")
        .replace(['<', '>', ':'], " ")
        .replace(['\\', '/', '|'], " ")
        .replace('*', " ")
        .replace(['\'', '\"'], "")
        .replace('?', "")
        .trim()
        .to_string()
}

// 修复文件夹名称
// 除了修复非法字符外，还会移除末尾的点号（Windows不允许）
pub fn fix_folder_name(name: &str) -> String {
    fix_filename(name).trim_end_matches('.').to_string()
}

pub fn render_song_template(template: &str, song: &SongDetail) -> String {
    let year = song.publish.get(..4).unwrap_or("");
    let short_year = song.publish.get(2..4).unwrap_or("");
    let month = song.publish.get(5..7).unwrap_or("");
    let day = song.publish.get(8..10).unwrap_or("");
    let date = song.publish.get(..10).unwrap_or("");

    let mut rendered = String::with_capacity(template.len() + song.title.len());
    let mut remaining = template;

    while let Some(open) = remaining.find('{') {
        rendered.push_str(&remaining[..open]);
        let token_start = open + 1;
        let Some(close_offset) = remaining[token_start..].find('}') else {
            rendered.push_str(&remaining[open..]);
            return rendered;
        };
        let close = token_start + close_offset;
        let token = &remaining[token_start..close];
        let value = match token {
            "title" => Some(song.title.as_str()),
            "album" => Some(song.album_title.as_str()),
            "id" => Some(song.id.as_str()),
            "year" => Some(year),
            "date" => Some(date),
            "date:yyMMdd" => None,
            "month" => Some(month),
            "day" => Some(day),
            "track" => None,
            "track:02" => None,
            _ => None,
        };

        if let Some(value) = value {
            rendered.push_str(value);
        } else if token == "track" {
            rendered.push_str(&song.track.to_string());
        } else if token == "track:02" {
            rendered.push_str(&format!("{:02}", song.track));
        } else if token == "date:yyMMdd" {
            rendered.push_str(&format!("{}{}{}", short_year, month, day));
        } else {
            rendered.push('{');
            rendered.push_str(token);
            rendered.push('}');
        }

        remaining = &remaining[close + 1..];
    }

    rendered.push_str(remaining);
    rendered
}
