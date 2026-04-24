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
