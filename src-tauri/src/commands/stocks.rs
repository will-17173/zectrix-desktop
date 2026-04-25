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