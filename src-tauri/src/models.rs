use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub last_sync_time: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeyRecord {
    pub id: i64,
    pub name: String,
    pub key: String,
    #[serde(alias = "created_at", rename = "createdAt")]
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceRecord {
    pub device_id: String,
    pub alias: String,
    pub board: String,
    #[serde(alias = "cached_at", rename = "cachedAt")]
    pub cached_at: String,
    pub api_key_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapState {
    pub api_keys: Vec<ApiKeyRecord>,
    pub devices: Vec<DeviceRecord>,
    pub todos: Vec<TodoRecord>,
    pub text_templates: Vec<TextTemplateRecord>,
    pub image_templates: Vec<ImageTemplateRecord>,
    pub last_sync_time: Option<String>,
    pub page_cache: Vec<PageCacheRecord>,
    #[serde(default)]
    pub image_loop_tasks: Vec<ImageLoopTaskRecord>,
    #[serde(default)]
    pub custom_plugins: Vec<CustomPluginRecord>,
    #[serde(default)]
    pub plugin_loop_tasks: Vec<PluginLoopTaskRecord>,
    #[serde(default)]
    pub stock_watchlist: Vec<StockWatchRecord>,
    #[serde(default)]
    pub stock_push_task: Option<StockPushTaskRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TodoRecord {
    #[serde(default)]
    pub local_id: String,
    pub id: Option<i64>,
    pub title: String,
    pub description: String,
    #[serde(alias = "due_date")]
    pub due_date: Option<String>,
    #[serde(alias = "due_time")]
    pub due_time: Option<String>,
    #[serde(default)]
    pub repeat_type: Option<String>,
    #[serde(default)]
    pub repeat_weekday: Option<i32>,
    #[serde(default)]
    pub repeat_month: Option<i32>,
    #[serde(default)]
    pub repeat_day: Option<i32>,
    pub status: i32,
    pub priority: i32,
    #[serde(alias = "device_id")]
    pub device_id: Option<String>,
    pub dirty: bool,
    pub deleted: bool,
    #[serde(alias = "created_at", rename = "createdAt")]
    pub created_at: String,
    #[serde(alias = "updated_at", rename = "updatedAt")]
    pub updated_at: String,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "last_synced_status"
    )]
    pub last_synced_status: Option<i32>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "calendar_external_id"
    )]
    pub calendar_external_id: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "calendar_synced_at"
    )]
    pub calendar_synced_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TodoUpsertInput {
    pub title: String,
    #[serde(default)]
    pub description: String,
    pub due_date: Option<String>,
    pub due_time: Option<String>,
    pub repeat_type: Option<String>,
    pub repeat_weekday: Option<i32>,
    pub repeat_month: Option<i32>,
    pub repeat_day: Option<i32>,
    pub priority: Option<i32>,
    pub device_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextTemplateRecord {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextTemplateInput {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageTemplateRecord {
    pub id: i64,
    pub name: String,
    pub file_path: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageEditInput {
    pub source_path: Option<String>,
    pub source_data_url: Option<String>,
    pub crop: CropRect,
    pub rotation: u32,
    pub flip_x: bool,
    pub flip_y: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CropRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageTemplateSaveInput {
    pub name: String,
    pub source_path: Option<String>,
    pub source_data_url: Option<String>,
    pub crop: CropRect,
    pub rotation: u32,
    pub flip_x: bool,
    pub flip_y: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageCacheRecord {
    pub device_id: String,
    pub page_id: u32,
    pub content_type: String,
    pub thumbnail: Option<String>,
    pub metadata: Option<String>,
    pub pushed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomPluginRecord {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub code: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomPluginInput {
    pub id: Option<i64>,
    pub name: String,
    pub description: String,
    pub code: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginRunResult {
    pub output_type: String,
    pub title: Option<String>,
    pub text: Option<String>,
    pub image_data_url: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginLoopTaskRecord {
    pub id: i64,
    pub plugin_kind: String,
    pub plugin_id: String,
    pub name: String,
    pub device_id: String,
    pub page_id: u32,
    pub interval_seconds: u32,
    pub duration_type: String,
    pub end_time: Option<String>,
    pub duration_minutes: Option<u32>,
    pub status: String,
    pub last_push_at: Option<String>,
    pub error_message: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub config: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginLoopTaskInput {
    pub plugin_kind: String,
    pub plugin_id: String,
    pub name: String,
    pub device_id: String,
    pub page_id: u32,
    pub interval_seconds: u32,
    pub duration_type: String,
    pub end_time: Option<String>,
    pub duration_minutes: Option<u32>,
    #[serde(default)]
    pub config: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StockWatchRecord {
    pub code: String,
    #[serde(default = "default_market")]
    pub market: String,
    #[serde(alias = "created_at", rename = "createdAt")]
    pub created_at: String,
}

fn default_market() -> String {
    "a".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StockPushTaskRecord {
    pub id: i64,
    pub device_id: String,
    pub page_id: u32,
    pub interval_seconds: u32,
    pub status: String,
    pub error_message: Option<String>,
    pub started_at: Option<String>,
    pub last_push_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageLoopTaskRecord {
    pub id: i64,
    pub name: String,
    pub folder_path: String,
    pub device_id: String,
    pub page_id: u32,
    pub interval_seconds: u32,
    pub duration_type: String,
    pub end_time: Option<String>,
    pub duration_minutes: Option<u32>,
    pub status: String,
    pub current_index: u32,
    pub total_images: u32,
    pub started_at: Option<String>,
    pub last_push_at: Option<String>,
    pub error_message: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageLoopTaskInput {
    pub name: String,
    pub folder_path: String,
    pub device_id: String,
    pub page_id: u32,
    pub interval_seconds: u32,
    pub duration_type: String,
    pub end_time: Option<String>,
    pub duration_minutes: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageFolderScanResult {
    pub total_images: u32,
    pub image_files: Vec<String>,
    pub warning: Option<String>,
}

// ============ Calendar Sync ============

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SyncDirection {
    ToCalendar,
    FromCalendar,
    Bidirectional,
}

impl Default for SyncDirection {
    fn default() -> Self {
        SyncDirection::ToCalendar
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CalendarTargetType {
    CalendarEvent,
    Reminder,
}

impl Default for CalendarTargetType {
    fn default() -> Self {
        CalendarTargetType::Reminder
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CalendarSyncConfig {
    pub enabled: bool,
    #[serde(default)]
    pub direction: SyncDirection,
    #[serde(default)]
    pub target_type: CalendarTargetType,
    pub target_calendar_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarInfo {
    pub id: String,
    pub title: String,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SyncResult {
    pub created: u32,
    pub updated: u32,
    pub skipped: u32,
    pub deleted: u32,
}
