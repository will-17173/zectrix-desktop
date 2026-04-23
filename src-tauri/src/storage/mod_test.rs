use crate::models::{AppConfig, DeviceRecord};
use crate::storage::{load_json, save_json};

#[test]
fn saves_and_loads_json_records() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("devices.json");
    let devices = vec![DeviceRecord {
        device_id: "AA:BB:CC:DD:EE:FF".into(),
        alias: "Desk".into(),
        board: "bread-compact-wifi".into(),
        cached_at: "2026-04-23T09:00:00Z".into(),
        api_key_id: 1,
    }];

    save_json(&path, &devices).unwrap();
    let loaded: Vec<DeviceRecord> = load_json(&path).unwrap();

    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].alias, "Desk");
}

#[test]
fn missing_json_file_returns_default_value() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.json");

    let loaded: AppConfig = load_json(&path).unwrap();

    assert_eq!(loaded.last_sync_time, None);
}
