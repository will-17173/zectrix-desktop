#[tauri::command]
pub fn list_custom_plugins(
    state: tauri::State<'_, crate::state::AppState>,
) -> Result<Vec<crate::models::CustomPluginRecord>, String> {
    state.list_custom_plugins().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_builtin_plugins() -> Result<Vec<crate::builtin_plugins::BuiltinPlugin>, String> {
    Ok(crate::builtin_plugins::list_builtin_plugins())
}

#[tauri::command]
pub fn save_custom_plugin(
    state: tauri::State<'_, crate::state::AppState>,
    input: crate::models::CustomPluginInput,
) -> Result<crate::models::CustomPluginRecord, String> {
    state.save_custom_plugin(input).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_custom_plugin(
    state: tauri::State<'_, crate::state::AppState>,
    plugin_id: i64,
) -> Result<(), String> {
    state.delete_custom_plugin(plugin_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn run_plugin_once(
    state: tauri::State<'_, crate::state::AppState>,
    plugin_kind: String,
    plugin_id: String,
    config: std::collections::HashMap<String, String>,
) -> Result<crate::models::PluginRunResult, String> {
    state
        .run_plugin_once(&plugin_kind, &plugin_id, config)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn push_plugin_once(
    state: tauri::State<'_, crate::state::AppState>,
    plugin_kind: String,
    plugin_id: String,
    device_id: String,
    page_id: u32,
    config: std::collections::HashMap<String, String>,
) -> Result<(), String> {
    state
        .push_plugin_once(&plugin_kind, &plugin_id, &device_id, page_id, config)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_plugin_loop_tasks(
    state: tauri::State<'_, crate::state::AppState>,
) -> Result<Vec<crate::models::PluginLoopTaskRecord>, String> {
    state.list_plugin_loop_tasks().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_plugin_loop_task(
    state: tauri::State<'_, crate::state::AppState>,
    input: crate::models::PluginLoopTaskInput,
) -> Result<crate::models::PluginLoopTaskRecord, String> {
    state.create_plugin_loop_task(input).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_plugin_loop_task(
    state: tauri::State<'_, crate::state::AppState>,
    task_id: i64,
    input: crate::models::PluginLoopTaskInput,
) -> Result<crate::models::PluginLoopTaskRecord, String> {
    state
        .update_plugin_loop_task(task_id, input)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_plugin_loop_task(
    state: tauri::State<'_, crate::state::AppState>,
    task_id: i64,
) -> Result<(), String> {
    state.delete_plugin_loop_task(task_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn start_plugin_loop_task(
    state: tauri::State<'_, crate::state::AppState>,
    task_id: i64,
) -> Result<crate::models::PluginLoopTaskRecord, String> {
    state.start_plugin_loop_task(task_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn stop_plugin_loop_task(
    state: tauri::State<'_, crate::state::AppState>,
    task_id: i64,
) -> Result<crate::models::PluginLoopTaskRecord, String> {
    state.stop_plugin_loop_task(task_id).map_err(|e| e.to_string())
}