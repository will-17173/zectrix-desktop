#[tauri::command]
pub fn load_bootstrap_state(
    state: tauri::State<'_, crate::state::AppState>,
) -> Result<crate::models::BootstrapState, String> {
    state.load_bootstrap_state().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_api_keys(
    state: tauri::State<'_, crate::state::AppState>,
) -> Result<Vec<crate::models::ApiKeyRecord>, String> {
    state.list_api_keys().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_api_key(
    state: tauri::State<'_, crate::state::AppState>,
    name: String,
    key: String,
) -> Result<crate::models::ApiKeyRecord, String> {
    state.add_api_key(&name, &key).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_api_key(
    state: tauri::State<'_, crate::state::AppState>,
    id: i64,
) -> Result<(), String> {
    state.remove_api_key(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn sync_all(
    state: tauri::State<'_, crate::state::AppState>,
) -> Result<crate::models::BootstrapState, String> {
    state.sync_all().await.map_err(|e| e.to_string())
}