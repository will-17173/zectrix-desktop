#[tauri::command]
pub fn list_stock_watchlist(
    state: tauri::State<'_, crate::state::AppState>,
) -> Result<Vec<crate::models::StockWatchRecord>, String> {
    state.list_stock_watchlist().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_stock_watch(
    state: tauri::State<'_, crate::state::AppState>,
    code: String,
) -> Result<crate::models::StockWatchRecord, String> {
    state.add_stock_watch(&code).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_stock_watch(
    state: tauri::State<'_, crate::state::AppState>,
    code: String,
) -> Result<(), String> {
    state.remove_stock_watch(&code).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn push_stock_quotes(
    state: tauri::State<'_, crate::state::AppState>,
    device_id: String,
    page_id: u32,
) -> Result<(), String> {
    state
        .push_stock_quotes(&device_id, page_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_stock_push_task(
    state: tauri::State<'_, crate::state::AppState>,
) -> Result<Option<crate::models::StockPushTaskRecord>, String> {
    state.get_stock_push_task().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_stock_push_task(
    state: tauri::State<'_, crate::state::AppState>,
    device_id: String,
    page_id: u32,
    interval_seconds: u32,
) -> Result<crate::models::StockPushTaskRecord, String> {
    state
        .create_stock_push_task(&device_id, page_id, interval_seconds)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn start_stock_push_task(
    state: tauri::State<'_, crate::state::AppState>,
) -> Result<crate::models::StockPushTaskRecord, String> {
    state.start_stock_push_task().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn stop_stock_push_task(
    state: tauri::State<'_, crate::state::AppState>,
) -> Result<crate::models::StockPushTaskRecord, String> {
    state.stop_stock_push_task().map_err(|e| e.to_string())
}