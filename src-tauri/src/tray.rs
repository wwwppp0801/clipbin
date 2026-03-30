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
/// Extended to 600ms to handle multi-monitor focus transition delays.
const GRACE_MS: u64 = 600;

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

/// Public version of mark_action for use by context_menu.
pub fn mark_action_public() {
    mark_action();
}

pub fn setup_tray(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    let toggle = MenuItemBuilder::with_id("toggle", "Show/Hide ClipBin").build(app)?;
    let settings = MenuItemBuilder::with_id("settings", "Settings...").build(app)?;
    let quit = MenuItemBuilder::with_id("quit", "Quit ClipBin").build(app)?;
    let menu = MenuBuilder::new(app)
        .item(&toggle)
        .item(&settings)
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
            "settings" => {
                open_settings(app).ok();
            }
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

        // Mark action FIRST so grace period covers the entire show sequence
        mark_action();
        position_at_bottom(app);
        window.show().ok();
        window.set_focus().ok();
        // Re-mark after show to reset the grace period timer
        mark_action();
        app.emit("window-will-show", ()).ok();

        // On multi-monitor setups, the window may lose focus during repositioning.
        // Re-focus after a short delay and re-mark grace period.
        // Skip if blur is paused (e.g. context menu is open).
        let handle = app.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(150));
            if BLUR_PAUSED.load(std::sync::atomic::Ordering::Relaxed) {
                return;
            }
            mark_action();
            if let Some(w) = handle.get_webview_window("main") {
                if w.is_visible().unwrap_or(false) {
                    w.set_focus().ok();
                    mark_action();
                }
            }
        });
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
        #[cfg(target_os = "macos")]
        let (tauri_x, tauri_y, panel_width) = {
            // macOS uses bottom-left origin; Tauri uses top-left origin of primary screen.
            // visibleFrame.origin.y is the bottom edge of usable area (above Dock).
            // Convert: tauri_y = primary_height - (vis_y + vis_h) gives the top-left Y
            // of the visible area's top edge. We want the panel at the bottom of visible area.
            let geo = macos_screen_at_cursor();
            let padding: f64 = 12.0;
            let panel_height: f64 = 260.0;
            let panel_w = geo.vis_w - (padding * 2.0);

            // Top of visible area in Tauri coords (top-left origin)
            let vis_top_y = geo.primary_height - (geo.vis_y + geo.vis_h);
            // Bottom of visible area in Tauri coords
            let vis_bottom_y = geo.primary_height - geo.vis_y;
            // Place panel at bottom with padding
            let y = vis_bottom_y - panel_height - padding;
            // Clamp: don't go above visible area top
            let y = y.max(vis_top_y + padding);

            let x = geo.vis_x + padding;
            (x, y, panel_w)
        };

        #[cfg(not(target_os = "macos"))]
        let (tauri_x, tauri_y, panel_width) = {
            let scale = window
                .current_monitor()
                .ok()
                .flatten()
                .map(|m| m.scale_factor())
                .unwrap_or(2.0);
            let h = window
                .current_monitor()
                .ok()
                .flatten()
                .map(|m| m.size().height as f64 / scale)
                .unwrap_or(900.0);
            let w = window
                .current_monitor()
                .ok()
                .flatten()
                .map(|m| m.size().width as f64 / scale)
                .unwrap_or(1440.0);
            let padding: f64 = 12.0;
            let panel_height: f64 = 260.0;
            (padding, h - panel_height - padding, w - padding * 2.0)
        };

        let panel_height: f64 = 260.0;
        window
            .set_size(tauri::LogicalSize::new(panel_width, panel_height))
            .ok();
        window
            .set_position(tauri::LogicalPosition::new(tauri_x, tauri_y))
            .ok();
    }
}

