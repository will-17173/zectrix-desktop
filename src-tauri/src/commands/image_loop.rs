#[tauri::command]
pub async fn select_folder_dialog() -> Result<Option<String>, String> {
    let home = dirs::home_dir()
        .ok_or_else(|| "无法获取用户主目录".to_string())?;

    let result = rfd::AsyncFileDialog::new()
        .set_directory(&home)
        .pick_folder()
        .await;

    Ok(result.map(|handle| handle.path().to_string_lossy().to_string()))
}

#[tauri::command]
pub fn scan_image_folder(
    state: tauri::State<'_, crate::state::AppState>,
    folder_path: String,
) -> Result<crate::models::ImageFolderScanResult, String> {
    state.scan_image_folder(&folder_path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_image_loop_tasks(
    state: tauri::State<'_, crate::state::AppState>,
) -> Result<Vec<crate::models::ImageLoopTaskRecord>, String> {
    state.list_image_loop_tasks().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_image_loop_task(
    state: tauri::State<'_, crate::state::AppState>,
    input: crate::models::ImageLoopTaskInput,
) -> Result<crate::models::ImageLoopTaskRecord, String> {
    state.create_image_loop_task(input).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_image_loop_task(
    state: tauri::State<'_, crate::state::AppState>,
    task_id: i64,
    input: crate::models::ImageLoopTaskInput,
) -> Result<crate::models::ImageLoopTaskRecord, String> {
    state.update_image_loop_task(task_id, input).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_image_loop_task(
    state: tauri::State<'_, crate::state::AppState>,
    task_id: i64,
) -> Result<(), String> {
    state.delete_image_loop_task(task_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn start_image_loop_task(
    state: tauri::State<'_, crate::state::AppState>,
    task_id: i64,
) -> Result<crate::models::ImageLoopTaskRecord, String> {
    state.start_image_loop_task(task_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn stop_image_loop_task(
    state: tauri::State<'_, crate::state::AppState>,
    task_id: i64,
) -> Result<crate::models::ImageLoopTaskRecord, String> {
    state.stop_image_loop_task(task_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn push_folder_image(
    state: tauri::State<'_, crate::state::AppState>,
    task_id: i64,
) -> Result<crate::models::ImageLoopTaskRecord, String> {
    state.push_folder_image(task_id).await.map_err(|e| e.to_string())
}
