use crate::models::{
    CalendarInfo, CalendarSyncConfig, CalendarTargetType, SyncDirection, SyncResult, TodoRecord,
};
use crate::storage::{load_json, save_json};
use serde::Deserialize;
use std::process::Command;
use tauri::Manager;

// ─── Bridge path helper ──────────────────────────────────────────────────────

fn get_bridge_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let resource_dir = app
        .path()
        .resource_dir()
        .map_err(|e| format!("resource_dir error: {e}"))?;
    Ok(resource_dir.join("calendar-bridge"))
}

// ─── Bridge runner ───────────────────────────────────────────────────────────

fn run_bridge(bridge: &std::path::Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new(bridge)
        .args(args)
        .output()
        .map_err(|e| format!("failed to run calendar-bridge: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("calendar-bridge error: {stderr}"));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

// ─── Config helpers ──────────────────────────────────────────────────────────

fn config_path(data_dir: &std::path::Path) -> std::path::PathBuf {
    data_dir.join("calendar_sync.json")
}

// ─── Tauri commands ──────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_calendar_sync_config(
    state: tauri::State<'_, crate::state::AppState>,
) -> Result<CalendarSyncConfig, String> {
    load_json(&config_path(&state.data_dir)).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_calendar_sync_config(
    state: tauri::State<'_, crate::state::AppState>,
    config: CalendarSyncConfig,
) -> Result<(), String> {
    save_json(&config_path(&state.data_dir), &config).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_calendars(
    app: tauri::AppHandle,
    target_type: String,
) -> Result<Vec<CalendarInfo>, String> {
    let bridge = get_bridge_path(&app)?;
    let type_arg = if target_type == "CalendarEvent" {
        "calendar"
    } else {
        "reminder"
    };
    let arg = format!("--type={}", type_arg);
    let output = run_bridge(&bridge, &["list-calendars", &arg])?;

    #[derive(Deserialize)]
    struct BridgeCalendar {
        id: String,
        title: String,
        color: Option<String>,
    }
    let items: Vec<BridgeCalendar> =
        serde_json::from_str(&output).map_err(|e| format!("parse error: {e}"))?;
    Ok(items
        .into_iter()
        .map(|c| CalendarInfo {
            id: c.id,
            title: c.title,
            color: c.color,
        })
        .collect())
}

#[tauri::command]
pub async fn sync_calendar(
    app: tauri::AppHandle,
    state: tauri::State<'_, crate::state::AppState>,
) -> Result<SyncResult, String> {
    let config: CalendarSyncConfig =
        load_json(&config_path(&state.data_dir)).map_err(|e| e.to_string())?;

    if !config.enabled {
        return Err("日历同步未启用".to_string());
    }
    let calendar_id = config
        .target_calendar_id
        .as_deref()
        .ok_or("未选择目标日历本")?
        .to_string();

    let bridge = get_bridge_path(&app)?;

    // Load remote items
    let id_arg = format!("--calendar-id={}", calendar_id);
    let items_json = run_bridge(&bridge, &["list-items", &id_arg])?;

    #[derive(Deserialize, Clone)]
    #[serde(rename_all = "camelCase")]
    struct BridgeItem {
        external_id: String,
        title: String,
        #[allow(dead_code)]
        due_date: Option<String>,
        is_completed: bool,
        last_modified: String,
    }

    let remote_items: Vec<BridgeItem> =
        serde_json::from_str(&items_json).map_err(|e| format!("parse remote items: {e}"))?;

    // Load local todos
    let mut todos: Vec<TodoRecord> = load_json(&state.data_dir.join("todos.json"))
        .map_err(|e| e.to_string())?;

    let mut result = SyncResult::default();
    let target_type_str = match config.target_type {
        CalendarTargetType::CalendarEvent => "calendar",
        CalendarTargetType::Reminder => "reminder",
    };

    // Build a map of externalId -> remote item
    use std::collections::HashMap;
    let remote_map: HashMap<String, BridgeItem> = remote_items
        .iter()
        .cloned()
        .map(|i| (i.external_id.clone(), i))
        .collect();

    // Handle ToCalendar / Bidirectional: push local → remote
    let should_push = matches!(
        config.direction,
        SyncDirection::ToCalendar | SyncDirection::Bidirectional
    );
    // Handle FromCalendar / Bidirectional: pull remote → local
    let should_pull = matches!(
        config.direction,
        SyncDirection::FromCalendar | SyncDirection::Bidirectional
    );

    if should_push {
        for todo in todos.iter_mut() {
            if todo.deleted {
                if let Some(ref ext_id) = todo.calendar_external_id.clone() {
                    let arg = format!("--external-id={}", ext_id);
                    if run_bridge(&bridge, &["delete-item", &arg]).is_ok() {
                        result.deleted += 1;
                        todo.calendar_external_id = None;
                    }
                }
                continue;
            }

            let due_date = todo
                .due_date
                .as_deref()
                .map(|d| {
                    todo.due_time
                        .as_deref()
                        .map(|t| format!("{}T{}:00Z", d, t))
                        .unwrap_or_else(|| format!("{}T00:00:00Z", d))
                })
                .unwrap_or_default();

            if let Some(ref ext_id) = todo.calendar_external_id.clone() {
                // Check if remote is newer
                let remote_newer = remote_map
                    .get(ext_id)
                    .map_or(false, |r| r.last_modified.as_str() > todo.updated_at.as_str());
                if !remote_newer || matches!(config.direction, SyncDirection::ToCalendar) {
                    let data = serde_json::json!({
                        "title": todo.title,
                        "dueDate": due_date,
                        "isCompleted": todo.status == 1,
                    });
                    let ext_arg = format!("--external-id={}", ext_id);
                    let data_arg = format!("--data={}", data);
                    if run_bridge(&bridge, &["update-item", &ext_arg, &data_arg]).is_ok() {
                        todo.calendar_synced_at = Some(chrono::Utc::now().to_rfc3339());
                        result.updated += 1;
                    } else {
                        result.skipped += 1;
                    }
                } else {
                    result.skipped += 1;
                }
            } else {
                let data = serde_json::json!({
                    "calendarId": calendar_id,
                    "title": todo.title,
                    "dueDate": due_date,
                    "isCompleted": todo.status == 1,
                    "targetType": target_type_str,
                });
                let data_arg = format!("--data={}", data);
                let output = run_bridge(&bridge, &["create-item", &data_arg]);
                match output {
                    Ok(json) => {
                        #[derive(Deserialize)]
                        struct Created {
                            #[serde(rename = "externalId")]
                            external_id: String,
                        }
                        if let Ok(c) = serde_json::from_str::<Created>(&json) {
                            todo.calendar_external_id = Some(c.external_id);
                            todo.calendar_synced_at = Some(chrono::Utc::now().to_rfc3339());
                            result.created += 1;
                        }
                    }
                    Err(_) => {
                        result.skipped += 1;
                    }
                }
            }
        }
    }

    if should_pull {
        let local_ext_ids: std::collections::HashSet<String> = todos
            .iter()
            .filter_map(|t| t.calendar_external_id.clone())
            .collect();

        for remote in &remote_items {
            if local_ext_ids.contains(&remote.external_id) {
                // Find the local todo and update if remote is newer
                if let Some(todo) = todos.iter_mut().find(|t| {
                    t.calendar_external_id.as_deref() == Some(&remote.external_id)
                }) {
                    let remote_newer =
                        remote.last_modified.as_str() > todo.updated_at.as_str();
                    if remote_newer || matches!(config.direction, SyncDirection::FromCalendar) {
                        todo.title = remote.title.clone();
                        todo.status = if remote.is_completed { 1 } else { 0 };
                        if let Some(ref ds) = remote.due_date {
                            // Extract date and time from ISO8601
                            let parts: Vec<&str> = ds.splitn(2, 'T').collect();
                            todo.due_date = Some(parts[0].to_string());
                            if parts.len() > 1 {
                                let time = parts[1]
                                    .trim_end_matches('Z')
                                    .trim_end_matches("+00:00");
                                let hm: Vec<&str> = time.splitn(3, ':').collect();
                                if hm.len() >= 2 {
                                    todo.due_time = Some(format!("{}:{}", hm[0], hm[1]));
                                }
                            }
                        }
                        todo.dirty = true;
                        todo.updated_at = chrono::Utc::now().to_rfc3339();
                        todo.calendar_synced_at = Some(chrono::Utc::now().to_rfc3339());
                        result.updated += 1;
                    } else {
                        result.skipped += 1;
                    }
                }
            } else {
                // Create new local todo from remote
                let new_local_id =
                    format!("todo-cal-{}", remote.external_id.replace(['/', ':'], "-"));
                let parts: Vec<&str> = remote
                    .due_date
                    .as_deref()
                    .unwrap_or("")
                    .splitn(2, 'T')
                    .collect();
                let due_date = if parts[0].is_empty() {
                    None
                } else {
                    Some(parts[0].to_string())
                };
                let new_todo = TodoRecord {
                    local_id: new_local_id,
                    id: None,
                    title: remote.title.clone(),
                    description: String::new(),
                    due_date,
                    due_time: None,
                    repeat_type: None,
                    repeat_weekday: None,
                    repeat_month: None,
                    repeat_day: None,
                    status: if remote.is_completed { 1 } else { 0 },
                    priority: 0,
                    device_id: None,
                    dirty: true,
                    deleted: false,
                    created_at: chrono::Utc::now().to_rfc3339(),
                    updated_at: chrono::Utc::now().to_rfc3339(),
                    last_synced_status: None,
                    calendar_external_id: Some(remote.external_id.clone()),
                    calendar_synced_at: Some(chrono::Utc::now().to_rfc3339()),
                };
                todos.push(new_todo);
                result.created += 1;
            }
        }
    }

    save_json(&state.data_dir.join("todos.json"), &todos).map_err(|e| e.to_string())?;

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_data_dir() -> TempDir {
        tempfile::tempdir().unwrap()
    }

    #[test]
    fn config_defaults_when_file_missing() {
        let tmp = make_data_dir();
        let path = tmp.path().join("calendar_sync.json");
        let config: CalendarSyncConfig = load_json(&path).unwrap();
        assert!(!config.enabled);
        assert!(config.target_calendar_id.is_none());
    }

    #[test]
    fn config_roundtrip() {
        let tmp = make_data_dir();
        let path = tmp.path().join("calendar_sync.json");
        let cfg = CalendarSyncConfig {
            enabled: true,
            direction: SyncDirection::Bidirectional,
            target_type: CalendarTargetType::CalendarEvent,
            target_calendar_id: Some("cal-abc".to_string()),
        };
        save_json(&path, &cfg).unwrap();

        let loaded: CalendarSyncConfig = load_json(&path).unwrap();
        assert!(loaded.enabled);
        assert_eq!(loaded.target_calendar_id.as_deref(), Some("cal-abc"));
    }

    #[test]
    fn todo_record_calendar_fields_default_none() {
        let json = r#"{
            "localId": "todo-1",
            "id": null,
            "title": "test",
            "description": "",
            "status": 0,
            "priority": 0,
            "dirty": false,
            "deleted": false,
            "createdAt": "2026-01-01T00:00:00Z",
            "updatedAt": "2026-01-01T00:00:00Z"
        }"#;
        let todo: crate::models::TodoRecord = serde_json::from_str(json).unwrap();
        assert!(todo.calendar_external_id.is_none());
        assert!(todo.calendar_synced_at.is_none());
    }
}
