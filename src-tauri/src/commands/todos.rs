#[tauri::command]
pub fn create_local_todo(
    state: tauri::State<'_, crate::state::AppState>,
    input: crate::models::TodoUpsertInput,
) -> Result<crate::models::TodoRecord, String> {
    state.create_local_todo(input).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn toggle_todo_status(
    state: tauri::State<'_, crate::state::AppState>,
    local_id: String,
) -> Result<crate::models::TodoRecord, String> {
    state
        .toggle_todo_status(&local_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_local_todo(
    state: tauri::State<'_, crate::state::AppState>,
    local_id: String,
) -> Result<(), String> {
    state
        .delete_local_todo(&local_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_local_todo(
    state: tauri::State<'_, crate::state::AppState>,
    local_id: String,
    input: crate::models::TodoUpsertInput,
) -> Result<crate::models::TodoRecord, String> {
    state
        .update_local_todo(&local_id, input)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn push_todo_to_device(
    state: tauri::State<'_, crate::state::AppState>,
    local_id: String,
    device_id: String,
) -> Result<(), String> {
    state
        .push_todo_to_device(&local_id, &device_id)
        .await
        .map_err(|e| e.to_string())
}
