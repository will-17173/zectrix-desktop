#[tauri::command]
pub fn create_text_template(
    state: tauri::State<'_, crate::state::AppState>,
    input: crate::models::TextTemplateInput,
) -> Result<crate::models::TextTemplateRecord, String> {
    state.create_text_template(input).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn push_text_template(
    state: tauri::State<'_, crate::state::AppState>,
    template_id: i64,
    device_id: String,
) -> Result<(), String> {
    state
        .push_text_template(template_id, &device_id)
        .await
        .map_err(|e| e.to_string())
}
