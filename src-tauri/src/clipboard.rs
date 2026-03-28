use sha2::{Digest, Sha256};

use crate::models::{ContentType, NewClip};

pub trait ClipboardReader: Send {
    fn get_text(&mut self) -> Option<String>;
    fn get_image(&mut self) -> Option<Vec<u8>>;
}

pub struct SystemClipboard {
    clipboard: arboard::Clipboard,
}

impl SystemClipboard {
    pub fn new() -> Result<Self, arboard::Error> {
        Ok(Self {
            clipboard: arboard::Clipboard::new()?,
        })
    }
}

impl ClipboardReader for SystemClipboard {
    fn get_text(&mut self) -> Option<String> {
        self.clipboard.get_text().ok().filter(|t| !t.is_empty())
    }

    fn get_image(&mut self) -> Option<Vec<u8>> {
        let img = self.clipboard.get_image().ok()?;
        encode_rgba_to_png(img.width as u32, img.height as u32, &img.bytes)
    }
}

/// Encode raw RGBA pixel data to PNG bytes using the `image` crate.
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
}

pub enum ClipboardContent {
    Text(String),
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

    pub fn into_new_clip(self) -> NewClip {
        let hash = self.compute_hash();
        match self {
            ClipboardContent::Text(text) => NewClip {
                content_type: ContentType::Text,
                text_content: Some(text),
                image_data: None,
                content_hash: hash,
            },
            ClipboardContent::Image(data) => NewClip {
                content_type: ContentType::Image,
                text_content: None,
                image_data: Some(data),
                content_hash: hash,
            },
            ClipboardContent::FilePath(path) => NewClip {
                content_type: ContentType::FilePath,
                text_content: Some(path),
                image_data: None,
                content_hash: hash,
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
        }
    }

    pub fn check(&mut self) -> Option<ClipboardContent> {
        // Try text first
        if let Some(text) = self.reader.get_text() {
            let content = if is_file_path(&text) {
                ClipboardContent::FilePath(text)
            } else {
                ClipboardContent::Text(text)
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

fn is_file_path(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.is_empty() || trimmed.contains('\n') {
        return false;
    }
    // Check if it looks like an absolute path
    trimmed.starts_with('/') || trimmed.starts_with("~/")
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockClipboard {
        text: Option<String>,
        image: Option<Vec<u8>>,
    }

    unsafe impl Send for MockClipboard {}

    impl MockClipboard {
        fn new() -> Self {
            Self {
                text: None,
                image: None,
            }
        }

        fn set_text(&mut self, text: &str) {
            self.text = Some(text.to_string());
            self.image = None;
        }

        fn set_image(&mut self, data: Vec<u8>) {
            self.image = Some(data);
            self.text = None;
        }
    }

    impl ClipboardReader for MockClipboard {
        fn get_text(&mut self) -> Option<String> {
            self.text.clone()
        }

        fn get_image(&mut self) -> Option<Vec<u8>> {
            self.image.clone()
        }
    }

    #[test]
    fn test_compute_hash_text() {
        let content = ClipboardContent::Text("hello".to_string());
        let hash = content.compute_hash();
        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 64); // SHA-256 = 32 bytes = 64 hex chars

        // Same content produces same hash
        let content2 = ClipboardContent::Text("hello".to_string());
        assert_eq!(content.compute_hash(), content2.compute_hash());
    }

    #[test]
    fn test_compute_hash_different_types() {
        let text = ClipboardContent::Text("hello".to_string());
        let filepath = ClipboardContent::FilePath("hello".to_string());

        // Different types with same string produce different hashes
        assert_ne!(text.compute_hash(), filepath.compute_hash());
    }

    #[test]
    fn test_compute_hash_image() {
        let content = ClipboardContent::Image(vec![1, 2, 3, 4]);
        let hash = content.compute_hash();
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_is_file_path() {
        assert!(is_file_path("/Users/test/file.txt"));
        assert!(is_file_path("~/Documents/file.txt"));
        assert!(!is_file_path("hello world"));
        assert!(!is_file_path("http://example.com"));
        assert!(!is_file_path("/path/one\n/path/two"));
        assert!(!is_file_path(""));
        assert!(!is_file_path("  "));
    }

    #[test]
    fn test_into_new_clip_text() {
        let content = ClipboardContent::Text("hello".to_string());
        let clip = content.into_new_clip();
        assert_eq!(clip.content_type, ContentType::Text);
        assert_eq!(clip.text_content.as_deref(), Some("hello"));
        assert!(clip.image_data.is_none());
    }

    #[test]
    fn test_into_new_clip_image() {
        let content = ClipboardContent::Image(vec![1, 2, 3]);
        let clip = content.into_new_clip();
        assert_eq!(clip.content_type, ContentType::Image);
        assert!(clip.text_content.is_none());
        assert_eq!(clip.image_data, Some(vec![1, 2, 3]));
    }

    #[test]
    fn test_into_new_clip_filepath() {
        let content = ClipboardContent::FilePath("/tmp/test.txt".to_string());
        let clip = content.into_new_clip();
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

        // First check: new content
        assert!(monitor.check().is_some());
        // Second check: same content, should be None
        assert!(monitor.check().is_none());
    }

    #[test]
    fn test_monitor_detects_change() {
        let mut mock = MockClipboard::new();
        mock.set_text("hello");
        let mut monitor = ClipboardMonitor::new(mock);

        assert!(monitor.check().is_some());
        assert!(monitor.check().is_none());

        // Change the text
        monitor.reader.set_text("world");
        let result = monitor.check();
        assert!(result.is_some());
        match result.unwrap() {
            ClipboardContent::Text(t) => assert_eq!(t, "world"),
            _ => panic!("Expected text content"),
        }
    }

    #[test]
    fn test_monitor_detects_file_path() {
        let mut mock = MockClipboard::new();
        mock.set_text("/Users/test/file.txt");
        let mut monitor = ClipboardMonitor::new(mock);

        let result = monitor.check();
        assert!(result.is_some());
        match result.unwrap() {
            ClipboardContent::FilePath(p) => assert_eq!(p, "/Users/test/file.txt"),
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
}
