#[tauri::command]
pub async fn add_device_cache(
    state: tauri::State<'_, crate::state::AppState>,
    device_id: String,
    api_key_id: i64,
) -> Result<crate::models::DeviceRecord, String> {
    state
        .validate_and_cache_device(&device_id, api_key_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_device_cache(
    state: tauri::State<'_, crate::state::AppState>,
    device_id: String,
) -> Result<(), String> {
    state
        .remove_device_cache(&device_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_device_cache(
    state: tauri::State<'_, crate::state::AppState>,
) -> Result<Vec<crate::models::DeviceRecord>, String> {
    state.list_device_cache().map_err(|e| e.to_string())
}