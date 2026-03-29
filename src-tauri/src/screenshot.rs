use tauri::Manager;

/// Read the current clipboard image and open a new editor window.
pub async fn open_editor(app: &tauri::AppHandle) -> Result<(), String> {
    // Small delay to ensure clipboard is populated
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    // Read image from clipboard
    let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    let img = clipboard.get_image().map_err(|e| e.to_string())?;

    // Encode as PNG base64
    let png_data =
        crate::clipboard::encode_rgba_to_png(img.width as u32, img.height as u32, &img.bytes)
            .ok_or("Failed to encode image")?;

    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&png_data);

    // Update screenshot data in app state
    if let Some(state) = app.try_state::<ScreenshotData>() {
        *state.0.lock().unwrap() = Some(b64);
    }

    // Close existing editor window if any
    if let Some(existing) = app.get_webview_window("screenshot-editor") {
        existing.close().ok();
        // Small delay for cleanup
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    // Create editor window
    let _editor = tauri::WebviewWindowBuilder::new(
        app,
        "screenshot-editor",
        tauri::WebviewUrl::App("/screenshot-editor.html".into()),
    )
    .title("Screenshot Editor")
    .inner_size(
        std::cmp::min(img.width, 1200) as f64,
        std::cmp::min(img.height, 800) as f64 + 60.0,
    )
    .resizable(true)
    .center()
    .build()
    .map_err(|e| e.to_string())?;

    Ok(())
}

pub struct ScreenshotData(pub std::sync::Mutex<Option<String>>);

#[tauri::command]
pub fn get_screenshot_data(state: tauri::State<'_, ScreenshotData>) -> Result<String, String> {
    state
        .0
        .lock()
        .map_err(|e| e.to_string())?
        .clone()
        .ok_or("No screenshot data".to_string())
}

#[tauri::command]
pub async fn save_screenshot(data: String, path: String) -> Result<(), String> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&data)
        .map_err(|e| e.to_string())?;
    std::fs::write(&path, &bytes).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn copy_screenshot_to_clipboard(data: String) -> Result<(), String> {
    use base64::Engine;
    let png_bytes = base64::engine::general_purpose::STANDARD
        .decode(&data)
        .map_err(|e| e.to_string())?;

    let img = image::load_from_memory(&png_bytes).map_err(|e| e.to_string())?;
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();

    let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    let arboard_img = arboard::ImageData {
        width: w as usize,
        height: h as usize,
        bytes: std::borrow::Cow::Borrowed(rgba.as_raw()),
    };
    clipboard
        .set_image(arboard_img)
        .map_err(|e| e.to_string())?;

    Ok(())
}
