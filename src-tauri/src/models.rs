use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContentType {
    Text,
    Html,
    Image,
    FilePath,
}

impl ContentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ContentType::Text => "text",
            ContentType::Html => "html",
            ContentType::Image => "image",
            ContentType::FilePath => "file_path",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s {
            "html" => ContentType::Html,
            "image" => ContentType::Image,
            "file_path" => ContentType::FilePath,
            _ => ContentType::Text,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clip {
    pub id: i64,
    pub content_type: ContentType,
    pub text_content: Option<String>,
    pub image_data: Option<Vec<u8>>,
    pub content_hash: String,
    pub source_app: Option<String>,
    pub created_at: String,
    pub last_used_at: String,
    pub use_count: i64,
    pub is_pinned: bool,
}

#[derive(Debug, Clone)]
pub struct NewClip {
    pub content_type: ContentType,
    pub text_content: Option<String>,
    pub image_data: Option<Vec<u8>>,
    pub content_hash: String,
    pub source_app: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipDto {
    pub id: i64,
    pub content_type: String,
    pub text_content: Option<String>,
    pub image_preview: Option<String>,
    pub source_app: Option<String>,
    pub created_at: String,
    pub last_used_at: String,
    pub use_count: i64,
    pub is_pinned: bool,
}

impl Clip {
    pub fn to_dto(&self) -> ClipDto {
        let image_preview = self.image_data.as_ref().map(|data| {
            use base64::Engine;
            let b64 = base64::engine::general_purpose::STANDARD.encode(data);
            format!("data:image/png;base64,{}", b64)
        });

        ClipDto {
            id: self.id,
            content_type: self.content_type.as_str().to_string(),
            text_content: self.text_content.clone(),
            image_preview,
            source_app: self.source_app.clone(),
            created_at: self.created_at.clone(),
            last_used_at: self.last_used_at.clone(),
            use_count: self.use_count,
            is_pinned: self.is_pinned,
        }
    }
}
