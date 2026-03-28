use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::path::Path;

use crate::models::{Clip, ContentType, NewClip};

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new(db_path: &Path) -> Result<Self, sqlx::Error> {
        let url = format!("sqlite:{}?mode=rwc", db_path.display());
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&url)
            .await?;

        let db = Self { pool };
        db.run_migrations().await?;
        Ok(db)
    }

    pub async fn new_in_memory() -> Result<Self, sqlx::Error> {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await?;

        let db = Self { pool };
        db.run_migrations().await?;
        Ok(db)
    }

    async fn run_migrations(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS clips (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                content_type TEXT NOT NULL CHECK(content_type IN ('text', 'html', 'image', 'file_path')),
                text_content TEXT,
                image_data BLOB,
                content_hash TEXT NOT NULL,
                source_app TEXT,
                created_at TEXT NOT NULL,
                last_used_at TEXT NOT NULL,
                use_count INTEGER NOT NULL DEFAULT 1,
                is_pinned INTEGER NOT NULL DEFAULT 0
            )",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE VIRTUAL TABLE IF NOT EXISTS clips_fts USING fts5(
                text_content,
                content='clips',
                content_rowid='id'
            )",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TRIGGER IF NOT EXISTS clips_ai AFTER INSERT ON clips BEGIN
                INSERT INTO clips_fts(rowid, text_content) VALUES (new.id, new.text_content);
            END",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TRIGGER IF NOT EXISTS clips_ad AFTER DELETE ON clips BEGIN
                INSERT INTO clips_fts(clips_fts, rowid, text_content) VALUES('delete', old.id, old.text_content);
            END"
        )
        .execute(&self.pool)
        .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_clips_content_hash ON clips(content_hash)")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn insert_clip(&self, clip: NewClip) -> Result<Clip, sqlx::Error> {
        let now = chrono::Utc::now().to_rfc3339();
        let content_type_str = clip.content_type.as_str();

        let id = sqlx::query_scalar::<_, i64>(
            "INSERT INTO clips (content_type, text_content, image_data, content_hash, created_at, last_used_at)
             VALUES (?, ?, ?, ?, ?, ?)
             RETURNING id"
        )
        .bind(content_type_str)
        .bind(&clip.text_content)
        .bind(&clip.image_data)
        .bind(&clip.content_hash)
        .bind(&now)
        .bind(&now)
        .fetch_one(&self.pool)
        .await?;

        Ok(Clip {
            id,
            content_type: clip.content_type,
            text_content: clip.text_content,
            image_data: clip.image_data,
            content_hash: clip.content_hash,
            source_app: None,
            created_at: now.clone(),
            last_used_at: now,
            use_count: 1,
            is_pinned: false,
        })
    }

    pub async fn find_by_hash(&self, hash: &str) -> Result<Option<Clip>, sqlx::Error> {
        let row = sqlx::query_as::<_, ClipRow>(
            "SELECT id, content_type, text_content, image_data, content_hash, source_app,
                    created_at, last_used_at, use_count, is_pinned
             FROM clips WHERE content_hash = ? LIMIT 1",
        )
        .bind(hash)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into_clip()))
    }

    pub async fn get_clips(&self, limit: i64, offset: i64) -> Result<Vec<Clip>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ClipRow>(
            "SELECT id, content_type, text_content, image_data, content_hash, source_app,
                    created_at, last_used_at, use_count, is_pinned
             FROM clips ORDER BY is_pinned DESC, last_used_at DESC LIMIT ? OFFSET ?",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into_clip()).collect())
    }

    pub async fn search_clips(&self, query: &str, limit: i64) -> Result<Vec<Clip>, sqlx::Error> {
        let search_query = format!("{}*", query);
        let rows = sqlx::query_as::<_, ClipRow>(
            "SELECT c.id, c.content_type, c.text_content, c.image_data, c.content_hash,
                    c.source_app, c.created_at, c.last_used_at, c.use_count, c.is_pinned
             FROM clips c
             INNER JOIN clips_fts f ON c.id = f.rowid
             WHERE clips_fts MATCH ?
             ORDER BY c.last_used_at DESC
             LIMIT ?",
        )
        .bind(&search_query)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into_clip()).collect())
    }

    pub async fn delete_clip(&self, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM clips WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn clear_unpinned(&self) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM clips WHERE is_pinned = 0")
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn toggle_pin(&self, id: i64) -> Result<bool, sqlx::Error> {
        let new_val = sqlx::query_scalar::<_, bool>(
            "UPDATE clips SET is_pinned = NOT is_pinned WHERE id = ? RETURNING is_pinned",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;
        Ok(new_val)
    }

    /// Delete oldest clips (by last_used_at) that exceed the max limit.
    /// Pinned clips are never deleted.
    pub async fn enforce_limit(&self, max_clips: i64) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            "DELETE FROM clips WHERE id IN (
                SELECT id FROM clips WHERE is_pinned = 0
                ORDER BY last_used_at DESC
                LIMIT -1 OFFSET ?
            )",
        )
        .bind(max_clips)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected())
    }

    pub async fn touch_clip(&self, id: i64) -> Result<(), sqlx::Error> {
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query("UPDATE clips SET last_used_at = ?, use_count = use_count + 1 WHERE id = ?")
            .bind(&now)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn touch_clip_by_hash(&self, hash: &str) -> Result<(), sqlx::Error> {
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            "UPDATE clips SET last_used_at = ?, use_count = use_count + 1 WHERE content_hash = ?",
        )
        .bind(&now)
        .bind(hash)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_clip_by_id(&self, id: i64) -> Result<Option<Clip>, sqlx::Error> {
        let row = sqlx::query_as::<_, ClipRow>(
            "SELECT id, content_type, text_content, image_data, content_hash, source_app,
                    created_at, last_used_at, use_count, is_pinned
             FROM clips WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into_clip()))
    }
}

