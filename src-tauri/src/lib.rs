pub mod clipboard;
pub mod commands;
pub mod db;
pub mod models;
pub mod tray;

use std::sync::Arc;

use clipboard::{ClipboardMonitor, SystemClipboard};
use db::Database;
use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::GlobalShortcutExt;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
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

            // Setup system tray
            tray::setup_tray(app)?;

            // Start clipboard monitor
            let app_handle = app.handle().clone();
            let monitor_db = db.clone();
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
                        let new_clip = content.into_new_clip();
                        let hash = new_clip.content_hash.clone();
                        let db = monitor_db.clone();
                        let handle = app_handle.clone();
                        tauri::async_runtime::spawn(async move {
                            match db.find_by_hash(&hash).await {
                                Ok(Some(existing)) => {
                                    db.touch_clip(existing.id).await.ok();
                                }
                                Ok(None) => {
                                    if let Ok(clip) = db.insert_clip(new_clip).await {
                                        handle.emit("clipboard-changed", clip.to_dto()).ok();
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

            // Register global shortcut: Cmd+Shift+V to toggle window
            use tauri_plugin_global_shortcut::ShortcutState;
            app.global_shortcut().on_shortcut(
                "CmdOrCtrl+Shift+V",
                move |app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        tray::toggle_window(app);
                    }
                },
            )?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_clips,
            commands::search_clips,
            commands::delete_clip,
        ])
        .run(tauri::generate_context!())
        .expect("error while running ClipBin");
}
