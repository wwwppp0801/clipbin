use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{TrayIconBuilder, TrayIconEvent},
    App, Manager,
};

pub fn setup_tray(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    let toggle = MenuItemBuilder::with_id("toggle", "Show/Hide ClipBin").build(app)?;
    let quit = MenuItemBuilder::with_id("quit", "Quit ClipBin").build(app)?;
    let menu = MenuBuilder::new(app)
        .item(&toggle)
        .separator()
        .item(&quit)
        .build()?;

    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "toggle" => toggle_window(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { .. } = event {
                toggle_window(tray.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}

pub fn toggle_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            window.hide().ok();
        } else {
            position_at_bottom(app);
            window.show().ok();
            window.set_focus().ok();
        }
    }
}

/// Position the window at the bottom center of the screen, like Paste app
fn position_at_bottom(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if let Ok(Some(monitor)) = window.current_monitor() {
            let screen_size = monitor.size();
            let screen_pos = monitor.position();
            let scale = monitor.scale_factor();

            // Window spans full screen width with padding
            let padding: f64 = 20.0;
            let panel_height: f64 = 260.0;
            let panel_width: f64 = (screen_size.width as f64 / scale) - (padding * 2.0);

            let x = screen_pos.x as f64 + padding;
            let y =
                screen_pos.y as f64 + (screen_size.height as f64 / scale) - panel_height - padding;

            window
                .set_size(tauri::LogicalSize::new(panel_width, panel_height))
                .ok();
            window.set_position(tauri::LogicalPosition::new(x, y)).ok();
        }
    }
}
