#[tauri::command]
pub fn scan_image_folder(
    state: tauri::State<'_, crate::state::AppState>,
    folder_path: String,
) -> Result<crate::models::ImageFolderScanResult, String> {
    state.scan_image_folder(&folder_path).map_err(|e| e.to_string())
}