/// Open the settings window (or focus it if already open).
pub fn open_settings(app: &tauri::AppHandle) -> Result<(), String> {
    // If already open, just focus it
    if let Some(win) = app.get_webview_window("settings") {
        win.set_focus().map_err(|e| e.to_string())?;
        return Ok(());
    }

    let win = tauri::WebviewWindowBuilder::new(
        app,
        "settings",
        tauri::WebviewUrl::App("/settings.html".into()),
    )
    .title("ClipBin Settings")
    .inner_size(400.0, 560.0)
    .resizable(false)
    .center()
    .decorations(false)
    .build()
    .map_err(|e| e.to_string())?;

    win.set_focus().ok();
    Ok(())
}

/// Screen geometry for positioning the panel.
/// All values in macOS logical points (bottom-left origin coordinate system).
#[cfg(target_os = "macos")]
struct ScreenGeometry {
    /// Primary screen total height (for bottom-left → top-left conversion)
    primary_height: f64,
    /// Visible frame (excluding Dock/menu bar)
    vis_x: f64,
    vis_y: f64,
    vis_w: f64,
    vis_h: f64,
}

/// Find the screen containing the mouse cursor and return its geometry.
#[cfg(target_os = "macos")]
#[allow(deprecated)]
fn macos_screen_at_cursor() -> ScreenGeometry {
    use cocoa::appkit::NSScreen;
    use cocoa::base::nil;
    use cocoa::foundation::NSArray;
    use core_graphics::event::CGEvent;
    use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

    unsafe {
        // Primary screen height (needed to convert coordinate systems).
        // NOTE: NSScreen::mainScreen returns the screen with keyboard focus, NOT the
        // primary display. screens()[0] is always the primary display.
        let screens = NSScreen::screens(nil);
        let count = NSArray::count(screens) as usize;
        let primary_h = if count > 0 {
            let first: *mut objc::runtime::Object =
                cocoa::foundation::NSArray::objectAtIndex(screens, 0);
            if first != nil {
                NSScreen::frame(first).size.height
            } else {
                900.0
            }
        } else {
            900.0
        };

        // Get cursor position (CGEvent uses top-left origin of primary screen)
        let cursor_pos =
            if let Ok(source) = CGEventSource::new(CGEventSourceStateID::HIDSystemState) {
                let event = CGEvent::new(source);
                event.map(|e| e.location()).unwrap_or_default()
            } else {
                core_graphics::geometry::CGPoint::new(0.0, 0.0)
            };

        // Convert cursor from top-left to bottom-left coords
        let cursor_bl_y = primary_h - cursor_pos.y;

        for i in 0..count {
            let screen: *mut objc::runtime::Object =
                cocoa::foundation::NSArray::objectAtIndex(screens, i as u64);
            if screen == nil {
                continue;
            }
            let full = NSScreen::frame(screen);

            if cursor_pos.x >= full.origin.x
                && cursor_pos.x < full.origin.x + full.size.width
                && cursor_bl_y >= full.origin.y
                && cursor_bl_y < full.origin.y + full.size.height
            {
                let vis = NSScreen::visibleFrame(screen);
                return ScreenGeometry {
                    primary_height: primary_h,
                    vis_x: vis.origin.x,
                    vis_y: vis.origin.y,
                    vis_w: vis.size.width,
                    vis_h: vis.size.height,
                };
            }
        }

        // Fallback to first screen (primary display)
        if count > 0 {
            let first: *mut objc::runtime::Object =
                cocoa::foundation::NSArray::objectAtIndex(screens, 0);
            if first != nil {
                let vis = NSScreen::visibleFrame(first);
                return ScreenGeometry {
                    primary_height: primary_h,
                    vis_x: vis.origin.x,
                    vis_y: vis.origin.y,
                    vis_w: vis.size.width,
                    vis_h: vis.size.height,
                };
            }
        }
    }
    ScreenGeometry {
        primary_height: 900.0,
        vis_x: 0.0,
        vis_y: 0.0,
        vis_w: 1440.0,
        vis_h: 900.0,
    }
}
