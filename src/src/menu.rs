use tauri::menu::{Menu, MenuBuilder, MenuItemBuilder, PredefinedMenuItem, SubmenuBuilder};
use tauri::{App, Emitter};

pub fn create_menu(app: &App) -> Result<Menu<tauri::Wry>, Box<dyn std::error::Error>> {
    let app_menu = SubmenuBuilder::new(app, "塞壬唱片下载器")
        .item(&MenuItemBuilder::with_id("about", "关于塞壬唱片下载器").build(app)?)
        .separator()
        .item(
            &MenuItemBuilder::with_id("settings", "设置")
                .accelerator("Cmd+,")
                .build(app)?,
        )
        .separator()
        .item(&PredefinedMenuItem::hide(app, Some("隐藏塞壬唱片下载器"))?)
        .item(&PredefinedMenuItem::hide_others(app, Some("隐藏其他"))?)
        .item(&PredefinedMenuItem::show_all(app, Some("显示全部"))?)
        .separator()
        .item(&PredefinedMenuItem::quit(app, Some("退出塞壬唱片下载器"))?)
        .build()?;

    let music_menu = SubmenuBuilder::new(app, "下载")
        .item(
            &MenuItemBuilder::with_id("refresh", "更新歌曲列表")
                .accelerator("Cmd+R")
                .build(app)?,
        )
        .separator()
        .item(&MenuItemBuilder::with_id("download_all", "下载所有歌曲").build(app)?)
        .build()?;

    let view_menu = SubmenuBuilder::new(app, "显示")
        .item(&PredefinedMenuItem::fullscreen(app, Some("进入全屏幕"))?)
        .build()?;

    let window_menu = SubmenuBuilder::new(app, "窗口")
        .item(&PredefinedMenuItem::minimize(app, Some("最小化"))?)
        .item(&PredefinedMenuItem::maximize(app, Some("最大化"))?)
        .separator()
        .item(&PredefinedMenuItem::close_window(app, Some("关闭窗口"))?)
        .build()?;

    let menu = MenuBuilder::new(app)
        .item(&app_menu)
        .item(&music_menu)
        .item(&view_menu)
        .item(&window_menu)
        .build()?;

    // 设置菜单事件监听
    app.on_menu_event(|app, event| match event.id().as_ref() {
        "about" => {
            let _ = app.emit("navigate", "/about");
        }
        "settings" => {
            let _ = app.emit("navigate", "/settings");
        }
        "refresh" => {
            let _ = app.emit("menu-action", "refresh");
        }
        "download_all" => {
            let _ = app.emit("menu-action", "download_all");
        }
        _ => {}
    });

    Ok(menu)
}
