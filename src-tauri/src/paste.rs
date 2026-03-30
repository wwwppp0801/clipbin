use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

use crate::db::Database;
use crate::models::{ClipRepresentation, ContentType};

/// Flag to indicate the next clipboard change was caused by us (skip monitoring).
static SELF_TRIGGERED: AtomicBool = AtomicBool::new(false);

/// PID of the app that was frontmost before ClipBin opened.
#[cfg(target_os = "macos")]
static PREVIOUS_APP_PID: AtomicI32 = AtomicI32::new(0);

/// Check and clear the self-triggered flag.
pub fn was_self_triggered() -> bool {
    SELF_TRIGGERED.swap(false, Ordering::Relaxed)
}

/// Write clip content to system clipboard. If stored representations are available,
/// writes ALL of them for perfect restoration. Otherwise falls back to legacy behavior.
async fn write_clip_to_clipboard(db: &Database, clip_id: i64) -> Result<(), String> {
    let clip = db
        .get_clip_by_id(clip_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Clip not found")?;

    SELF_TRIGGERED.store(true, Ordering::Relaxed);

    // Try full restoration path first
    let representations = db
        .get_representations(clip_id)
        .await
        .map_err(|e| e.to_string())?;

    if !representations.is_empty() {
        #[cfg(target_os = "macos")]
        write_all_representations_to_pasteboard(&representations)?;
        #[cfg(not(target_os = "macos"))]
        {
            // Non-macOS fallback: write text only
            let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
            if let Some(text) = &clip.text_content {
                clipboard.set_text(text).map_err(|e| e.to_string())?;
            }
        }
    } else {
        // Legacy fallback for clips without stored representations
        let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
        match clip.content_type {
            ContentType::Text | ContentType::Html => {
                if let Some(text) = &clip.text_content {
                    clipboard.set_text(text).map_err(|e| e.to_string())?;
                }
            }
            ContentType::FilePath => {
                if let Some(text) = &clip.text_content {
                    #[cfg(target_os = "macos")]
                    write_file_urls_to_pasteboard(text)?;
                    #[cfg(not(target_os = "macos"))]
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
    }

    db.touch_clip(clip_id).await.ok();
    Ok(())
}

/// Copy a clip to system clipboard only (no paste simulation, no window hide).
pub async fn copy_clip_to_clipboard(db: &Database, id: i64) -> Result<(), String> {
    write_clip_to_clipboard(db, id).await
}

/// Write a clip's content to the system clipboard, then simulate Cmd+V.
pub async fn paste_clip(db: &Database, id: i64) -> Result<(), String> {
    write_clip_to_clipboard(db, id).await?;

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

/// Write ALL representations back to NSPasteboard for perfect clipboard restoration.
#[cfg(target_os = "macos")]
fn write_all_representations_to_pasteboard(
    representations: &[ClipRepresentation],
) -> Result<(), String> {
    use objc::runtime::{Class, Object};
    use objc::{msg_send, sel, sel_impl};

    if representations.is_empty() {
        return Err("No representations to write".to_string());
    }

    unsafe {
        let pb_cls = Class::get("NSPasteboard").ok_or("NSPasteboard not found")?;
        let pb: *mut Object = msg_send![pb_cls, generalPasteboard];
        let _: () = msg_send![pb, clearContents];

        let nsstring_cls = Class::get("NSString").ok_or("NSString not found")?;
        let nsdata_cls = Class::get("NSData").ok_or("NSData not found")?;

        // Build NSArray of all UTI type strings and declare them
        let arr_cls = Class::get("NSMutableArray").ok_or("NSMutableArray not found")?;
        let type_arr: *mut Object = msg_send![arr_cls, arrayWithCapacity: representations.len()];

        for rep in representations {
            let c_uti = std::ffi::CString::new(rep.uti.as_str()).map_err(|e| e.to_string())?;
            let ns_uti: *mut Object = msg_send![nsstring_cls, stringWithUTF8String: c_uti.as_ptr()];
            let _: () = msg_send![type_arr, addObject: ns_uti];
        }

        // declareTypes:owner: declares all types at once
        let _: isize = msg_send![pb, declareTypes: type_arr owner: std::ptr::null::<Object>()];

        // Now set data for each type
        for rep in representations {
            let c_uti = std::ffi::CString::new(rep.uti.as_str()).map_err(|e| e.to_string())?;
            let ns_uti: *mut Object = msg_send![nsstring_cls, stringWithUTF8String: c_uti.as_ptr()];
            let ns_data: *mut Object =
                msg_send![nsdata_cls, dataWithBytes: rep.data.as_ptr() length: rep.data.len()];
            let _: bool = msg_send![pb, setData: ns_data forType: ns_uti];
        }
    }

    Ok(())
}

/// Write file paths back to NSPasteboard as file URLs (so Finder can paste them).
#[cfg(target_os = "macos")]
fn write_file_urls_to_pasteboard(paths_text: &str) -> Result<(), String> {
    use objc::runtime::{Class, Object, BOOL, YES};
    use objc::{msg_send, sel, sel_impl};

    let paths: Vec<&str> = paths_text
        .lines()
        .filter(|l| !l.trim().is_empty())
        .collect();
    if paths.is_empty() {
        return Err("No file paths".to_string());
    }

    unsafe {
        let pb_cls = Class::get("NSPasteboard").ok_or("NSPasteboard not found")?;
        let pb: *mut Object = msg_send![pb_cls, generalPasteboard];
        let _: () = msg_send![pb, clearContents];

        // Build NSMutableArray of NSURL objects
        let arr_cls = Class::get("NSMutableArray").ok_or("NSMutableArray not found")?;
        let arr: *mut Object = msg_send![arr_cls, arrayWithCapacity: paths.len()];

        let nsurl_cls = Class::get("NSURL").ok_or("NSURL not found")?;
        let nsstring_cls = Class::get("NSString").ok_or("NSString not found")?;

        for path in &paths {
            let c_path = std::ffi::CString::new(*path).map_err(|e| e.to_string())?;
            let ns_path: *mut Object =
                msg_send![nsstring_cls, stringWithUTF8String: c_path.as_ptr()];
            let url: *mut Object = msg_send![nsurl_cls, fileURLWithPath: ns_path];
            if !url.is_null() {
                let _: () = msg_send![arr, addObject: url];
            }
        }

        let result: BOOL = msg_send![pb, writeObjects: arr];
        if result != YES {
            return Err("Failed to write file URLs to pasteboard".to_string());
        }
    }

    Ok(())
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
