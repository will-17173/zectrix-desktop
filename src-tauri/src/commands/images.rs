#[tauri::command]
pub fn render_image_preview(
    state: tauri::State<'_, crate::state::AppState>,
    input: crate::models::ImageEditInput,
) -> Result<String, String> {
    state.render_image_preview(input).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_image_template(
    state: tauri::State<'_, crate::state::AppState>,
    input: crate::models::ImageTemplateSaveInput,
) -> Result<crate::models::ImageTemplateRecord, String> {
    state.save_image_template(input).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_image_thumbnail(
    state: tauri::State<'_, crate::state::AppState>,
    template_id: i64,
) -> Result<String, String> {
    state.get_image_thumbnail(template_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn push_image_template(
    state: tauri::State<'_, crate::state::AppState>,
    template_id: i64,
    device_id: String,
    page_id: u32,
) -> Result<(), String> {
    state
        .push_image_template(template_id, &device_id, page_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_image_template(
    state: tauri::State<'_, crate::state::AppState>,
    template_id: i64,
) -> Result<(), String> {
    state.delete_image_template(template_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn push_sketch(
    state: tauri::State<'_, crate::state::AppState>,
    data_url: String,
    device_id: String,
    page_id: u32,
) -> Result<(), String> {
    state
        .push_sketch(&data_url, &device_id, page_id)
        .await
        .map_err(|e| e.to_string())
}
