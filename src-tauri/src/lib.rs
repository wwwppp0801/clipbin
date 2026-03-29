pub mod clipboard;
pub mod commands;
pub mod context_menu;
pub mod db;
pub mod models;
pub mod paste;
pub mod screenshot;
pub mod settings;
pub mod tray;

use std::sync::Arc;

use clipboard::{ClipboardMonitor, SystemClipboard};
use db::Database;
use settings::Settings;
use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::GlobalShortcutExt;
use tokio::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .setup(|app| {
            // Hide dock icon on macOS — run as menu bar only app
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            // Initialize database
            let app_data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_data_dir)?;
            let db_path = app_data_dir.join("clipbin.db");
            let db = tauri::async_runtime::block_on(Database::new(&db_path))
                .map_err(|e| anyhow::anyhow!("Failed to init database: {}", e))?;
            let db = Arc::new(db);
            app.manage(db.clone());

            // Load settings
            let settings = Settings::load(&app_data_dir);
            let hotkey = settings.hotkey.clone();
            let settings = Arc::new(Mutex::new(settings));
            app.manage(settings.clone());

            // Initialize screenshot state
            app.manage(screenshot::ScreenshotData(std::sync::Mutex::new(None)));

            // Setup system tray
            tray::setup_tray(app)?;

            // Auto-hide when window loses focus (user clicks outside)
            if let Some(window) = app.get_webview_window("main") {
                tray::setup_blur_hide(&window);
            }

            // Setup native context menu event handler
            context_menu::setup_menu_handler(app);

            // Start clipboard monitor
            let app_handle = app.handle().clone();
            let monitor_db = db.clone();
            let monitor_settings = settings.clone();
            std::thread::spawn(move || {
                let reader = match SystemClipboard::new() {
                    Ok(r) => r,
                    Err(e) => {
                        log::error!("Failed to init clipboard: {}", e);
                        return;
                    }
                };
                let mut monitor = ClipboardMonitor::new(reader);
                loop {
                    if let Some(content) = monitor.check() {
                        // Skip if this change was triggered by our own paste action
                        if paste::was_self_triggered() {
                            continue;
                        }
                        let source = clipboard::get_frontmost_app_name();

                        // Skip clips from ignored apps (e.g., password managers)
                        if let Some(ref app_name) = source {
                            if let Ok(settings) = monitor_settings.try_lock() {
                                if settings.is_ignored_app(app_name) {
                                    continue;
                                }
                            }
                        }

                        let new_clip = content.into_new_clip(source);
                        let hash = new_clip.content_hash.clone();
                        let db = monitor_db.clone();
                        let handle = app_handle.clone();
                        let settings_ref = monitor_settings.clone();
                        tauri::async_runtime::spawn(async move {
                            match db.find_by_hash(&hash).await {
                                Ok(Some(existing)) => {
                                    db.touch_clip(existing.id).await.ok();
                                }
                                Ok(None) => {
                                    if let Ok(clip) = db.insert_clip(new_clip).await {
                                        handle.emit("clipboard-changed", clip.to_dto()).ok();
                                        // Enforce max clips limit
                                        let max = settings_ref.lock().await.max_clips;
                                        db.enforce_limit(max).await.ok();
                                    }
                                }
                                Err(e) => {
                                    log::error!("DB error: {}", e);
                                }
                            }
                        });
                    }
                    std::thread::sleep(std::time::Duration::from_millis(500));
                }
            });

            // Register global shortcut from settings
            use tauri_plugin_global_shortcut::ShortcutState;
            app.global_shortcut()
                .on_shortcut(hotkey.as_str(), move |app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        tray::toggle_window(app);
                    }
                })?;

            // Register screenshot shortcut: Cmd+Shift+A
            app.global_shortcut().on_shortcut(
                "CmdOrCtrl+Shift+A",
                move |app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        let handle = app.clone();
                        std::thread::spawn(move || {
                            // Run screencapture and wait for it to finish
                            let status = std::process::Command::new("screencapture")
                                .args(["-i", "-c"])
                                .status();
                            // If user completed the screenshot (didn't cancel)
                            if let Ok(s) = status {
                                if s.success() {
                                    // Open editor window
                                    tauri::async_runtime::spawn(async move {
                                        screenshot::open_editor(&handle).await.ok();
                                    });
                                }
                            }
                        });
                    }
                },
            )?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_clips,
            commands::search_clips,
            commands::delete_clip,
            commands::copy_clip,
            commands::paste_clip,
            commands::create_collection,
            commands::list_collections,
            commands::delete_collection,
            commands::add_to_collection,
            commands::get_collection_clips,
            commands::export_history,
            commands::import_history,
            commands::toggle_pin,
            commands::clear_history,
            commands::get_settings,
            commands::save_settings,
            context_menu::show_clip_context_menu,
            commands::set_blur_paused,
            commands::do_hide_window,
            screenshot::get_screenshot_data,
            screenshot::save_screenshot,
            screenshot::copy_screenshot_to_clipboard,
            screenshot::close_editor,
        ])
        .run(tauri::generate_context!())
        .expect("error while running ClipBin");
}
