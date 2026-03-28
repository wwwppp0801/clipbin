use std::sync::atomic::{AtomicBool, Ordering};

use crate::db::Database;
use crate::models::ContentType;

/// Flag to indicate the next clipboard change was caused by us (skip monitoring).
static SELF_TRIGGERED: AtomicBool = AtomicBool::new(false);

/// Check and clear the self-triggered flag.
pub fn was_self_triggered() -> bool {
    SELF_TRIGGERED.swap(false, Ordering::Relaxed)
}

/// Write a clip's content to the system clipboard, then simulate Cmd+V.
pub async fn paste_clip(db: &Database, id: i64) -> Result<(), String> {
    let clip = db
        .get_clip_by_id(id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Clip not found")?;

    // Mark self-triggered so monitor skips the next clipboard change
    SELF_TRIGGERED.store(true, Ordering::Relaxed);

    // Write content to system clipboard
    let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;

    match clip.content_type {
        ContentType::Text | ContentType::FilePath => {
            if let Some(text) = &clip.text_content {
                clipboard.set_text(text).map_err(|e| e.to_string())?;
            }
        }
        ContentType::Image => {
            if let Some(png_data) = &clip.image_data {
                // Decode PNG back to RGBA for arboard
                let img = image::load_from_memory(png_data).map_err(|e| e.to_string())?;
                let rgba = img.to_rgba8();
                let (w, h) = rgba.dimensions();
                let arboard_img = arboard::ImageData {
                    width: w as usize,
                    height: h as usize,
                    bytes: std::borrow::Cow::Borrowed(rgba.as_raw()),
                };
                clipboard
                    .set_image(arboard_img)
                    .map_err(|e| e.to_string())?;
            }
        }
    }

    // Update last_used_at and use_count
    db.touch_clip(id).await.ok();

    // Simulate Cmd+V keypress to paste into the active app
    #[cfg(target_os = "macos")]
    simulate_paste();

    Ok(())
}

/// Simulate Cmd+V using Core Graphics events (macOS).
#[cfg(target_os = "macos")]
fn simulate_paste() {
    use core_graphics::event::{CGEvent, CGEventFlags, CGEventTapLocation, CGKeyCode};
    use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

    // Virtual key code for 'V' on macOS
    const V_KEY: CGKeyCode = 9;

    let source = match CGEventSource::new(CGEventSourceStateID::HIDSystemState) {
        Ok(s) => s,
        Err(_) => return,
    };

    // Key down
    if let Ok(event) = CGEvent::new_keyboard_event(source.clone(), V_KEY, true) {
        event.set_flags(CGEventFlags::CGEventFlagCommand);
        event.post(CGEventTapLocation::HID);
    }

    // Small delay between key down and up
    std::thread::sleep(std::time::Duration::from_millis(20));

    // Key up
    if let Ok(event) = CGEvent::new_keyboard_event(source, V_KEY, false) {
        event.set_flags(CGEventFlags::CGEventFlagCommand);
        event.post(CGEventTapLocation::HID);
    }
}
