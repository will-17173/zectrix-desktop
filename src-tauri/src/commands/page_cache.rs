#[tauri::command]
pub fn get_page_cache_list(
    state: tauri::State<'_, crate::state::AppState>,
    device_id: String,
) -> Result<Vec<crate::models::PageCacheRecord>, String> {
    state
        .get_page_cache_list(&device_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_page_cache(
    state: tauri::State<'_, crate::state::AppState>,
    device_id: String,
    page_id: u32,
) -> Result<(), String> {
    state
        .delete_page_cache(&device_id, page_id)
        .await
        .map_err(|e| e.to_string())
}