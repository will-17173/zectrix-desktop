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
    page_id: Option<u32>,
) -> Result<(), String> {
    state
        .push_text_template(template_id, &device_id, page_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn push_text(
    state: tauri::State<'_, crate::state::AppState>,
    text: String,
    font_size: Option<u32>,
    device_id: String,
    page_id: Option<u32>,
) -> Result<(), String> {
    state
        .push_text(&text, font_size, &device_id, page_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn push_structured_text(
    state: tauri::State<'_, crate::state::AppState>,
    title: String,
    body: String,
    device_id: String,
    page_id: Option<u32>,
) -> Result<(), String> {
    state
        .push_structured_text(&title, &body, &device_id, page_id)
        .await
        .map_err(|e| e.to_string())
}
