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
    ignored_apps: Option<Vec<String>>,
) -> Result<(), String> {
    let mut settings = state.lock().await;
    let old_hotkey = settings.hotkey.clone();
    settings.hotkey = hotkey.clone();
    settings.max_clips = max_clips;
    if let Some(apps) = ignored_apps {
        settings.ignored_apps = apps;
    }

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

// --- Collections ---

#[tauri::command]
pub async fn create_collection(
    state: State<'_, Arc<Database>>,
    name: String,
) -> Result<i64, String> {
    state
        .create_collection(&name)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_collections(
    state: State<'_, Arc<Database>>,
) -> Result<Vec<(i64, String)>, String> {
    state.list_collections().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_collection(state: State<'_, Arc<Database>>, id: i64) -> Result<(), String> {
    state.delete_collection(id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_to_collection(
    state: State<'_, Arc<Database>>,
    clip_id: i64,
    collection_id: i64,
) -> Result<(), String> {
    state
        .add_clip_to_collection(clip_id, collection_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_collection_clips(
    state: State<'_, Arc<Database>>,
    collection_id: i64,
    limit: Option<i64>,
) -> Result<Vec<ClipDto>, String> {
    let limit = limit.unwrap_or(50);
    state
        .get_collection_clips(collection_id, limit)
        .await
        .map(|clips| clips.iter().map(|c| c.to_dto()).collect())
        .map_err(|e| e.to_string())
}

// --- Export/Import ---

#[tauri::command]
pub async fn export_history(state: State<'_, Arc<Database>>) -> Result<String, String> {
    let clips = state.export_clips().await.map_err(|e| e.to_string())?;
    let dtos: Vec<_> = clips.iter().map(|c| c.to_dto()).collect();
    serde_json::to_string_pretty(&dtos).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn import_history(state: State<'_, Arc<Database>>, json: String) -> Result<u32, String> {
    let items: Vec<crate::models::ClipDto> =
        serde_json::from_str(&json).map_err(|e| e.to_string())?;
    let mut count = 0u32;
    for item in items {
        let content_type = crate::models::ContentType::parse(&item.content_type);
        if let Some(text) = &item.text_content {
            let new_clip = crate::models::NewClip {
                content_type,
                text_content: Some(text.clone()),
                image_data: None,
                content_hash: {
                    use sha2::Digest;
                    let mut hasher = sha2::Sha256::new();
                    hasher.update(text.as_bytes());
                    format!("import_{}", hex::encode(hasher.finalize()))
                },
                source_app: None,
            };
            // Skip if already exists
            if state
                .find_by_hash(&new_clip.content_hash)
                .await
                .ok()
                .flatten()
                .is_none()
            {
                state.insert_clip(new_clip).await.ok();
                count += 1;
            }
        }
    }
    Ok(count)
}

#[tauri::command]
pub async fn toggle_pin(state: State<'_, Arc<Database>>, id: i64) -> Result<bool, String> {
    state.toggle_pin(id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn clear_history(state: State<'_, Arc<Database>>) -> Result<(), String> {
    state.clear_unpinned().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn copy_clip(state: State<'_, Arc<Database>>, id: i64) -> Result<(), String> {
    crate::paste::copy_clip_to_clipboard(&state, id).await
}

#[tauri::command]
pub async fn paste_clip(
    app: tauri::AppHandle,
    state: State<'_, Arc<Database>>,
    id: i64,
) -> Result<(), String> {
    // 1. Hide our window
    crate::tray::do_hide(&app);

    // 2. Activate the app that was focused before ClipBin opened
    #[cfg(target_os = "macos")]
    crate::paste::activate_previous_app();

    // 3. Wait for the previous app to regain focus
    tokio::time::sleep(std::time::Duration::from_millis(150)).await;

    // 4. Write to clipboard + simulate Cmd+V
    crate::paste::paste_clip(&state, id).await
}

#[tauri::command]
pub async fn set_blur_paused(paused: bool) -> Result<(), String> {
    crate::tray::set_blur_paused(paused);
    Ok(())
}

#[tauri::command]
pub async fn do_hide_window(app: tauri::AppHandle) -> Result<(), String> {
    crate::tray::do_hide(&app);
    Ok(())
}

#[tauri::command]
pub async fn open_settings_window(app: tauri::AppHandle) -> Result<(), String> {
    crate::tray::open_settings(&app)
}

#[tauri::command]
pub async fn close_settings_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("settings") {
        win.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}