#[derive(sqlx::FromRow)]
struct ClipRow {
    id: i64,
    content_type: String,
    text_content: Option<String>,
    image_data: Option<Vec<u8>>,
    content_hash: String,
    source_app: Option<String>,
    created_at: String,
    last_used_at: String,
    use_count: i64,
    is_pinned: bool,
}

impl ClipRow {
    fn into_clip(self) -> Clip {
        Clip {
            id: self.id,
            content_type: ContentType::parse(&self.content_type),
            text_content: self.text_content,
            image_data: self.image_data,
            content_hash: self.content_hash,
            source_app: self.source_app,
            created_at: self.created_at,
            last_used_at: self.last_used_at,
            use_count: self.use_count,
            is_pinned: self.is_pinned,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ContentType;

    async fn setup_db() -> Database {
        Database::new_in_memory().await.unwrap()
    }

    fn make_text_clip(text: &str, hash: &str) -> NewClip {
        NewClip {
            content_type: ContentType::Text,
            text_content: Some(text.to_string()),
            image_data: None,
            content_hash: hash.to_string(),
        }
    }

    #[tokio::test]
    async fn test_insert_and_retrieve() {
        let db = setup_db().await;
        let clip = db
            .insert_clip(make_text_clip("hello world", "hash1"))
            .await
            .unwrap();

        assert_eq!(clip.text_content.as_deref(), Some("hello world"));
        assert_eq!(clip.content_hash, "hash1");
        assert_eq!(clip.use_count, 1);

        let found = db.get_clip_by_id(clip.id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().text_content.as_deref(), Some("hello world"));
    }

    #[tokio::test]
    async fn test_find_by_hash() {
        let db = setup_db().await;
        db.insert_clip(make_text_clip("test", "unique_hash"))
            .await
            .unwrap();

        let found = db.find_by_hash("unique_hash").await.unwrap();
        assert!(found.is_some());

        let not_found = db.find_by_hash("nonexistent").await.unwrap();
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn test_get_clips_ordered_by_last_used() {
        let db = setup_db().await;
        db.insert_clip(make_text_clip("first", "h1")).await.unwrap();
        db.insert_clip(make_text_clip("second", "h2"))
            .await
            .unwrap();
        db.insert_clip(make_text_clip("third", "h3")).await.unwrap();

        let clips = db.get_clips(10, 0).await.unwrap();
        assert_eq!(clips.len(), 3);
        // Most recent first
        assert_eq!(clips[0].text_content.as_deref(), Some("third"));
    }

    #[tokio::test]
    async fn test_full_text_search() {
        let db = setup_db().await;
        db.insert_clip(make_text_clip("hello world", "h1"))
            .await
            .unwrap();
        db.insert_clip(make_text_clip("goodbye world", "h2"))
            .await
            .unwrap();
        db.insert_clip(make_text_clip("rust programming", "h3"))
            .await
            .unwrap();

        let results = db.search_clips("hello", 10).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].text_content.as_deref(), Some("hello world"));

        let results = db.search_clips("world", 10).await.unwrap();
        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn test_delete_clip() {
        let db = setup_db().await;
        let clip = db
            .insert_clip(make_text_clip("to delete", "hd"))
            .await
            .unwrap();

        db.delete_clip(clip.id).await.unwrap();
        let found = db.get_clip_by_id(clip.id).await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_touch_updates_use_count() {
        let db = setup_db().await;
        let clip = db
            .insert_clip(make_text_clip("touchme", "ht"))
            .await
            .unwrap();
        assert_eq!(clip.use_count, 1);

        db.touch_clip(clip.id).await.unwrap();
        let updated = db.get_clip_by_id(clip.id).await.unwrap().unwrap();
        assert_eq!(updated.use_count, 2);

        db.touch_clip(clip.id).await.unwrap();
        let updated = db.get_clip_by_id(clip.id).await.unwrap().unwrap();
        assert_eq!(updated.use_count, 3);
    }

    #[tokio::test]
    async fn test_pagination() {
        let db = setup_db().await;
        for i in 0..5 {
            db.insert_clip(make_text_clip(&format!("item {}", i), &format!("hp{}", i)))
                .await
                .unwrap();
        }

        let page1 = db.get_clips(2, 0).await.unwrap();
        assert_eq!(page1.len(), 2);

        let page2 = db.get_clips(2, 2).await.unwrap();
        assert_eq!(page2.len(), 2);

        let page3 = db.get_clips(2, 4).await.unwrap();
        assert_eq!(page3.len(), 1);
    }

    #[tokio::test]
    async fn test_touch_by_hash() {
        let db = setup_db().await;
        let clip = db
            .insert_clip(make_text_clip("byhash", "hash_touch"))
            .await
            .unwrap();

        db.touch_clip_by_hash("hash_touch").await.unwrap();
        let updated = db.get_clip_by_id(clip.id).await.unwrap().unwrap();
        assert_eq!(updated.use_count, 2);
    }

    #[tokio::test]
    async fn test_enforce_limit() {
        let db = setup_db().await;
        for i in 0..10 {
            db.insert_clip(make_text_clip(&format!("clip {}", i), &format!("lim{}", i)))
                .await
                .unwrap();
        }

        let all = db.get_clips(100, 0).await.unwrap();
        assert_eq!(all.len(), 10);

        // Enforce limit of 5
        let deleted = db.enforce_limit(5).await.unwrap();
        assert_eq!(deleted, 5);

        let remaining = db.get_clips(100, 0).await.unwrap();
        assert_eq!(remaining.len(), 5);
    }

    #[tokio::test]
    async fn test_toggle_pin() {
        let db = setup_db().await;
        let clip = db
            .insert_clip(make_text_clip("pin me", "hpin"))
            .await
            .unwrap();
        assert!(!clip.is_pinned);

        let pinned = db.toggle_pin(clip.id).await.unwrap();
        assert!(pinned);

        let updated = db.get_clip_by_id(clip.id).await.unwrap().unwrap();
        assert!(updated.is_pinned);

        let unpinned = db.toggle_pin(clip.id).await.unwrap();
        assert!(!unpinned);
    }

    #[tokio::test]
    async fn test_clear_unpinned() {
        let db = setup_db().await;
        let clip1 = db
            .insert_clip(make_text_clip("normal", "hc1"))
            .await
            .unwrap();
        let clip2 = db
            .insert_clip(make_text_clip("pinned", "hc2"))
            .await
            .unwrap();
        db.toggle_pin(clip2.id).await.unwrap();
        db.insert_clip(make_text_clip("normal2", "hc3"))
            .await
            .unwrap();

        db.clear_unpinned().await.unwrap();

        let remaining = db.get_clips(100, 0).await.unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].id, clip2.id);
        assert!(remaining[0].is_pinned);
        // clip1 should be gone
        assert!(db.get_clip_by_id(clip1.id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_pinned_clips_sorted_first() {
        let db = setup_db().await;
        db.insert_clip(make_text_clip("old", "hs1")).await.unwrap();
        let clip2 = db
            .insert_clip(make_text_clip("pinned", "hs2"))
            .await
            .unwrap();
        db.insert_clip(make_text_clip("newest", "hs3"))
            .await
            .unwrap();

        // Pin the middle clip
        db.toggle_pin(clip2.id).await.unwrap();

        let clips = db.get_clips(10, 0).await.unwrap();
        // Pinned should be first
        assert_eq!(clips[0].id, clip2.id);
        assert!(clips[0].is_pinned);
    }
}
