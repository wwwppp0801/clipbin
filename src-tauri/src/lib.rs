pub mod commands;
pub mod clipboard;
pub mod db;
pub mod models;
pub mod tray;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            commands::get_clips,
            commands::search_clips,
            commands::delete_clip,
        ])
        .run(tauri::generate_context!())
        .expect("error while running ClipBin");
}
