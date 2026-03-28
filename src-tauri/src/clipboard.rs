use sha2::{Digest, Sha256};

use crate::models::{ContentType, NewClip};

pub trait ClipboardReader: Send {
    fn has_changed(&mut self) -> bool;
    fn get_text(&mut self) -> Option<String>;
    fn get_html(&mut self) -> Option<String>;
    fn get_image(&mut self) -> Option<Vec<u8>>;
    fn get_file_urls(&mut self) -> Option<Vec<String>>;
}

pub struct SystemClipboard {
    clipboard: arboard::Clipboard,
    last_change_count: i64,
}

impl SystemClipboard {
    pub fn new() -> Result<Self, arboard::Error> {
        Ok(Self {
            clipboard: arboard::Clipboard::new()?,
            last_change_count: -1,
        })
    }
}

impl ClipboardReader for SystemClipboard {
    fn has_changed(&mut self) -> bool {
        #[cfg(target_os = "macos")]
        {
            let count = get_pasteboard_change_count();
            if count != self.last_change_count {
                self.last_change_count = count;
                true
            } else {
                false
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            true // Always check on non-macOS
        }
    }

    fn get_text(&mut self) -> Option<String> {
        self.clipboard.get_text().ok().filter(|t| !t.is_empty())
    }

    fn get_html(&mut self) -> Option<String> {
        #[cfg(target_os = "macos")]
        {
            read_html_from_pasteboard()
        }
        #[cfg(not(target_os = "macos"))]
        {
            None
        }
    }

    fn get_image(&mut self) -> Option<Vec<u8>> {
        let img = self.clipboard.get_image().ok()?;
        encode_rgba_to_png(img.width as u32, img.height as u32, &img.bytes)
    }

    fn get_file_urls(&mut self) -> Option<Vec<String>> {
        #[cfg(target_os = "macos")]
        {
            read_file_urls_from_pasteboard()
        }
        #[cfg(not(target_os = "macos"))]
        {
            None
        }
    }
}

/// Get NSPasteboard.changeCount to detect clipboard changes efficiently.
#[cfg(target_os = "macos")]
fn get_pasteboard_change_count() -> i64 {
    use objc::runtime::{Class, Object};
    use objc::{msg_send, sel, sel_impl};

    unsafe {
        let cls = match Class::get("NSPasteboard") {
            Some(c) => c,
            None => return -1,
        };
        let pb: *mut Object = msg_send![cls, generalPasteboard];
        if pb.is_null() {
            return -1;
        }
        msg_send![pb, changeCount]
    }
}

/// Read HTML content from NSPasteboard.
#[cfg(target_os = "macos")]
fn read_html_from_pasteboard() -> Option<String> {
    use objc::runtime::{Class, Object};
    use objc::{msg_send, sel, sel_impl};

    unsafe {
        let pb_cls = Class::get("NSPasteboard")?;
        let pb: *mut Object = msg_send![pb_cls, generalPasteboard];
        if pb.is_null() {
            return None;
        }

        // Check for "public.html" type
        let nsstring_cls = Class::get("NSString")?;
        let html_type_str = std::ffi::CString::new("public.html").ok()?;
        let html_type: *mut Object =
            msg_send![nsstring_cls, stringWithUTF8String: html_type_str.as_ptr()];
        let html_data: *mut Object = msg_send![pb, stringForType: html_type];

        if html_data.is_null() {
            return None;
        }

        let cstr: *const std::os::raw::c_char = msg_send![html_data, UTF8String];
        if cstr.is_null() {
            return None;
        }

        let s = std::ffi::CStr::from_ptr(cstr).to_string_lossy().to_string();
        if s.is_empty() {
            None
        } else {
            Some(s)
        }
    }
}

/// Read file URLs from NSPasteboard using Objective-C FFI.
#[cfg(target_os = "macos")]
fn read_file_urls_from_pasteboard() -> Option<Vec<String>> {
    use objc::runtime::{Class, Object};
    use objc::{msg_send, sel, sel_impl};

    unsafe {
        let cls = Class::get("NSPasteboard")?;
        let pb: *mut Object = msg_send![cls, generalPasteboard];
        if pb.is_null() {
            return None;
        }

        // Check if pasteboard has file URLs
        let nsurl_cls = Class::get("NSURL")?;
        let class_array: *mut Object =
            msg_send![Class::get("NSArray")?, arrayWithObject: nsurl_cls];
        let urls: *mut Object =
            msg_send![pb, readObjectsForClasses: class_array options: std::ptr::null::<Object>()];

        if urls.is_null() {
            return None;
        }

        let count: usize = msg_send![urls, count];
        if count == 0 {
            return None;
        }

        let mut paths = Vec::new();
        for i in 0..count {
            let url: *mut Object = msg_send![urls, objectAtIndex: i];
            if url.is_null() {
                continue;
            }
            let path: *mut Object = msg_send![url, path];
            if path.is_null() {
                continue;
            }
            let cstr: *const std::os::raw::c_char = msg_send![path, UTF8String];
            if !cstr.is_null() {
                let s = std::ffi::CStr::from_ptr(cstr).to_string_lossy().to_string();
                if !s.is_empty() {
                    paths.push(s);
                }
            }
        }

        if paths.is_empty() {
            None
        } else {
            Some(paths)
        }
    }
}

/// Get the name of the currently frontmost application.
#[cfg(target_os = "macos")]
pub fn get_frontmost_app_name() -> Option<String> {
    use objc::runtime::{Class, Object};
    use objc::{msg_send, sel, sel_impl};

    unsafe {
        let cls = Class::get("NSWorkspace")?;
        let workspace: *mut Object = msg_send![cls, sharedWorkspace];
        let app: *mut Object = msg_send![workspace, frontmostApplication];
        if app.is_null() {
            return None;
        }
        let name: *mut Object = msg_send![app, localizedName];
        if name.is_null() {
            return None;
        }
        let cstr: *const std::os::raw::c_char = msg_send![name, UTF8String];
        if cstr.is_null() {
            return None;
        }
        Some(std::ffi::CStr::from_ptr(cstr).to_string_lossy().to_string())
    }
}

#[cfg(not(target_os = "macos"))]
pub fn get_frontmost_app_name() -> Option<String> {
    None
}

fn encode_rgba_to_png(width: u32, height: u32, rgba: &[u8]) -> Option<Vec<u8>> {
    use image::{ImageBuffer, RgbaImage};
    use std::io::Cursor;

    let img: RgbaImage = ImageBuffer::from_raw(width, height, rgba.to_vec())?;
    let mut buf = Cursor::new(Vec::new());
    img.write_to(&mut buf, image::ImageFormat::Png).ok()?;
    Some(buf.into_inner())
}

pub struct ClipboardMonitor<R: ClipboardReader> {
    reader: R,
    last_text_hash: Option<String>,
    last_image_hash: Option<String>,
    last_file_hash: Option<String>,
}

pub enum ClipboardContent {
    Text(String),
    Html { html: String, plain: String },
    Image(Vec<u8>),
    FilePath(String),
}

impl ClipboardContent {
    pub fn compute_hash(&self) -> String {
        let mut hasher = Sha256::new();
        match self {
            ClipboardContent::Text(t) => {
                hasher.update(b"text:");
                hasher.update(t.as_bytes());
            }
            ClipboardContent::Html { html, .. } => {
                hasher.update(b"html:");
                hasher.update(html.as_bytes());
            }
            ClipboardContent::Image(data) => {
                hasher.update(b"image:");
                hasher.update(data);
            }
            ClipboardContent::FilePath(p) => {
                hasher.update(b"filepath:");
                hasher.update(p.as_bytes());
            }
        }
        hex::encode(hasher.finalize())
    }

