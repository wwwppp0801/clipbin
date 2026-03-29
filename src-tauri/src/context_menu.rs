use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::Manager;

#[tauri::command]
pub async fn show_clip_context_menu(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    _clip_id: i64,
    is_pinned: bool,
) -> Result<Option<String>, String> {
    let paste_item = MenuItemBuilder::with_id("paste", "Paste")
        .build(&app)
        .map_err(|e| e.to_string())?;
    let paste_plain = MenuItemBuilder::with_id("paste_plain", "Paste as Plain Text")
        .build(&app)
        .map_err(|e| e.to_string())?;
    let copy_item = MenuItemBuilder::with_id("copy", "Copy to Clipboard")
        .build(&app)
        .map_err(|e| e.to_string())?;
    let pin_label = if is_pinned { "Unpin" } else { "Pin" };
    let pin_item = MenuItemBuilder::with_id("pin", pin_label)
        .build(&app)
        .map_err(|e| e.to_string())?;
    let delete_item = MenuItemBuilder::with_id("delete", "Delete")
        .build(&app)
        .map_err(|e| e.to_string())?;

    let menu = MenuBuilder::new(&app)
        .item(&paste_item)
        .item(&paste_plain)
        .item(&copy_item)
        .separator()
        .item(&pin_item)
        .separator()
        .item(&delete_item)
        .build()
        .map_err(|e| e.to_string())?;

    // Pause blur while native menu is open
    crate::tray::set_blur_paused(true);

    // popup_menu is synchronous — blocks until user picks or dismisses
    window.popup_menu(&menu).map_err(|e| e.to_string())?;

    // Resume blur after menu closes
    crate::tray::set_blur_paused(false);

    // We can't easily get the selected item from popup_menu in Tauri 2.
    // Instead, use on_menu_event on the window to handle actions.
    // Return None here — the frontend listens via the app-level menu event.
    Ok(None)
}

/// Setup menu event handler on the main window to dispatch context menu actions.
pub fn setup_menu_handler(app: &tauri::App) {
    use tauri::Emitter;
    if let Some(window) = app.get_webview_window("main") {
        window.on_menu_event(move |win, event| {
            let action = event.id().0.clone();
            win.emit("context-menu-action", action).ok();
        });
    }
}
