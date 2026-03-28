use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{TrayIconBuilder, TrayIconEvent},
    App, Manager, WebviewWindow,
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

/// Auto-hide when window loses focus (user clicks outside)
pub fn setup_blur_hide(window: &WebviewWindow) {
    let win = window.clone();
    window.on_window_event(move |event| {
        if let tauri::WindowEvent::Focused(false) = event {
            win.hide().ok();
        }
    });
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

/// Position the window at the bottom of the visible screen area (above Dock)
fn position_at_bottom(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let scale = window
            .current_monitor()
            .ok()
            .flatten()
            .map(|m| m.scale_factor())
            .unwrap_or(2.0);

        let screen_total_height = window
            .current_monitor()
            .ok()
            .flatten()
            .map(|m| m.size().height as f64 / scale)
            .unwrap_or(900.0);

        #[cfg(target_os = "macos")]
        let (vis_x, vis_y, vis_w, _vis_h) = macos_visible_frame();

        #[cfg(not(target_os = "macos"))]
        let (vis_x, vis_y, vis_w, _vis_h) = {
            let w = window
                .current_monitor()
                .ok()
                .flatten()
                .map(|m| m.size().width as f64 / scale)
                .unwrap_or(1440.0);
            (0.0, 0.0, w, screen_total_height)
        };

        let padding: f64 = 12.0;
        let panel_height: f64 = 260.0;
        let panel_width: f64 = vis_w - (padding * 2.0);

        // macOS: vis_y is distance from screen bottom to bottom of visible area (above Dock)
        // Convert to Tauri top-left coords
        let tauri_y = screen_total_height - vis_y - panel_height - padding;
        let tauri_x = vis_x + padding;

        window
            .set_size(tauri::LogicalSize::new(panel_width, panel_height))
            .ok();
        window
            .set_position(tauri::LogicalPosition::new(tauri_x, tauri_y))
            .ok();
    }
}

#[cfg(target_os = "macos")]
fn macos_visible_frame() -> (f64, f64, f64, f64) {
    use cocoa::appkit::NSScreen;
    use cocoa::base::nil;

    unsafe {
        let screen = NSScreen::mainScreen(nil);
        if screen != nil {
            let frame = NSScreen::visibleFrame(screen);
            return (
                frame.origin.x,
                frame.origin.y,
                frame.size.width,
                frame.size.height,
            );
        }
    }
    (0.0, 0.0, 1440.0, 900.0)
}