    pub fn into_new_clip(self, source_app: Option<String>) -> NewClip {
        let hash = self.compute_hash();
        match self {
            ClipboardContent::Text(text) => NewClip {
                content_type: ContentType::Text,
                text_content: Some(text),
                image_data: None,
                content_hash: hash,
                source_app,
            },
            ClipboardContent::Html { plain, .. } => NewClip {
                content_type: ContentType::Html,
                text_content: Some(plain),
                image_data: None,
                content_hash: hash,
                source_app,
            },
            ClipboardContent::Image(data) => NewClip {
                content_type: ContentType::Image,
                text_content: None,
                image_data: Some(data),
                content_hash: hash,
                source_app,
            },
            ClipboardContent::FilePath(path) => NewClip {
                content_type: ContentType::FilePath,
                text_content: Some(path),
                image_data: None,
                content_hash: hash,
                source_app,
            },
        }
    }
}

impl<R: ClipboardReader> ClipboardMonitor<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            last_text_hash: None,
            last_image_hash: None,
            last_file_hash: None,
        }
    }

    pub fn check(&mut self) -> Option<ClipboardContent> {
        // Fast check: skip if clipboard hasn't changed (via NSPasteboard.changeCount)
        if !self.reader.has_changed() {
            return None;
        }

        // Priority: file URLs > text > image
        // Check file URLs first (Finder copy)
        if let Some(paths) = self.reader.get_file_urls() {
            let joined = paths.join("\n");
            let content = ClipboardContent::FilePath(joined);
            let hash = content.compute_hash();
            if self.last_file_hash.as_ref() != Some(&hash) {
                self.last_file_hash = Some(hash);
                return Some(content);
            }
            return None;
        }

        // Try text (check for HTML enrichment)
        if let Some(text) = self.reader.get_text() {
            let content = if let Some(html) = self.reader.get_html() {
                ClipboardContent::Html { html, plain: text }
            } else {
                classify_text(text)
            };
            let hash = content.compute_hash();
            if self.last_text_hash.as_ref() != Some(&hash) {
                self.last_text_hash = Some(hash);
                return Some(content);
            }
            return None;
        }

        // Try image
        if let Some(image_data) = self.reader.get_image() {
            let content = ClipboardContent::Image(image_data);
            let hash = content.compute_hash();
            if self.last_image_hash.as_ref() != Some(&hash) {
                self.last_image_hash = Some(hash);
                return Some(content);
            }
        }

        None
    }
}

