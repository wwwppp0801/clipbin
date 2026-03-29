use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
    App, Emitter, Manager, WebviewWindow,
};

/// Timestamp (millis) of the last show/hide action.
static LAST_ACTION_TIME: AtomicU64 = AtomicU64::new(0);

/// When true, blur events are ignored (e.g. during context menu).
static BLUR_PAUSED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Grace period in ms — ignore all hotkey/blur during this window.
const GRACE_MS: u64 = 400;

pub fn set_blur_paused(paused: bool) {
    BLUR_PAUSED.store(paused, std::sync::atomic::Ordering::Relaxed);
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn is_in_grace_period() -> bool {
    now_millis().saturating_sub(LAST_ACTION_TIME.load(Ordering::Relaxed)) < GRACE_MS
}

fn mark_action() {
    LAST_ACTION_TIME.store(now_millis(), Ordering::Relaxed);
}

pub fn setup_tray(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    let toggle = MenuItemBuilder::with_id("toggle", "Show/Hide ClipBin").build(app)?;
    let quit = MenuItemBuilder::with_id("quit", "Quit ClipBin").build(app)?;
    let menu = MenuBuilder::new(app)
        .item(&toggle)
        .separator()
        .item(&quit)
        .build()?;

    // Load dedicated tray icon (22pt = 44px @2x, with padding)
    let tray_icon = {
        let bytes = include_bytes!("../icons/tray-icon.png");
        tauri::image::Image::from_bytes(bytes)?
    };

    TrayIconBuilder::new()
        .icon(tray_icon)
        .icon_as_template(true)
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "toggle" => toggle_window(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|_tray, _event| {
            // Menu handles all interactions now
        })
        .build(app)?;

    Ok(())
}

/// Auto-hide when window loses focus (user clicks outside).
pub fn setup_blur_hide(window: &WebviewWindow) {
    let win_handle = window.app_handle().clone();
    window.on_window_event(move |event| {
        if let tauri::WindowEvent::Focused(false) = event {
            if !is_in_grace_period() && !BLUR_PAUSED.load(std::sync::atomic::Ordering::Relaxed) {
                request_hide(&win_handle);
            }
        }
    });
}

pub fn toggle_window(app: &tauri::AppHandle) {
    if is_in_grace_period() {
        return;
    }

    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            request_hide(app);
        } else {
            show_window(app);
        }
    }
}

fn show_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        // Remember which app was active before we steal focus
        #[cfg(target_os = "macos")]
        crate::paste::remember_frontmost_app();

        mark_action();
        position_at_bottom(app);
        window.show().ok();
        window.set_focus().ok();
        app.emit("window-will-show", ()).ok();
    }
}

fn request_hide(app: &tauri::AppHandle) {
    mark_action();
    app.emit("window-will-hide", ()).ok();
}

/// Actually hide the window (called by frontend after exit animation)
pub fn do_hide(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        window.hide().ok();
    }
}

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
#[allow(deprecated)]
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
