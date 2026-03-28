use std::sync::Arc;
use tauri::{Manager, State};
use tokio::sync::Mutex;

use crate::db::Database;
use crate::models::ClipDto;
use crate::settings::Settings;

#[tauri::command]
pub async fn get_clips(
    state: State<'_, Arc<Database>>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<ClipDto>, String> {
    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);
    state
        .get_clips(limit, offset)
        .await
        .map(|clips| clips.iter().map(|c| c.to_dto()).collect())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn search_clips(
    state: State<'_, Arc<Database>>,
    query: String,
    limit: Option<i64>,
) -> Result<Vec<ClipDto>, String> {
    let limit = limit.unwrap_or(50);
    state
        .search_clips(&query, limit)
        .await
        .map(|clips| clips.iter().map(|c| c.to_dto()).collect())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_clip(state: State<'_, Arc<Database>>, id: i64) -> Result<(), String> {
    state.delete_clip(id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_settings(state: State<'_, Arc<Mutex<Settings>>>) -> Result<Settings, String> {
    let settings = state.lock().await;
    Ok(settings.clone())
}

#[tauri::command]
pub async fn save_settings(
    app: tauri::AppHandle,
    state: State<'_, Arc<Mutex<Settings>>>,
    hotkey: String,
    max_clips: i64,
) -> Result<(), String> {
    let mut settings = state.lock().await;
    let old_hotkey = settings.hotkey.clone();
    settings.hotkey = hotkey.clone();
    settings.max_clips = max_clips;

    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    settings.save(&app_data_dir)?;

    // Re-register hotkey if changed
    if old_hotkey != hotkey {
        use tauri_plugin_global_shortcut::GlobalShortcutExt;
        let gs = app.global_shortcut();
        // Unregister old
        if let Ok(shortcut) = old_hotkey.parse::<tauri_plugin_global_shortcut::Shortcut>() {
            gs.unregister(shortcut).ok();
        }
        // Register new
        if let Ok(shortcut) = hotkey.parse::<tauri_plugin_global_shortcut::Shortcut>() {
            use tauri_plugin_global_shortcut::ShortcutState;
            gs.on_shortcut(shortcut, move |app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    crate::tray::toggle_window(app);
                }
            })
            .map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn do_hide_window(app: tauri::AppHandle) -> Result<(), String> {
    crate::tray::do_hide(&app);
    Ok(())
}

#[tauri::command]
pub async fn animation_done() -> Result<(), String> {
    crate::tray::animation_done();
    Ok(())
}
