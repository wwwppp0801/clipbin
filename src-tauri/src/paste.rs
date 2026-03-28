use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

use crate::db::Database;
use crate::models::ContentType;

/// Flag to indicate the next clipboard change was caused by us (skip monitoring).
static SELF_TRIGGERED: AtomicBool = AtomicBool::new(false);

/// PID of the app that was frontmost before ClipBin opened.
#[cfg(target_os = "macos")]
static PREVIOUS_APP_PID: AtomicI32 = AtomicI32::new(0);

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

    SELF_TRIGGERED.store(true, Ordering::Relaxed);

    let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;

    match clip.content_type {
        ContentType::Text | ContentType::FilePath => {
            if let Some(text) = &clip.text_content {
                clipboard.set_text(text).map_err(|e| e.to_string())?;
            }
        }
        ContentType::Image => {
            if let Some(png_data) = &clip.image_data {
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

    db.touch_clip(id).await.ok();

    #[cfg(target_os = "macos")]
    simulate_paste();

    Ok(())
}

/// Remember the frontmost application before ClipBin takes focus.
#[cfg(target_os = "macos")]
pub fn remember_frontmost_app() {
    use objc::runtime::{Class, Object};
    use objc::{msg_send, sel, sel_impl};

    unsafe {
        let cls = Class::get("NSWorkspace").unwrap();
        let workspace: *mut Object = msg_send![cls, sharedWorkspace];
        let app: *mut Object = msg_send![workspace, frontmostApplication];
        if !app.is_null() {
            let pid: i32 = msg_send![app, processIdentifier];
            PREVIOUS_APP_PID.store(pid, Ordering::Relaxed);
        }
    }
}

/// Activate the previously frontmost application so paste goes to it.
#[cfg(target_os = "macos")]
pub fn activate_previous_app() {
    use objc::runtime::{Class, Object};
    use objc::{msg_send, sel, sel_impl};

    let pid = PREVIOUS_APP_PID.load(Ordering::Relaxed);
    if pid == 0 {
        return;
    }

    unsafe {
        let cls = Class::get("NSRunningApplication").unwrap();
        let app: *mut Object = msg_send![cls, runningApplicationWithProcessIdentifier: pid];
        if !app.is_null() {
            // NSApplicationActivateIgnoringOtherApps = 1 << 1 = 2
            let _: bool = msg_send![app, activateWithOptions: 2u64];
        }
    }
}

/// Simulate Cmd+V using Core Graphics events (same approach as Maccy).
#[cfg(target_os = "macos")]
fn simulate_paste() {
    use core_graphics::event::{CGEvent, CGEventFlags, CGEventTapLocation, CGKeyCode};
    use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

    const V_KEY: CGKeyCode = 9;

    let source = match CGEventSource::new(CGEventSourceStateID::CombinedSessionState) {
        Ok(s) => s,
        Err(_) => return,
    };

    let cmd_flag = CGEventFlags::CGEventFlagCommand;

    if let (Ok(down), Ok(up)) = (
        CGEvent::new_keyboard_event(source.clone(), V_KEY, true),
        CGEvent::new_keyboard_event(source, V_KEY, false),
    ) {
        down.set_flags(cmd_flag);
        up.set_flags(cmd_flag);
        down.post(CGEventTapLocation::Session);
        up.post(CGEventTapLocation::Session);
    }
}