/// Classify text content — check if it's a file path that exists on disk.
fn classify_text(text: String) -> ClipboardContent {
    let trimmed = text.trim();
    // Single line, looks like an absolute path, and actually exists
    if !trimmed.contains('\n') && (trimmed.starts_with('/') || trimmed.starts_with("~/")) {
        let expanded = if trimmed.starts_with("~/") {
            if let Ok(home) = std::env::var("HOME") {
                trimmed.replacen("~", &home, 1)
            } else {
                trimmed.to_string()
            }
        } else {
            trimmed.to_string()
        };
        if std::path::Path::new(&expanded).exists() {
            return ClipboardContent::FilePath(text);
        }
    }
    ClipboardContent::Text(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockClipboard {
        text: Option<String>,
        html: Option<String>,
        image: Option<Vec<u8>>,
        file_urls: Option<Vec<String>>,
    }

    unsafe impl Send for MockClipboard {}

    impl MockClipboard {
        fn new() -> Self {
            Self {
                text: None,
                html: None,
                image: None,
                file_urls: None,
            }
        }

        fn set_text(&mut self, text: &str) {
            self.text = Some(text.to_string());
            self.html = None;
            self.image = None;
            self.file_urls = None;
        }

        fn set_html(&mut self, text: &str, html: &str) {
            self.text = Some(text.to_string());
            self.html = Some(html.to_string());
            self.image = None;
            self.file_urls = None;
        }

        fn set_image(&mut self, data: Vec<u8>) {
            self.image = Some(data);
            self.text = None;
            self.html = None;
            self.file_urls = None;
        }

        fn set_file_urls(&mut self, urls: Vec<String>) {
            self.file_urls = Some(urls);
            self.text = None;
            self.html = None;
            self.image = None;
        }
    }

    impl ClipboardReader for MockClipboard {
        fn has_changed(&mut self) -> bool {
            true // Always changed in tests
        }

        fn get_text(&mut self) -> Option<String> {
            self.text.clone()
        }

        fn get_html(&mut self) -> Option<String> {
            self.html.clone()
        }

        fn get_image(&mut self) -> Option<Vec<u8>> {
            self.image.clone()
        }

        fn get_file_urls(&mut self) -> Option<Vec<String>> {
            self.file_urls.clone()
        }
    }

    #[test]
    fn test_compute_hash_text() {
        let content = ClipboardContent::Text("hello".to_string());
        let hash = content.compute_hash();
        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 64);

        let content2 = ClipboardContent::Text("hello".to_string());
        assert_eq!(content.compute_hash(), content2.compute_hash());
    }

    #[test]
    fn test_compute_hash_different_types() {
        let text = ClipboardContent::Text("hello".to_string());
        let filepath = ClipboardContent::FilePath("hello".to_string());
        assert_ne!(text.compute_hash(), filepath.compute_hash());
    }

    #[test]
    fn test_compute_hash_image() {
        let content = ClipboardContent::Image(vec![1, 2, 3, 4]);
        let hash = content.compute_hash();
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_classify_text_regular() {
        let content = classify_text("hello world".to_string());
        match content {
            ClipboardContent::Text(t) => assert_eq!(t, "hello world"),
            _ => panic!("Expected text"),
        }
    }

    #[test]
    fn test_classify_text_nonexistent_path() {
        // Path that doesn't exist should be classified as text
        let content = classify_text("/nonexistent/path/abc123".to_string());
        match content {
            ClipboardContent::Text(_) => {}
            _ => panic!("Expected text for nonexistent path"),
        }
    }

    #[test]
    fn test_classify_text_existing_path() {
        // /tmp always exists
        let content = classify_text("/tmp".to_string());
        match content {
            ClipboardContent::FilePath(p) => assert_eq!(p, "/tmp"),
            _ => panic!("Expected file path for /tmp"),
        }
    }

    #[test]
    fn test_into_new_clip_text() {
        let content = ClipboardContent::Text("hello".to_string());
        let clip = content.into_new_clip(None);
        assert_eq!(clip.content_type, ContentType::Text);
        assert_eq!(clip.text_content.as_deref(), Some("hello"));
        assert!(clip.image_data.is_none());
    }

    #[test]
    fn test_into_new_clip_image() {
        let content = ClipboardContent::Image(vec![1, 2, 3]);
        let clip = content.into_new_clip(None);
        assert_eq!(clip.content_type, ContentType::Image);
        assert!(clip.text_content.is_none());
        assert_eq!(clip.image_data, Some(vec![1, 2, 3]));
    }

    #[test]
    fn test_into_new_clip_filepath() {
        let content = ClipboardContent::FilePath("/tmp/test.txt".to_string());
        let clip = content.into_new_clip(None);
        assert_eq!(clip.content_type, ContentType::FilePath);
        assert_eq!(clip.text_content.as_deref(), Some("/tmp/test.txt"));
    }

    #[test]
    fn test_monitor_detects_new_text() {
        let mut mock = MockClipboard::new();
        mock.set_text("hello");
        let mut monitor = ClipboardMonitor::new(mock);

        let result = monitor.check();
        assert!(result.is_some());
        match result.unwrap() {
            ClipboardContent::Text(t) => assert_eq!(t, "hello"),
            _ => panic!("Expected text content"),
        }
    }

    #[test]
    fn test_monitor_ignores_duplicate() {
        let mut mock = MockClipboard::new();
        mock.set_text("hello");
        let mut monitor = ClipboardMonitor::new(mock);

        assert!(monitor.check().is_some());
        assert!(monitor.check().is_none());
    }

    #[test]
    fn test_monitor_detects_change() {
        let mut mock = MockClipboard::new();
        mock.set_text("hello");
        let mut monitor = ClipboardMonitor::new(mock);

        assert!(monitor.check().is_some());
        assert!(monitor.check().is_none());

        monitor.reader.set_text("world");
        let result = monitor.check();
        assert!(result.is_some());
        match result.unwrap() {
            ClipboardContent::Text(t) => assert_eq!(t, "world"),
            _ => panic!("Expected text content"),
        }
    }

    #[test]
    fn test_monitor_detects_file_urls() {
        let mut mock = MockClipboard::new();
        mock.set_file_urls(vec![
            "/Users/test/file.txt".to_string(),
            "/Users/test/other.txt".to_string(),
        ]);
        let mut monitor = ClipboardMonitor::new(mock);

        let result = monitor.check();
        assert!(result.is_some());
        match result.unwrap() {
            ClipboardContent::FilePath(p) => {
                assert!(p.contains("/Users/test/file.txt"));
                assert!(p.contains("/Users/test/other.txt"));
            }
            _ => panic!("Expected file path content"),
        }
    }

    #[test]
    fn test_monitor_detects_image() {
        let mut mock = MockClipboard::new();
        mock.set_image(vec![10, 20, 30]);
        let mut monitor = ClipboardMonitor::new(mock);

        let result = monitor.check();
        assert!(result.is_some());
        match result.unwrap() {
            ClipboardContent::Image(data) => assert_eq!(data, vec![10, 20, 30]),
            _ => panic!("Expected image content"),
        }
    }

    #[test]
    fn test_monitor_detects_html() {
        let mut mock = MockClipboard::new();
        mock.set_html("Hello World", "<b>Hello World</b>");
        let mut monitor = ClipboardMonitor::new(mock);

        let result = monitor.check();
        assert!(result.is_some());
        match result.unwrap() {
            ClipboardContent::Html { html, plain } => {
                assert_eq!(html, "<b>Hello World</b>");
                assert_eq!(plain, "Hello World");
            }
            _ => panic!("Expected HTML content"),
        }
    }

    #[test]
    fn test_html_into_new_clip() {
        let content = ClipboardContent::Html {
            html: "<p>test</p>".to_string(),
            plain: "test".to_string(),
        };
        let clip = content.into_new_clip(None);
        assert_eq!(clip.content_type, ContentType::Html);
        assert_eq!(clip.text_content.as_deref(), Some("test"));
    }

    #[test]
    fn test_monitor_skips_when_no_change() {
        struct NoChangeMock;
        unsafe impl Send for NoChangeMock {}
        impl ClipboardReader for NoChangeMock {
            fn has_changed(&mut self) -> bool {
                false
            }
            fn get_text(&mut self) -> Option<String> {
                Some("should not see this".to_string())
            }
            fn get_html(&mut self) -> Option<String> {
                None
            }
            fn get_image(&mut self) -> Option<Vec<u8>> {
                None
            }
            fn get_file_urls(&mut self) -> Option<Vec<String>> {
                None
            }
        }

        let mut monitor = ClipboardMonitor::new(NoChangeMock);
        assert!(monitor.check().is_none());
    }
}
