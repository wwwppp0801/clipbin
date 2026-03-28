use std::sync::Arc;
use tauri::State;

use crate::db::Database;
use crate::models::ClipDto;

#[tauri::command]
pub async fn get_clips(
    state: State<'_, Arc<Database>>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<ClipDto>, String> {
    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);
    state
        .get_clips(limit, offset)
        .await
        .map(|clips| clips.iter().map(|c| c.to_dto()).collect())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn search_clips(
    state: State<'_, Arc<Database>>,
    query: String,
    limit: Option<i64>,
) -> Result<Vec<ClipDto>, String> {
    let limit = limit.unwrap_or(50);
    state
        .search_clips(&query, limit)
        .await
        .map(|clips| clips.iter().map(|c| c.to_dto()).collect())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_clip(
    state: State<'_, Arc<Database>>,
    id: i64,
) -> Result<(), String> {
    state.delete_clip(id).await.map_err(|e| e.to_string())
}
