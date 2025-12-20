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
