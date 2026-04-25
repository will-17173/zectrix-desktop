use crate::models::{
    ApiKeyRecord, AppConfig, BootstrapState, CropRect, CustomPluginInput,
    CustomPluginRecord, DeviceRecord, ImageEditInput, ImageTemplateRecord,
    ImageTemplateSaveInput, PageCacheRecord, PluginLoopTaskInput, PluginLoopTaskRecord,
    PluginRunResult, StockPushTaskRecord, StockWatchRecord, TextTemplateInput,
    TextTemplateRecord, TodoRecord, TodoUpsertInput,
};
use crate::storage::{load_json, save_json};
use base64::Engine;
use image::GenericImageView;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

/// Execute a single image push for a running loop task (standalone function for background task)
async fn execute_image_loop_push(data_dir: PathBuf, task_id: i64) -> anyhow::Result<crate::models::ImageLoopTaskRecord> {
    let mut tasks: Vec<crate::models::ImageLoopTaskRecord> = crate::storage::load_json(
        &data_dir.join("image_loop_tasks.json"),
    )?;
    let task_idx = tasks
        .iter()
        .position(|t| t.id == task_id)
        .ok_or_else(|| anyhow::anyhow!("任务 {task_id} 未找到"))?;

    if tasks[task_idx].status != "running" {
        anyhow::bail!("任务未在运行中");
    }

    // 获取设备信息
    let devices: Vec<crate::models::DeviceRecord> = crate::storage::load_json(
        &data_dir.join("devices.json"),
    )?;
    let device = devices
        .iter()
        .find(|d| d.device_id.eq_ignore_ascii_case(&tasks[task_idx].device_id))
        .ok_or_else(|| anyhow::anyhow!("设备 {} 未找到", tasks[task_idx].device_id))?;

    // Get API key from api_keys.json
    let api_key: String = {
        let api_keys: Vec<crate::models::ApiKeyRecord> = crate::storage::load_json(
            &data_dir.join("api_keys.json"),
        )?;
        api_keys
            .iter()
            .find(|k| k.id == device.api_key_id)
            .map(|k| k.key.clone())
            .ok_or_else(|| anyhow::anyhow!("API Key ID {} 未找到", device.api_key_id))?
    };

    // 获取图片列表
    let folder_path = &tasks[task_idx].folder_path;
    let scan_result = scan_image_folder_standalone(folder_path)?;
    if scan_result.image_files.is_empty() {
        tasks[task_idx].status = "completed".to_string();
        tasks[task_idx].error_message = Some("文件夹中没有图片".to_string());
        tasks[task_idx].updated_at = chrono::Utc::now().to_rfc3339();
        let updated = tasks[task_idx].clone();
        crate::storage::save_json(&data_dir.join("image_loop_tasks.json"), &tasks)?;
        return Ok(updated);
    }

    // 获取当前图片路径
    let image_file = &scan_result.image_files[tasks[task_idx].current_index as usize];
    let image_path = std::path::Path::new(folder_path).join(image_file);

    // 加载并处理图片为 400x300
    // Use Reader::new().with_guessed_format() to auto-detect format from content, not extension
    let img = image::io::Reader::open(&image_path)
        .map_err(|e| anyhow::anyhow!("无法打开图片 {}: {}", image_file, e))?
        .with_guessed_format()
        .map_err(|e| anyhow::anyhow!("无法识别图片格式 {}: {}", image_file, e))?
        .decode()
        .map_err(|e| anyhow::anyhow!("无法解码图片 {}: {}", image_file, e))?;
    // Apply EXIF orientation if present
    let img = apply_exif_orientation(img, &image_path);
    let processed = img.resize_to_fill(400, 300, image::imageops::FilterType::Lanczos3);

    // 编码为 PNG
    let mut buf = std::io::Cursor::new(Vec::new());
    processed
        .write_to(&mut buf, image::ImageFormat::Png)
        .map_err(|e| anyhow::anyhow!("编码 PNG 失败: {}", e))?;
    let image_bytes = buf.into_inner();

    // 推送图片 (async call)
    let page_id = tasks[task_idx].page_id;
    let device_id = tasks[task_idx].device_id.clone();
    let push_result = crate::api::client::push_image(&api_key, &device_id, image_bytes.clone(), page_id).await;

    if let Err(e) = push_result {
        tasks[task_idx].status = "error".to_string();
        tasks[task_idx].error_message = Some(e.to_string());
        tasks[task_idx].updated_at = chrono::Utc::now().to_rfc3339();
        crate::storage::save_json(&data_dir.join("image_loop_tasks.json"), &tasks)?;
        anyhow::bail!("推送失败: {}", e);
    }

    // 更新索引
    tasks[task_idx].current_index = (tasks[task_idx].current_index + 1) % scan_result.total_images;
    tasks[task_idx].last_push_at = Some(chrono::Utc::now().to_rfc3339());
    tasks[task_idx].updated_at = chrono::Utc::now().to_rfc3339();

    // 检查持续时间条件
    let should_complete = check_duration_condition_standalone(&tasks[task_idx]);
    if should_complete {
        tasks[task_idx].status = "completed".to_string();
    }

    let updated = tasks[task_idx].clone();
    crate::storage::save_json(&data_dir.join("image_loop_tasks.json"), &tasks)?;
    Ok(updated)
}

/// Execute a single stock push for a running loop task
async fn execute_stock_push(data_dir: PathBuf, task_id: i64) -> anyhow::Result<()> {
    let task_opt: Option<StockPushTaskRecord> = load_json(&data_dir.join("stock_push_task.json"))?;
    let task = task_opt
        .ok_or_else(|| anyhow::anyhow!("股票推送任务 {task_id} 未找到"))?;

    if task.status != "running" {
        return Ok(()); // Task stopped, exit gracefully
    }

    // Get device info
    let devices: Vec<DeviceRecord> = load_json(&data_dir.join("devices.json"))?;
    let device = devices
        .iter()
        .find(|d| d.device_id.eq_ignore_ascii_case(&task.device_id))
        .ok_or_else(|| anyhow::anyhow!("设备 {} 未找到", task.device_id))?;

    // Get API key
    let api_key: String = {
        let api_keys: Vec<ApiKeyRecord> = load_json(&data_dir.join("api_keys.json"))?;
        api_keys
            .iter()
            .find(|k| k.id == device.api_key_id)
            .map(|k| k.key.clone())
            .ok_or_else(|| anyhow::anyhow!("API Key ID {} 未找到", device.api_key_id))?
    };

    // Load watchlist and fetch quotes
    let watchlist: Vec<StockWatchRecord> = load_json(&data_dir.join("stock_watchlist.json"))?;
    if watchlist.is_empty() {
        return Ok(()); // No stocks, skip push
    }

    let codes = watchlist.iter().map(|r| r.code.clone()).collect::<Vec<_>>();
    let quotes = crate::stock_quote::fetch_eastmoney_quotes(&codes).await?;
    let text = crate::stock_quote::format_stock_push_text(&quotes, chrono::Local::now());

    // Push text
    crate::api::client::push_text(&api_key, &task.device_id, &text, Some(20), Some(task.page_id)).await?;

    // Update last_push_at
    let mut task = task;
    task.last_push_at = Some(chrono::Utc::now().to_rfc3339());
    task.updated_at = chrono::Utc::now().to_rfc3339();
    save_json(&data_dir.join("stock_push_task.json"), &task)?;

    Ok(())
}

/// Standalone image folder scanner
fn scan_image_folder_standalone(folder_path: &str) -> anyhow::Result<crate::models::ImageFolderScanResult> {
    let path = std::path::Path::new(folder_path);
    if !path.exists() {
        anyhow::bail!("文件夹不存在");
    }
    if !path.is_dir() {
        anyhow::bail!("路径不是文件夹");
    }

    let image_extensions = ["jpg", "jpeg", "png", "gif", "bmp", "webp"];
    let mut image_files: Vec<String> = Vec::new();

    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let file_name = entry.file_name().to_string_lossy().to_string();
        let ext = file_name
            .rsplit('.')
            .next()
            .unwrap_or("")
            .to_lowercase();
        if image_extensions.contains(&ext.as_str()) {
            image_files.push(file_name);
        }
    }

    // Sort by filename for predictable order
    image_files.sort();

    let total_images = image_files.len() as u32;
    let warning = if total_images == 0 {
        Some("文件夹中没有图片".to_string())
    } else if total_images > 100 {
        Some("文件夹中图片数量超过100张，可能导致性能问题".to_string())
    } else {
        None
    };

    Ok(crate::models::ImageFolderScanResult {
        total_images,
        image_files,
        warning,
    })
}

/// Check duration condition standalone
fn check_duration_condition_standalone(task: &crate::models::ImageLoopTaskRecord) -> bool {
    if task.duration_type == "none" {
        return false;
    }

    let started_at = task.started_at.as_ref();
    if started_at.is_none() {
        return false;
    }

    let started = chrono::DateTime::parse_from_rfc3339(started_at.unwrap())
        .ok()
        .map(|dt| dt.with_timezone(&chrono::Utc));
    if started.is_none() {
        return false;
    }
    let started = started.unwrap();
    let now = chrono::Utc::now();

    if task.duration_type == "until_time" {
        if let Some(end_time) = &task.end_time {
            let parts: Vec<&str> = end_time.split(':').collect();
            if parts.len() == 2 {
                let hour: u32 = parts[0].parse().unwrap_or(0);
                let minute: u32 = parts[1].parse().unwrap_or(0);

                let end_datetime = now
                    .date_naive()
                    .and_hms_opt(hour, minute, 0)
                    .map(|dt| chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(dt, chrono::Utc));

                if let Some(end) = end_datetime {
                    if now >= end {
                        return true;
                    }
                }
            }
        }
    } else if task.duration_type == "for_duration" {
        if let Some(minutes) = task.duration_minutes {
            let elapsed = now - started;
            if elapsed.num_minutes() >= minutes as i64 {
                return true;
            }
        }
    }

    false
}

/// Apply EXIF orientation transformation to the image.
/// Returns the transformed image if orientation tag exists, otherwise returns original.
fn apply_exif_orientation(img: image::DynamicImage, path: &std::path::Path) -> image::DynamicImage {
    let exif_reader = exif::Reader::new();
    let file = std::fs::File::open(path);
    if file.is_err() {
        return img;
    }
    let exif = exif_reader.read_from_container(&mut std::io::BufReader::new(file.unwrap()));
    if exif.is_err() {
        return img;
    }
    let exif = exif.unwrap();

    let orientation = exif.get_field(exif::Tag::Orientation, exif::In::PRIMARY);
    if orientation.is_none() {
        return img;
    }

    // EXIF orientation values: 1-8
    // 1 = Normal
    // 2 = Flip horizontal
    // 3 = Rotate 180
    // 4 = Flip vertical
    // 5 = Rotate 90 + flip horizontal
    // 6 = Rotate 90
    // 7 = Rotate 270 + flip horizontal
    // 8 = Rotate 270
    let orientation_value = match &orientation.unwrap().value {
        exif::Value::Short(v) => v.first().copied().unwrap_or(1),
        _ => 1,
    };
    match orientation_value {
        1 => img,                                   // Normal
        2 => img.fliph(),                           // Flip horizontal
        3 => img.rotate180(),                       // Rotate 180
        4 => img.flipv(),                           // Flip vertical
        5 => img.rotate90().fliph(),                // Rotate 90 + flip horizontal
        6 => img.rotate90(),                        // Rotate 90 (portrait)
        7 => img.rotate270().fliph(),               // Rotate 270 + flip horizontal
        8 => img.rotate270(),                       // Rotate 270
        _ => img,
    }
}

static LOCAL_TODO_COUNTER: AtomicU64 = AtomicU64::new(1);

fn generate_local_todo_id() -> String {
    let counter = LOCAL_TODO_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("todo-{}-{}", chrono::Utc::now().timestamp_micros(), counter)
}

#[derive(Debug, Clone, Copy)]
struct RemoteSyncPlan {
    uses_complete_endpoint: bool,
}

fn plan_remote_sync(todo: &TodoRecord) -> RemoteSyncPlan {
    RemoteSyncPlan {
        uses_complete_endpoint: todo.id.is_some()
            && todo.status == 1
            && todo.last_synced_status != Some(1),
    }
}

fn mark_local_todo_synced(
    local_todos: &mut [TodoRecord],
    local_id: &str,
    remote_id: i64,
    synced_status: i32,
) {
    if let Some(todo) = local_todos.iter_mut().find(|t| t.local_id == local_id) {
        todo.id = Some(remote_id);
        todo.dirty = false;
        todo.updated_at = chrono::Utc::now().to_rfc3339();
        todo.last_synced_status = Some(synced_status);
    }
}

fn mark_remote_delete_synced(local_todos: &mut Vec<TodoRecord>, local_id: &str) {
    local_todos.retain(|todo| todo.local_id != local_id);
}

fn todo_upload_body(todo: &TodoRecord) -> crate::api::client::CreateTodoBody {
    crate::api::client::CreateTodoBody {
        title: todo.title.clone(),
        description: todo.description.clone(),
        due_date: todo.due_date.clone(),
        due_time: todo.due_time.clone(),
        repeat_type: todo.repeat_type.clone(),
        repeat_weekday: todo.repeat_weekday,
        repeat_month: todo.repeat_month,
        repeat_day: todo.repeat_day,
        priority: Some(todo.priority),
        device_id: todo.device_id.clone(),
    }
}

fn apply_local_state_to_api_todo(
    mut api_todo: crate::api::client::ApiTodo,
    local_todo: &TodoRecord,
) -> crate::api::client::ApiTodo {
    api_todo.title = local_todo.title.clone();
    api_todo.description = local_todo.description.clone();
    api_todo.due_date = local_todo.due_date.clone();
    api_todo.due_time = local_todo.due_time.clone();
    api_todo.repeat_type = local_todo.repeat_type.clone();
    api_todo.repeat_weekday = local_todo.repeat_weekday;
    api_todo.repeat_month = local_todo.repeat_month;
    api_todo.repeat_day = local_todo.repeat_day;
    api_todo.status = local_todo.status;
    api_todo.priority = local_todo.priority;
    api_todo.completed = local_todo.status == 1;
    api_todo.device_id = local_todo.device_id.clone();
    api_todo
}

fn merge_cloud_todos(
    local_todos: &[TodoRecord],
    cloud_todos: Vec<crate::api::client::ApiTodo>,
    synced_cloud_todos: HashMap<i64, crate::api::client::ApiTodo>,
    deleted_remote_ids: HashSet<i64>,
    now: &str,
) -> Vec<TodoRecord> {
    let mut effective_cloud_todos: HashMap<i64, crate::api::client::ApiTodo> = cloud_todos
        .into_iter()
        .filter(|todo| !deleted_remote_ids.contains(&todo.id))
        .map(|todo| (todo.id, todo))
        .collect();
    effective_cloud_todos.extend(synced_cloud_todos);

    let mut merged = Vec::with_capacity(local_todos.len() + effective_cloud_todos.len());

    for local in local_todos {
        match local.id {
            None => merged.push(local.clone()),
            Some(id) if local.dirty => {
                effective_cloud_todos.remove(&id);
                merged.push(local.clone());
            }
            Some(id) => {
                if let Some(cloud) = effective_cloud_todos.remove(&id) {
                    merged.push(TodoRecord {
                        local_id: local.local_id.clone(),
                        id: Some(cloud.id),
                        title: cloud.title,
                        description: cloud.description,
                        due_date: cloud.due_date,
                        due_time: cloud.due_time,
                        repeat_type: cloud.repeat_type,
                        repeat_weekday: cloud.repeat_weekday,
                        repeat_month: cloud.repeat_month,
                        repeat_day: cloud.repeat_day,
                        status: cloud.status,
                        priority: cloud.priority,
                        device_id: cloud.device_id,
                        dirty: false,
                        deleted: false,
                        created_at: cloud
                            .create_date
                            .unwrap_or_else(|| local.created_at.clone()),
                        updated_at: now.to_string(),
                        last_synced_status: Some(cloud.status),
                    });
                }
            }
        }
    }

    for cloud in effective_cloud_todos.into_values() {
        merged.push(TodoRecord {
            local_id: generate_local_todo_id(),
            id: Some(cloud.id),
            title: cloud.title,
            description: cloud.description,
            due_date: cloud.due_date,
            due_time: cloud.due_time,
            repeat_type: cloud.repeat_type,
            repeat_weekday: cloud.repeat_weekday,
            repeat_month: cloud.repeat_month,
            repeat_day: cloud.repeat_day,
            status: cloud.status,
            priority: cloud.priority,
            device_id: cloud.device_id,
            dirty: false,
            deleted: false,
            created_at: cloud.create_date.unwrap_or_else(|| now.to_string()),
            updated_at: now.to_string(),
            last_synced_status: Some(cloud.status),
        });
    }

    merged
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LegacyTodoRecord {
    #[serde(rename = "id", default)]
    _legacy_id: Option<i64>,
    #[serde(default, alias = "cloudId", alias = "cloud_id")]
    cloud_id: Option<i64>,
    #[serde(default, alias = "localId")]
    local_id: Option<String>,
    title: String,
    description: String,
    #[serde(default, alias = "due_date")]
    due_date: Option<String>,
    #[serde(default, alias = "due_time")]
    due_time: Option<String>,
    #[serde(default)]
    status: i32,
    #[serde(default)]
    priority: i32,
    #[serde(default, alias = "device_id")]
    device_id: Option<String>,
    #[serde(default)]
    dirty: bool,
    #[serde(default)]
    deleted: bool,
    #[serde(default, alias = "created_at")]
    created_at: Option<String>,
    #[serde(default, alias = "updated_at")]
    updated_at: Option<String>,
}

fn legacy_remote_id(item: &serde_json::Value) -> Option<i64> {
    item.get("cloudId")
        .or_else(|| item.get("cloud_id"))
        .and_then(serde_json::Value::as_i64)
}

fn has_sync_metadata(item: &serde_json::Value) -> bool {
    item.get("lastSyncedStatus").is_some() || item.get("last_synced_status").is_some()
}

fn has_local_id(item: &serde_json::Value) -> bool {
    item.get("localId").is_some() || item.get("local_id").is_some()
}

fn todo_from_legacy(legacy: LegacyTodoRecord) -> TodoRecord {
    TodoRecord {
        local_id: legacy.local_id.unwrap_or_else(generate_local_todo_id),
        id: legacy.cloud_id,
        title: legacy.title,
        description: legacy.description,
        due_date: legacy.due_date,
        due_time: legacy.due_time,
        repeat_type: None,
        repeat_weekday: None,
        repeat_month: None,
        repeat_day: None,
        status: legacy.status,
        priority: legacy.priority,
        device_id: legacy.device_id,
        dirty: legacy.dirty,
        deleted: legacy.deleted,
        created_at: legacy
            .created_at
            .unwrap_or_else(|| chrono::Utc::now().to_rfc3339()),
        updated_at: legacy
            .updated_at
            .unwrap_or_else(|| chrono::Utc::now().to_rfc3339()),
        last_synced_status: if legacy.cloud_id.is_some() && !legacy.dirty {
            Some(legacy.status)
        } else {
            None
        },
    }
}

fn page_cache_uses_thumbnail(content_type: &str) -> bool {
    matches!(content_type, "sketch" | "image" | "plugin_image")
}

pub struct AppState {
    pub data_dir: PathBuf,
}

impl AppState {
    pub fn new() -> anyhow::Result<Self> {
        let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("no home directory"))?;
        let data_dir = home.join(".zectrix-note");
        std::fs::create_dir_all(&data_dir)?;
        Ok(Self { data_dir })
    }

    pub fn load_bootstrap_state(&self) -> anyhow::Result<BootstrapState> {
        let config: AppConfig = load_json(&self.data_dir.join("config.json"))?;
        let api_keys = load_json(&self.data_dir.join("api_keys.json"))?;
        let devices = load_json(&self.data_dir.join("devices.json"))?;
        let todos = self.load_todos()?;
        let text_templates = load_json(&self.data_dir.join("text_templates.json"))?;
        let image_templates = load_json(&self.data_dir.join("image_templates.json"))?;
        let page_cache = self.load_page_cache()?;
        let image_loop_tasks = self.load_image_loop_tasks()?;
        let custom_plugins = self.load_custom_plugins()?;
        let plugin_loop_tasks = self.load_plugin_loop_tasks()?;
        let stock_watchlist = self.load_stock_watchlist()?;
        let stock_push_task = self.load_stock_push_task()?;

        Ok(BootstrapState {
            api_keys,
            devices,
            todos,
            text_templates,
            image_templates,
            last_sync_time: config.last_sync_time,
            page_cache,
            image_loop_tasks,
            custom_plugins,
            plugin_loop_tasks,
            stock_watchlist,
            stock_push_task,
        })
    }

    pub fn list_api_keys(&self) -> anyhow::Result<Vec<ApiKeyRecord>> {
        load_json(&self.data_dir.join("api_keys.json"))
    }

    pub fn add_api_key(&self, name: &str, key: &str) -> anyhow::Result<ApiKeyRecord> {
        let path = self.data_dir.join("api_keys.json");
        let mut api_keys: Vec<ApiKeyRecord> = load_json(&path)?;
        let next_id = api_keys.iter().map(|k| k.id).max().unwrap_or(0) + 1;
        let now = chrono::Utc::now().to_rfc3339();
        let record = ApiKeyRecord {
            id: next_id,
            name: name.to_string(),
            key: key.to_string(),
            created_at: now,
        };
        api_keys.push(record.clone());
        save_json(&path, &api_keys)?;
        Ok(record)
    }

    pub fn remove_api_key(&self, id: i64) -> anyhow::Result<()> {
        // 检查是否有设备关联此 API Key
        let devices: Vec<DeviceRecord> = load_json(&self.data_dir.join("devices.json"))?;
        if devices.iter().any(|d| d.api_key_id == id) {
            anyhow::bail!("该 API Key 有关联设备，请先删除关联设备");
        }

        let path = self.data_dir.join("api_keys.json");
        let mut api_keys: Vec<ApiKeyRecord> = load_json(&path)?;
        api_keys.retain(|k| k.id != id);
        save_json(&path, &api_keys)?;
        Ok(())
    }

    pub fn get_api_key_by_id(&self, id: i64) -> anyhow::Result<String> {
        let api_keys: Vec<ApiKeyRecord> = load_json(&self.data_dir.join("api_keys.json"))?;
        api_keys
            .iter()
            .find(|k| k.id == id)
            .map(|k| k.key.clone())
            .ok_or_else(|| anyhow::anyhow!("API Key ID {id} 未找到"))
    }

    pub async fn validate_and_cache_device(
        &self,
        device_id: &str,
        api_key_id: i64,
    ) -> anyhow::Result<DeviceRecord> {
        let api_key = self.get_api_key_by_id(api_key_id)?;
        let api_device = crate::api::client::validate_device(&api_key, device_id).await?;

        let record = DeviceRecord {
            device_id: api_device.device_id,
            alias: api_device.alias,
            board: api_device.board,
            cached_at: chrono::Utc::now().to_rfc3339(),
            api_key_id,
        };

        let path = self.data_dir.join("devices.json");
        let mut devices: Vec<DeviceRecord> = load_json(&path)?;
        if !devices.iter().any(|d| d.device_id == record.device_id) {
            devices.push(record.clone());
            save_json(&path, &devices)?;
        }

        Ok(record)
    }

    pub fn remove_device_cache(&self, device_id: &str) -> anyhow::Result<()> {
        // 删除关联该设备的所有 todo
        let todos_path = self.data_dir.join("todos.json");
        if todos_path.exists() {
            let mut todos = self.load_todos()?;
            todos.retain(|t| t.device_id.as_deref() != Some(device_id));
            save_json(&todos_path, &todos)?;
        }

        // 删除设备缓存
        let path = self.data_dir.join("devices.json");
        let mut devices: Vec<DeviceRecord> = load_json(&path)?;
        devices.retain(|d| !d.device_id.eq_ignore_ascii_case(device_id));
        save_json(&path, &devices)?;
        Ok(())
    }

    fn load_page_cache(&self) -> anyhow::Result<Vec<PageCacheRecord>> {
        load_json(&self.data_dir.join("page_cache.json"))
    }

    fn load_stock_watchlist(&self) -> anyhow::Result<Vec<StockWatchRecord>> {
        load_json(&self.data_dir.join("stock_watchlist.json"))
    }

    pub fn list_stock_watchlist(&self) -> anyhow::Result<Vec<StockWatchRecord>> {
        self.load_stock_watchlist()
    }

    fn save_stock_watchlist(&self, records: &[StockWatchRecord]) -> anyhow::Result<()> {
        let records = records.to_vec();
        save_json(&self.data_dir.join("stock_watchlist.json"), &records)
    }

    pub fn add_stock_watch(&self, code: &str) -> anyhow::Result<StockWatchRecord> {
        let normalized = crate::stock_quote::normalize_stock_code(code)?;
        let mut records = self.load_stock_watchlist()?;

        if records.iter().any(|record| record.code == normalized) {
            anyhow::bail!("股票代码 {normalized} 已存在");
        }

        let record = StockWatchRecord {
            code: normalized,
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        records.push(record.clone());
        self.save_stock_watchlist(&records)?;

        Ok(record)
    }

    pub fn remove_stock_watch(&self, code: &str) -> anyhow::Result<()> {
        let normalized = crate::stock_quote::normalize_stock_code(code)?;
        let mut records = self.load_stock_watchlist()?;
        let before = records.len();
        records.retain(|record| record.code != normalized);

        if records.len() == before {
            anyhow::bail!("股票代码 {normalized} 未找到");
        }

        self.save_stock_watchlist(&records)
    }

    fn load_custom_plugins(&self) -> anyhow::Result<Vec<CustomPluginRecord>> {
        let path = self.data_dir.join("custom_plugins.json");
        if !path.exists() {
            return Ok(Vec::new());
        }

        load_json(&path)
    }

    pub fn list_custom_plugins(&self) -> anyhow::Result<Vec<CustomPluginRecord>> {
        self.load_custom_plugins()
    }

    fn save_custom_plugins(&self, plugins: &[CustomPluginRecord]) -> anyhow::Result<()> {
        let plugins = plugins.to_vec();
        save_json(&self.data_dir.join("custom_plugins.json"), &plugins)
    }

    pub fn save_custom_plugin(&self, input: CustomPluginInput) -> anyhow::Result<CustomPluginRecord> {
        let mut plugins = self.load_custom_plugins()?;
        let now = chrono::Utc::now().to_rfc3339();
        let name = input.name.trim().to_string();
        let description = input.description.trim().to_string();

        if let Some(plugin_id) = input.id {
            let plugin = plugins
                .iter_mut()
                .find(|plugin| plugin.id == plugin_id)
                .ok_or_else(|| anyhow::anyhow!("插件 {plugin_id} 未找到"))?;

            plugin.name = name;
            plugin.description = description;
            plugin.code = input.code;
            plugin.updated_at = now;

            let updated = plugin.clone();
            self.save_custom_plugins(&plugins)?;
            return Ok(updated);
        }

        let next_id = plugins.iter().map(|plugin| plugin.id).max().unwrap_or(0) + 1;
        let record = CustomPluginRecord {
            id: next_id,
            name,
            description,
            code: input.code,
            created_at: now.clone(),
            updated_at: now,
        };

        plugins.push(record.clone());
        self.save_custom_plugins(&plugins)?;
        Ok(record)
    }

    pub fn delete_custom_plugin(&self, plugin_id: i64) -> anyhow::Result<()> {
        let mut plugins = self.load_custom_plugins()?;
        plugins.retain(|plugin| plugin.id != plugin_id);
        self.save_custom_plugins(&plugins)
    }

    fn find_custom_plugin(&self, plugin_id: &str) -> anyhow::Result<CustomPluginRecord> {
        let parsed_id = plugin_id
            .trim()
            .parse::<i64>()
            .map_err(|_| anyhow::anyhow!("插件 ID {plugin_id} 无效"))?;

        self.load_custom_plugins()?
            .into_iter()
            .find(|plugin| plugin.id == parsed_id)
            .ok_or_else(|| anyhow::anyhow!("插件 {plugin_id} 未找到"))
    }

    pub async fn run_plugin_once(
        &self,
        plugin_kind: &str,
        plugin_id: &str,
    ) -> anyhow::Result<PluginRunResult> {
        let plugin = match plugin_kind {
            "custom" => self.find_custom_plugin(plugin_id)?,
            "builtin" => anyhow::bail!("内置插件尚未接入"),
            other => anyhow::bail!("未知插件类型: {other}"),
        };

        let raw = crate::plugin_runtime::run_plugin_code(&plugin.code).await?;
        let output = crate::plugin_output::parse_plugin_output(raw)?;
        Self::plugin_output_to_run_result(output)
    }

    pub async fn push_plugin_once(
        &self,
        plugin_kind: &str,
        plugin_id: &str,
        device_id: &str,
        page_id: u32,
    ) -> anyhow::Result<()> {
        let plugin = match plugin_kind {
            "custom" => self.find_custom_plugin(plugin_id)?,
            "builtin" => anyhow::bail!("内置插件尚未接入"),
            other => anyhow::bail!("未知插件类型: {other}"),
        };

        let raw = crate::plugin_runtime::run_plugin_code(&plugin.code).await?;
        let output = crate::plugin_output::parse_plugin_output(raw)?;

        self.push_plugin_output(
            plugin_kind,
            plugin_id,
            &plugin.name,
            output,
            device_id,
            page_id,
        )
        .await
    }

    fn plugin_output_to_run_result(
        output: crate::plugin_output::PluginOutput,
    ) -> anyhow::Result<PluginRunResult> {
        match output {
            crate::plugin_output::PluginOutput::Text(text) => Ok(PluginRunResult {
                output_type: "text".to_string(),
                title: text.title,
                text: Some(text.text),
                image_data_url: None,
                preview_png_base64: None,
                metadata: text.metadata,
            }),
            crate::plugin_output::PluginOutput::TextImage(text_image) => {
                let rendered_png = crate::text_image::render_text_to_png(
                    &text_image.text,
                    &text_image.style,
                )?;

                Ok(PluginRunResult {
                    output_type: "textImage".to_string(),
                    title: text_image.title,
                    text: Some(text_image.text),
                    image_data_url: None,
                    preview_png_base64: Some(
                        base64::engine::general_purpose::STANDARD.encode(rendered_png),
                    ),
                    metadata: text_image.metadata,
                })
            }
            crate::plugin_output::PluginOutput::Image(image) => Ok(PluginRunResult {
                output_type: "image".to_string(),
                title: image.title,
                text: None,
                image_data_url: Some(image.image_data_url),
                preview_png_base64: None,
                metadata: image.metadata,
            }),
        }
    }

    async fn push_plugin_output(
        &self,
        plugin_kind: &str,
        plugin_id: &str,
        plugin_name: &str,
        output: crate::plugin_output::PluginOutput,
        device_id: &str,
        page_id: u32,
    ) -> anyhow::Result<()> {
        let device = self
            .list_device_cache()?
            .into_iter()
            .find(|candidate| candidate.device_id.eq_ignore_ascii_case(device_id))
            .ok_or_else(|| anyhow::anyhow!("设备 {device_id} 未找到"))?;
        let api_key = self.get_api_key_by_id(device.api_key_id)?;
        let now = chrono::Utc::now().to_rfc3339();

        match output {
            crate::plugin_output::PluginOutput::Text(text) => {
                crate::api::client::push_text(
                    &api_key,
                    device_id,
                    &text.text,
                    Some(text.font_size),
                    Some(page_id),
                )
                .await?;

                let record = PageCacheRecord {
                    device_id: device_id.to_string(),
                    page_id,
                    content_type: "plugin_text".to_string(),
                    thumbnail: Some(text.text.chars().take(100).collect()),
                    metadata: Some(Self::plugin_cache_metadata(
                        plugin_name,
                        plugin_kind,
                        plugin_id,
                        "text",
                        text.title.as_deref(),
                        text.metadata.as_ref(),
                        &now,
                    )),
                    pushed_at: now,
                };
                self.save_page_cache_record(record)?;
            }
            crate::plugin_output::PluginOutput::TextImage(text_image) => {
                let rendered_png = crate::text_image::render_text_to_png(
                    &text_image.text,
                    &text_image.style,
                )?;
                crate::api::client::push_image(&api_key, device_id, rendered_png.clone(), page_id)
                    .await?;

                let thumbnail_filename = format!(
                    "thumb_{}_{}.png",
                    device_id.replace(':', "_"),
                    page_id
                );
                let thumbnail = self.save_image_thumbnail(&rendered_png, &thumbnail_filename)?;

                let record = PageCacheRecord {
                    device_id: device_id.to_string(),
                    page_id,
                    content_type: "plugin_image".to_string(),
                    thumbnail: Some(thumbnail),
                    metadata: Some(Self::plugin_cache_metadata(
                        plugin_name,
                        plugin_kind,
                        plugin_id,
                        "textImage",
                        text_image.title.as_deref(),
                        text_image.metadata.as_ref(),
                        &now,
                    )),
                    pushed_at: now,
                };
                self.save_page_cache_record(record)?;
            }
            crate::plugin_output::PluginOutput::Image(image) => {
                let png_bytes = Self::decode_plugin_image_to_png(&image.image_data_url)?;
                crate::api::client::push_image(&api_key, device_id, png_bytes.clone(), page_id)
                    .await?;

                let thumbnail_filename = format!(
                    "thumb_{}_{}.png",
                    device_id.replace(':', "_"),
                    page_id
                );
                let thumbnail = self.save_image_thumbnail(&png_bytes, &thumbnail_filename)?;

                let record = PageCacheRecord {
                    device_id: device_id.to_string(),
                    page_id,
                    content_type: "plugin_image".to_string(),
                    thumbnail: Some(thumbnail),
                    metadata: Some(Self::plugin_cache_metadata(
                        plugin_name,
                        plugin_kind,
                        plugin_id,
                        "image",
                        image.title.as_deref(),
                        image.metadata.as_ref(),
                        &now,
                    )),
                    pushed_at: now,
                };
                self.save_page_cache_record(record)?;
            }
        }

        Ok(())
    }

    fn plugin_cache_metadata(
        plugin_name: &str,
        plugin_kind: &str,
        plugin_id: &str,
        output_type: &str,
        title: Option<&str>,
        metadata: Option<&serde_json::Value>,
        run_at: &str,
    ) -> String {
        serde_json::json!({
            "pluginName": plugin_name,
            "pluginKind": plugin_kind,
            "pluginId": plugin_id,
            "outputType": output_type,
            "title": title,
            "metadata": metadata.cloned(),
            "runAt": run_at,
        })
        .to_string()
    }

    fn decode_plugin_image_to_png(data_url: &str) -> anyhow::Result<Vec<u8>> {
        if !data_url.contains(";base64,") {
            anyhow::bail!("图片数据格式错误");
        }

        let (_, payload) = data_url
            .split_once(',')
            .ok_or_else(|| anyhow::anyhow!("图片数据格式错误"))?;
        let image_bytes = base64::engine::general_purpose::STANDARD
            .decode(payload)
            .map_err(|error| anyhow::anyhow!("图片解码失败: {error}"))?;
        let image = image::load_from_memory(&image_bytes)
            .map_err(|error| anyhow::anyhow!("无法读取图片: {error}"))?;
        let resized = image.resize_to_fill(400, 300, image::imageops::FilterType::Lanczos3);
        let mut cursor = std::io::Cursor::new(Vec::new());
        resized
            .write_to(&mut cursor, image::ImageFormat::Png)
            .map_err(|error| anyhow::anyhow!("编码 PNG 失败: {error}"))?;

        Ok(cursor.into_inner())
    }

    fn load_plugin_loop_tasks(&self) -> anyhow::Result<Vec<PluginLoopTaskRecord>> {
        let path = self.data_dir.join("plugin_loop_tasks.json");
        if !path.exists() {
            return Ok(Vec::new());
        }

        load_json(&path)
    }

    pub fn list_plugin_loop_tasks(&self) -> anyhow::Result<Vec<PluginLoopTaskRecord>> {
        self.load_plugin_loop_tasks()
    }

    fn save_plugin_loop_tasks(&self, tasks: &[PluginLoopTaskRecord]) -> anyhow::Result<()> {
        let tasks = tasks.to_vec();
        save_json(&self.data_dir.join("plugin_loop_tasks.json"), &tasks)
    }

    pub fn create_plugin_loop_task(
        &self,
        input: PluginLoopTaskInput,
    ) -> anyhow::Result<PluginLoopTaskRecord> {
        let mut tasks = self.load_plugin_loop_tasks()?;
        let next_id = tasks.iter().map(|task| task.id).max().unwrap_or(0) + 1;
        let now = chrono::Utc::now().to_rfc3339();

        let record = PluginLoopTaskRecord {
            id: next_id,
            plugin_kind: input.plugin_kind,
            plugin_id: input.plugin_id,
            name: input.name.trim().to_string(),
            device_id: input.device_id,
            page_id: input.page_id,
            interval_seconds: input.interval_seconds,
            duration_type: input.duration_type,
            end_time: input.end_time,
            duration_minutes: input.duration_minutes,
            status: "idle".to_string(),
            last_push_at: None,
            error_message: None,
            created_at: now.clone(),
            updated_at: now,
        };

        tasks.push(record.clone());
        self.save_plugin_loop_tasks(&tasks)?;
        Ok(record)
    }

    pub fn update_plugin_loop_task(
        &self,
        task_id: i64,
        input: PluginLoopTaskInput,
    ) -> anyhow::Result<PluginLoopTaskRecord> {
        let mut tasks = self.load_plugin_loop_tasks()?;
        let task = tasks
            .iter_mut()
            .find(|task| task.id == task_id)
            .ok_or_else(|| anyhow::anyhow!("任务 {task_id} 未找到"))?;

        task.plugin_kind = input.plugin_kind;
        task.plugin_id = input.plugin_id;
        task.name = input.name.trim().to_string();
        task.device_id = input.device_id;
        task.page_id = input.page_id;
        task.interval_seconds = input.interval_seconds;
        task.duration_type = input.duration_type;
        task.end_time = input.end_time;
        task.duration_minutes = input.duration_minutes;
        task.updated_at = chrono::Utc::now().to_rfc3339();

        let updated = task.clone();
        self.save_plugin_loop_tasks(&tasks)?;
        Ok(updated)
    }

    pub fn delete_plugin_loop_task(&self, task_id: i64) -> anyhow::Result<()> {
        let mut tasks = self.load_plugin_loop_tasks()?;
        tasks.retain(|task| task.id != task_id);
        self.save_plugin_loop_tasks(&tasks)
    }

    pub fn start_plugin_loop_task(&self, task_id: i64) -> anyhow::Result<PluginLoopTaskRecord> {
        let mut tasks = self.load_plugin_loop_tasks()?;
        let task = tasks
            .iter_mut()
            .find(|task| task.id == task_id)
            .ok_or_else(|| anyhow::anyhow!("插件循环任务 {task_id} 不存在"))?;

        task.status = "running".to_string();
        task.error_message = None;
        task.updated_at = chrono::Utc::now().to_rfc3339();

        let updated = task.clone();
        self.save_plugin_loop_tasks(&tasks)?;
        Ok(updated)
    }

    pub fn stop_plugin_loop_task(&self, task_id: i64) -> anyhow::Result<PluginLoopTaskRecord> {
        let mut tasks = self.load_plugin_loop_tasks()?;
        let task = tasks
            .iter_mut()
            .find(|task| task.id == task_id)
            .ok_or_else(|| anyhow::anyhow!("插件循环任务 {task_id} 不存在"))?;

        task.status = "idle".to_string();
        task.updated_at = chrono::Utc::now().to_rfc3339();

        let updated = task.clone();
        self.save_plugin_loop_tasks(&tasks)?;
        Ok(updated)
    }

    pub fn mark_plugin_loop_task_pushed(
        &self,
        task_id: i64,
    ) -> anyhow::Result<PluginLoopTaskRecord> {
        let mut tasks = self.load_plugin_loop_tasks()?;
        let task = tasks
            .iter_mut()
            .find(|task| task.id == task_id)
            .ok_or_else(|| anyhow::anyhow!("插件循环任务 {task_id} 不存在"))?;
        let now = chrono::Utc::now().to_rfc3339();

        task.last_push_at = Some(now.clone());
        task.error_message = None;
        task.updated_at = now;

        let updated = task.clone();
        self.save_plugin_loop_tasks(&tasks)?;
        Ok(updated)
    }

    pub fn mark_plugin_loop_task_error(
        &self,
        task_id: i64,
        error_message: &str,
    ) -> anyhow::Result<PluginLoopTaskRecord> {
        let mut tasks = self.load_plugin_loop_tasks()?;
        let task = tasks
            .iter_mut()
            .find(|task| task.id == task_id)
            .ok_or_else(|| anyhow::anyhow!("插件循环任务 {task_id} 不存在"))?;

        task.status = "error".to_string();
        task.error_message = Some(error_message.to_string());
        task.updated_at = chrono::Utc::now().to_rfc3339();

        let updated = task.clone();
        self.save_plugin_loop_tasks(&tasks)?;
        Ok(updated)
    }

    pub fn get_page_cache_list(&self, device_id: &str) -> anyhow::Result<Vec<PageCacheRecord>> {
        let all = self.load_page_cache()?;
        let filtered: Vec<PageCacheRecord> = all
            .into_iter()
            .filter(|p| p.device_id.eq_ignore_ascii_case(device_id))
            .collect();
        Ok(filtered)
    }

    pub async fn delete_page_cache(&self, device_id: &str, page_id: u32) -> anyhow::Result<()> {
        // 1. 删除本地缓存
        let path = self.data_dir.join("page_cache.json");
        let mut cache: Vec<PageCacheRecord> = self.load_page_cache()?;
        let record = cache
            .iter()
            .find(|p| p.device_id.eq_ignore_ascii_case(device_id) && p.page_id == page_id)
            .cloned();

        // 删除缩略图文件（如果是图片类型）
        if let Some(rec) = &record {
            if page_cache_uses_thumbnail(&rec.content_type) {
                if let Some(thumbnail_path) = &rec.thumbnail {
                    let thumb_path = self.data_dir.join("thumbnails").join(thumbnail_path);
                    if std::fs::metadata(&thumb_path).is_ok() {
                        std::fs::remove_file(&thumb_path)?;
                    }
                }
            }
        }

        cache.retain(|p| !p.device_id.eq_ignore_ascii_case(device_id) || p.page_id != page_id);
        save_json(&path, &cache)?;

        // 2. 调用云端 API
        let devices: Vec<DeviceRecord> = load_json(&self.data_dir.join("devices.json"))?;
        let device = devices
            .iter()
            .find(|d| d.device_id.eq_ignore_ascii_case(device_id))
            .ok_or_else(|| anyhow::anyhow!("设备 {device_id} 未找到"))?;

        let api_key = self.get_api_key_by_id(device.api_key_id)?;
        crate::api::client::delete_page(&api_key, device_id, Some(page_id)).await?;

        Ok(())
    }

    fn save_page_cache(&self, cache: &Vec<PageCacheRecord>) -> anyhow::Result<()> {
        save_json(&self.data_dir.join("page_cache.json"), cache)
    }

    fn save_page_cache_record(&self, record: PageCacheRecord) -> anyhow::Result<()> {
        let path = self.data_dir.join("page_cache.json");
        let mut cache: Vec<PageCacheRecord> = if path.exists() {
            self.load_page_cache()?
        } else {
            Vec::new()
        };

        // 先找旧记录用于删除缩略图
        let old = cache.iter().find(|p|
            p.device_id.eq_ignore_ascii_case(&record.device_id) && p.page_id == record.page_id
        ).cloned();

        // 如果有旧的缩略图文件，删除它
        if let Some(old_rec) = old {
            if page_cache_uses_thumbnail(&old_rec.content_type) {
                if let Some(thumbnail_path) = &old_rec.thumbnail {
                    let thumb_path = self.data_dir.join("thumbnails").join(thumbnail_path);
                    if std::fs::metadata(&thumb_path).is_ok() {
                        std::fs::remove_file(&thumb_path)?;
                    }
                }
            }
        }

        // 移除同设备同页面的旧记录
        cache.retain(|p| !p.device_id.eq_ignore_ascii_case(&record.device_id) || p.page_id != record.page_id);
        cache.push(record);
        save_json(&path, &cache)?;
        Ok(())
    }

    fn save_image_thumbnail(&self, image_bytes: &[u8], filename: &str) -> anyhow::Result<String> {
        let thumbnails_dir = self.data_dir.join("thumbnails");
        std::fs::create_dir_all(&thumbnails_dir)?;

        let img = image::load_from_memory(image_bytes)
            .map_err(|e| anyhow::anyhow!("无法读取图片: {e}"))?;

        let thumbnail = img.resize(200, 150, image::imageops::FilterType::Lanczos3);

        let file_path = thumbnails_dir.join(filename);
        thumbnail
            .save_with_format(&file_path, image::ImageFormat::Png)
            .map_err(|e| anyhow::anyhow!("保存缩略图失败: {e}"))?;

        Ok(filename.to_string())
    }

    pub fn list_device_cache(&self) -> anyhow::Result<Vec<DeviceRecord>> {
        load_json(&self.data_dir.join("devices.json"))
    }

    fn load_todos(&self) -> anyhow::Result<Vec<TodoRecord>> {
        let path = self.data_dir.join("todos.json");
        if !path.exists() {
            return Ok(Vec::new());
        }

        let text = std::fs::read_to_string(&path)?;
        let raw_items: Vec<serde_json::Value> = serde_json::from_str(&text)?;
        let mut todos = Vec::with_capacity(raw_items.len());
        let mut migrated = false;

        for item in raw_items {
            let todo = if has_local_id(&item) {
                let mut todo: TodoRecord = serde_json::from_value(item.clone())?;
                if todo.local_id.is_empty() {
                    todo.local_id = generate_local_todo_id();
                    migrated = true;
                }
                if let Some(remote_id) = legacy_remote_id(&item) {
                    if todo.id != Some(remote_id) {
                        todo.id = Some(remote_id);
                    }
                    if todo.last_synced_status.is_none() && !todo.dirty {
                        todo.last_synced_status = Some(todo.status);
                    }
                    migrated = true;
                } else if todo.id.is_some() && !has_sync_metadata(&item) && !todo.dirty {
                    todo.last_synced_status = Some(todo.status);
                    migrated = true;
                }
                todo
            } else {
                let legacy: LegacyTodoRecord = serde_json::from_value(item)?;
                migrated = true;
                todo_from_legacy(legacy)
            };
            todos.push(todo);
        }

        if migrated {
            save_json(&path, &todos)?;
        }

        Ok(todos)
    }

    pub fn create_local_todo(&self, input: TodoUpsertInput) -> anyhow::Result<TodoRecord> {
        let path = self.data_dir.join("todos.json");
        let mut todos = self.load_todos()?;
        let now = chrono::Utc::now().to_rfc3339();
        let record = TodoRecord {
            local_id: generate_local_todo_id(),
            id: None,
            title: input.title,
            description: input.description,
            due_date: input.due_date,
            due_time: input.due_time,
            repeat_type: input.repeat_type,
            repeat_weekday: input.repeat_weekday,
            repeat_month: input.repeat_month,
            repeat_day: input.repeat_day,
            status: 0,
            priority: input.priority.unwrap_or(0),
            device_id: input.device_id,
            dirty: true,
            deleted: false,
            created_at: now.clone(),
            updated_at: now,
            last_synced_status: None,
        };
        todos.push(record.clone());
        save_json(&path, &todos)?;
        Ok(record)
    }

    pub fn toggle_todo_status(&self, local_id: &str) -> anyhow::Result<TodoRecord> {
        let path = self.data_dir.join("todos.json");
        let mut todos = self.load_todos()?;
        let todo = todos
            .iter_mut()
            .find(|t| t.local_id == local_id)
            .ok_or_else(|| anyhow::anyhow!("待办 {local_id} 未找到"))?;
        todo.status = if todo.status == 0 { 1 } else { 0 };
        todo.dirty = true;
        todo.updated_at = chrono::Utc::now().to_rfc3339();
        let updated = todo.clone();
        save_json(&path, &todos)?;
        Ok(updated)
    }

    pub fn delete_local_todo(&self, local_id: &str) -> anyhow::Result<()> {
        let path = self.data_dir.join("todos.json");
        let mut todos = self.load_todos()?;

        if let Some(todo) = todos.iter_mut().find(|t| t.local_id == local_id) {
            if todo.id.is_none() {
                todos.retain(|t| t.local_id != local_id);
            } else {
                todo.deleted = true;
                todo.dirty = true;
                todo.updated_at = chrono::Utc::now().to_rfc3339();
            }
        } else {
            anyhow::bail!("待办 {local_id} 未找到");
        }

        save_json(&path, &todos)?;
        Ok(())
    }

    pub fn update_local_todo(
        &self,
        local_id: &str,
        input: TodoUpsertInput,
    ) -> anyhow::Result<TodoRecord> {
        let path = self.data_dir.join("todos.json");
        let mut todos = self.load_todos()?;
        let todo = todos
            .iter_mut()
            .find(|t| t.local_id == local_id)
            .ok_or_else(|| anyhow::anyhow!("待办 {local_id} 未找到"))?;
        todo.title = input.title;
        todo.description = input.description;
        todo.due_date = input.due_date;
        todo.due_time = input.due_time;
        todo.repeat_type = input.repeat_type;
        todo.repeat_weekday = input.repeat_weekday;
        todo.repeat_month = input.repeat_month;
        todo.repeat_day = input.repeat_day;
        if let Some(p) = input.priority {
            todo.priority = p;
        }
        todo.device_id = input.device_id;
        todo.dirty = true;
        todo.updated_at = chrono::Utc::now().to_rfc3339();
        let updated = todo.clone();
        save_json(&path, &todos)?;
        Ok(updated)
    }

    pub async fn push_todo_to_device(&self, local_id: &str, device_id: &str, page_id: Option<u32>) -> anyhow::Result<()> {
        let todos = self.load_todos()?;
        let todo = todos
            .iter()
            .find(|t| t.local_id == local_id)
            .ok_or_else(|| anyhow::anyhow!("待办 {local_id} 未找到"))?;

        let devices_path = self.data_dir.join("devices.json");
        let devices: Vec<DeviceRecord> = load_json(&devices_path)?;
        let device = devices
            .iter()
            .find(|d| d.device_id.eq_ignore_ascii_case(device_id))
            .ok_or_else(|| anyhow::anyhow!("设备 {device_id} 未找到"))?;

        let api_key = self.get_api_key_by_id(device.api_key_id)?;

        let mut body_parts: Vec<String> = Vec::new();
        if !todo.description.is_empty() {
            body_parts.push(todo.description.clone());
        }
        if let Some(ref d) = todo.due_date {
            let time_part = todo.due_time.as_deref().unwrap_or("");
            if time_part.is_empty() {
                body_parts.push(format!("截止: {d}"));
            } else {
                body_parts.push(format!("截止: {d} {time_part}"));
            }
        }
        let body = body_parts.join("\n");

        crate::api::client::push_structured_text(&api_key, device_id, &todo.title, &body, page_id).await
    }

    pub fn create_text_template(
        &self,
        input: TextTemplateInput,
    ) -> anyhow::Result<TextTemplateRecord> {
        let path = self.data_dir.join("text_templates.json");
        let mut templates: Vec<TextTemplateRecord> = load_json(&path)?;
        let next_id = templates.iter().map(|t| t.id).max().unwrap_or(0) + 1;
        let now = chrono::Utc::now().to_rfc3339();
        let record = TextTemplateRecord {
            id: next_id,
            title: input.title,
            content: input.content,
            created_at: now.clone(),
            updated_at: now,
        };
        templates.push(record.clone());
        save_json(&path, &templates)?;
        Ok(record)
    }

    pub async fn push_text_template(
        &self,
        template_id: i64,
        device_id: &str,
        page_id: Option<u32>,
    ) -> anyhow::Result<()> {
        let templates_path = self.data_dir.join("text_templates.json");
        let templates: Vec<TextTemplateRecord> = load_json(&templates_path)?;
        let template = templates
            .iter()
            .find(|t| t.id == template_id)
            .ok_or_else(|| anyhow::anyhow!("模板 {template_id} 未找到"))?;

        let devices_path = self.data_dir.join("devices.json");
        let devices: Vec<DeviceRecord> = load_json(&devices_path)?;
        let device = devices
            .iter()
            .find(|d| d.device_id.eq_ignore_ascii_case(device_id))
            .ok_or_else(|| anyhow::anyhow!("设备 {device_id} 未找到"))?;

        let api_key = self.get_api_key_by_id(device.api_key_id)?;
        crate::api::client::push_structured_text(
            &api_key,
            device_id,
            &template.title,
            &template.content,
            page_id,
        )
        .await
    }

    pub async fn push_text(
        &self,
        text: &str,
        font_size: Option<u32>,
        device_id: &str,
        page_id: Option<u32>,
    ) -> anyhow::Result<()> {
        let devices_path = self.data_dir.join("devices.json");
        let devices: Vec<DeviceRecord> = load_json(&devices_path)?;
        let device = devices
            .iter()
            .find(|d| d.device_id.eq_ignore_ascii_case(device_id))
            .ok_or_else(|| anyhow::anyhow!("设备 {device_id} 未找到"))?;

        let api_key = self.get_api_key_by_id(device.api_key_id)?;
        crate::api::client::push_text(&api_key, device_id, text, font_size, page_id).await?;

        // 推送成功后写入缓存
        if let Some(pid) = page_id {
            let now = chrono::Utc::now().to_rfc3339();
            let preview = if text.len() > 100 {
                text.chars().take(100).collect::<String>() + "..."
            } else {
                text.to_string()
            };

            let record = PageCacheRecord {
                device_id: device_id.to_string(),
                page_id: pid,
                content_type: "text".to_string(),
                thumbnail: Some(preview),
                metadata: Some(serde_json::json!({"fontSize": font_size.unwrap_or(20)}).to_string()),
                pushed_at: now,
            };
            self.save_page_cache_record(record)?;
        }

        Ok(())
    }

    pub async fn push_stock_quotes(&self, device_id: &str, page_id: u32) -> anyhow::Result<()> {
        let records = self.load_stock_watchlist()?;
        if records.is_empty() {
            anyhow::bail!("股票列表为空");
        }

        let codes = records
            .iter()
            .map(|record| record.code.clone())
            .collect::<Vec<_>>();
        let quotes = crate::stock_quote::fetch_eastmoney_quotes(&codes).await?;
        let text = crate::stock_quote::format_stock_push_text(&quotes, chrono::Local::now());

        self.push_text(&text, Some(20), device_id, Some(page_id)).await
    }

    pub async fn fetch_stock_quotes(&self) -> anyhow::Result<Vec<crate::stock_quote::StockQuote>> {
        let records = self.load_stock_watchlist()?;
        if records.is_empty() {
            return Ok(Vec::new());
        }

        let codes = records
            .iter()
            .map(|record| record.code.clone())
            .collect::<Vec<_>>();
        crate::stock_quote::fetch_eastmoney_quotes(&codes).await
    }

    // ==================== Stock Push Task Methods ====================

    fn load_stock_push_task(&self) -> anyhow::Result<Option<StockPushTaskRecord>> {
        let path = self.data_dir.join("stock_push_task.json");
        if !path.exists() {
            return Ok(None);
        }
        load_json(&path)
    }

    fn save_stock_push_task(&self, task: &StockPushTaskRecord) -> anyhow::Result<()> {
        let path = self.data_dir.join("stock_push_task.json");
        save_json(&path, task)
    }

    pub fn get_stock_push_task(&self) -> anyhow::Result<Option<StockPushTaskRecord>> {
        self.load_stock_push_task()
    }

    pub fn create_stock_push_task(
        &self,
        device_id: &str,
        page_id: u32,
        interval_seconds: u32,
    ) -> anyhow::Result<StockPushTaskRecord> {
        // Validate interval
        if ![30, 60, 300, 600].contains(&interval_seconds) {
            anyhow::bail!("间隔时间必须是 30、60、300 或 600 秒");
        }

        // Check if task already exists
        if let Some(existing) = self.load_stock_push_task()? {
            if existing.status == "running" {
                anyhow::bail!("已有正在运行的股票推送任务，请先停止");
            }
        }

        let now = chrono::Utc::now().to_rfc3339();
        let task = StockPushTaskRecord {
            id: 1,
            device_id: device_id.to_string(),
            page_id,
            interval_seconds,
            status: "stopped".to_string(),
            error_message: None,
            started_at: None,
            last_push_at: None,
            created_at: now.clone(),
            updated_at: now,
        };

        self.save_stock_push_task(&task)?;
        Ok(task)
    }

    pub fn start_stock_push_task(&self) -> anyhow::Result<StockPushTaskRecord> {
        let mut task = self
            .load_stock_push_task()?
            .ok_or_else(|| anyhow::anyhow!("请先创建股票推送任务"))?;

        // Check if watchlist has stocks
        let watchlist = self.load_stock_watchlist()?;
        if watchlist.is_empty() {
            anyhow::bail!("股票列表为空，请先添加股票代码");
        }

        if task.status == "running" {
            return Ok(task);
        }

        task.status = "running".to_string();
        task.error_message = None;
        task.started_at = Some(chrono::Utc::now().to_rfc3339());
        task.updated_at = chrono::Utc::now().to_rfc3339();

        let updated = task.clone();
        self.save_stock_push_task(&task)?;

        // Spawn background task
        let data_dir = self.data_dir.clone();
        let task_id = task.id;
        let interval_seconds = task.interval_seconds;

        tauri::async_runtime::spawn(async move {
            // First push immediately
            if let Err(e) = execute_stock_push(data_dir.clone(), task_id).await {
                eprintln!("[stock-push] 第一次推送失败: {}", e);
                return;
            }

            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(interval_seconds as u64)).await;

                // Check if task is still running
                let still_running = {
                    let task_opt: Option<StockPushTaskRecord> = match load_json(
                        &data_dir.join("stock_push_task.json")
                    ) {
                        Ok(t) => t,
                        Err(_) => break,
                    };
                    task_opt.map(|t| t.status == "running").unwrap_or(false)
                };

                if !still_running {
                    break;
                }

                if let Err(e) = execute_stock_push(data_dir.clone(), task_id).await {
                    eprintln!("[stock-push] 推送失败: {}", e);
                }
            }
        });

        Ok(updated)
    }

    pub fn stop_stock_push_task(&self) -> anyhow::Result<StockPushTaskRecord> {
        let mut task = self
            .load_stock_push_task()?
            .ok_or_else(|| anyhow::anyhow!("没有股票推送任务"))?;

        task.status = "stopped".to_string();
        task.updated_at = chrono::Utc::now().to_rfc3339();

        let updated = task.clone();
        self.save_stock_push_task(&task)?;
        Ok(updated)
    }

    /// Stop running stock push task on app startup
    pub fn stop_stock_push_task_on_boot(&self) -> anyhow::Result<()> {
        if let Some(mut task) = self.load_stock_push_task()? {
            if task.status == "running" {
                task.status = "stopped".to_string();
                task.updated_at = chrono::Utc::now().to_rfc3339();
                self.save_stock_push_task(&task)?;
            }
        }
        Ok(())
    }

    pub async fn push_structured_text(
        &self,
        title: &str,
        body: &str,
        device_id: &str,
        page_id: Option<u32>,
    ) -> anyhow::Result<()> {
        let devices_path = self.data_dir.join("devices.json");
        let devices: Vec<DeviceRecord> = load_json(&devices_path)?;
        let device = devices
            .iter()
            .find(|d| d.device_id.eq_ignore_ascii_case(device_id))
            .ok_or_else(|| anyhow::anyhow!("设备 {device_id} 未找到"))?;

        let api_key = self.get_api_key_by_id(device.api_key_id)?;
        crate::api::client::push_structured_text(&api_key, device_id, title, body, page_id).await?;

        // 推送成功后写入缓存
        if let Some(pid) = page_id {
            let now = chrono::Utc::now().to_rfc3339();
            let body_preview = if body.len() > 100 {
                body.chars().take(100).collect::<String>() + "..."
            } else {
                body.to_string()
            };

            let thumbnail = if title.is_empty() {
                body_preview.clone()
            } else if body_preview.is_empty() {
                title.to_string()
            } else {
                format!("{}\n{}", title, body_preview)
            };

            let record = PageCacheRecord {
                device_id: device_id.to_string(),
                page_id: pid,
                content_type: "structured_text".to_string(),
                thumbnail: Some(thumbnail),
                metadata: Some(serde_json::json!({"title": title, "bodyPreview": body_preview}).to_string()),
                pushed_at: now,
            };
            self.save_page_cache_record(record)?;
        }

        Ok(())
    }

    pub async fn sync_all(&self) -> anyhow::Result<BootstrapState> {
        let api_keys: Vec<ApiKeyRecord> = load_json(&self.data_dir.join("api_keys.json"))?;
        if api_keys.is_empty() {
            anyhow::bail!("未配置 API Key，请先在设置中添加");
        }

        // 使用第一个 API Key 进行同步
        let api_key = api_keys[0].key.clone();
        let primary_api_key_id = api_keys[0].id;

        // 1. Upload dirty local todos
        let todos_path = self.data_dir.join("todos.json");
        let mut local_todos = self.load_todos()?;
        let mut synced_cloud_todos = HashMap::new();
        let mut deleted_remote_ids = HashSet::new();
        for todo in local_todos.clone() {
            if !todo.dirty {
                continue;
            }
            if todo.deleted {
                if let Some(id) = todo.id {
                    if crate::api::client::delete_cloud_todo(&api_key, id)
                        .await
                        .is_ok()
                    {
                        deleted_remote_ids.insert(id);
                        mark_remote_delete_synced(&mut local_todos, &todo.local_id);
                    }
                } else {
                    mark_remote_delete_synced(&mut local_todos, &todo.local_id);
                }
                continue;
            }
            let body = todo_upload_body(&todo);
            let plan = plan_remote_sync(&todo);
            if let Some(id) = todo.id {
                if let Ok(updated) =
                    crate::api::client::update_cloud_todo(&api_key, id, &body).await
                {
                    if plan.uses_complete_endpoint
                        && crate::api::client::complete_cloud_todo(&api_key, id)
                            .await
                            .is_err()
                    {
                        continue;
                    }
                    let mut synced_remote = apply_local_state_to_api_todo(updated, &todo);
                    if plan.uses_complete_endpoint {
                        synced_remote.status = 1;
                        synced_remote.completed = true;
                    }
                    mark_local_todo_synced(&mut local_todos, &todo.local_id, id, todo.status);
                    synced_cloud_todos.insert(id, synced_remote);
                }
            } else {
                if let Ok(created) = crate::api::client::create_cloud_todo(&api_key, &body).await {
                    let synced_remote = apply_local_state_to_api_todo(created, &todo);
                    mark_local_todo_synced(
                        &mut local_todos,
                        &todo.local_id,
                        synced_remote.id,
                        todo.status,
                    );
                    synced_cloud_todos.insert(synced_remote.id, synced_remote);
                }
            }
        }

        save_json(&todos_path, &local_todos)?;

        // 2. Fetch devices from cloud
        let cloud_devices = crate::api::client::fetch_devices(&api_key).await?;
        let devices: Vec<DeviceRecord> = cloud_devices
            .into_iter()
            .map(|d| DeviceRecord {
                device_id: d.device_id,
                alias: d.alias,
                board: d.board,
                cached_at: chrono::Utc::now().to_rfc3339(),
                api_key_id: primary_api_key_id,
            })
            .collect();
        save_json(&self.data_dir.join("devices.json"), &devices)?;

        // 3. Fetch todos from cloud
        let cloud_todos = crate::api::client::fetch_todos(&api_key).await?;
        let now = chrono::Utc::now().to_rfc3339();
        let todos = merge_cloud_todos(
            &local_todos,
            cloud_todos,
            synced_cloud_todos,
            deleted_remote_ids,
            &now,
        );
        save_json(&todos_path, &todos)?;

        // 4. Write last_sync_time
        let config = AppConfig {
            last_sync_time: Some(now),
        };
        save_json(&self.data_dir.join("config.json"), &config)?;

        // 5. Return refreshed bootstrap state
        self.load_bootstrap_state()
    }

    fn load_image_source(
        &self,
        source_path: Option<&str>,
        source_data_url: Option<&str>,
    ) -> anyhow::Result<image::DynamicImage> {
        if let Some(data_url) = source_data_url {
            let payload = data_url
                .split_once(',')
                .map(|(_, payload)| payload)
                .ok_or_else(|| anyhow::anyhow!("图片数据格式错误"))?;
            use base64::Engine;
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(payload)
                .map_err(|e| anyhow::anyhow!("图片解码失败: {e}"))?;
            return image::load_from_memory(&bytes)
                .map_err(|e| anyhow::anyhow!("无法读取导入的图片: {e}"));
        }

        let source_path = source_path
            .filter(|path| !path.is_empty())
            .ok_or_else(|| anyhow::anyhow!("请选择图片"))?;
        let path = std::path::Path::new(source_path);
        let img = image::open(path).map_err(|e| anyhow::anyhow!("无法打开图片: {e}"))?;
        // Apply EXIF orientation if present (e.g., portrait photos from phones)
        Ok(apply_exif_orientation(img, path))
    }

    fn process_image(
        &self,
        source_path: Option<&str>,
        source_data_url: Option<&str>,
        crop: &CropRect,
        rotation: u32,
        flip_x: bool,
        flip_y: bool,
    ) -> anyhow::Result<image::DynamicImage> {
        let mut img = self.load_image_source(source_path, source_data_url)?;

        // Crop
        let (iw, ih) = img.dimensions();
        let cx = crop.x.min(iw.saturating_sub(1));
        let cy = crop.y.min(ih.saturating_sub(1));
        let cw = crop.width.min(iw - cx);
        let ch = crop.height.min(ih - cy);
        if cw > 0 && ch > 0 {
            img = img.crop_imm(cx, cy, cw, ch);
        }

        // Rotation
        img = match rotation % 360 {
            90 => img.rotate90(),
            180 => img.rotate180(),
            270 => img.rotate270(),
            _ => img,
        };

        // Flip
        if flip_x {
            img = img.fliph();
        }
        if flip_y {
            img = img.flipv();
        }

        // Resize to 400x300 without distortion (crop excess)
        img = img.resize_to_fill(400, 300, image::imageops::FilterType::Lanczos3);

        Ok(img)
    }

    pub fn render_image_preview(&self, input: ImageEditInput) -> anyhow::Result<String> {
        let img = self.process_image(
            input.source_path.as_deref(),
            input.source_data_url.as_deref(),
            &input.crop,
            input.rotation,
            input.flip_x,
            input.flip_y,
        )?;

        let mut buf = std::io::Cursor::new(Vec::new());
        img.write_to(&mut buf, image::ImageFormat::Png)
            .map_err(|e| anyhow::anyhow!("编码 PNG 失败: {e}"))?;

        let bytes = buf.into_inner();
        if bytes.len() > 2 * 1024 * 1024 {
            anyhow::bail!("图片超过 2MB 限制");
        }

        use base64::Engine;
        Ok(base64::engine::general_purpose::STANDARD.encode(&bytes))
    }

    pub fn save_image_template(
        &self,
        input: ImageTemplateSaveInput,
    ) -> anyhow::Result<ImageTemplateRecord> {
        let img = self.process_image(
            input.source_path.as_deref(),
            input.source_data_url.as_deref(),
            &input.crop,
            input.rotation,
            input.flip_x,
            input.flip_y,
        )?;

        let images_dir = self.data_dir.join("images");
        std::fs::create_dir_all(&images_dir)?;

        let path = self.data_dir.join("image_templates.json");
        let mut templates: Vec<ImageTemplateRecord> = load_json(&path)?;
        let next_id = templates.iter().map(|t| t.id).max().unwrap_or(0) + 1;

        let file_path = images_dir.join(format!("{next_id}.png"));
        img.save_with_format(&file_path, image::ImageFormat::Png)
            .map_err(|e| anyhow::anyhow!("保存图片失败: {e}"))?;

        let file_size = std::fs::metadata(&file_path)?.len();
        if file_size > 2 * 1024 * 1024 {
            std::fs::remove_file(&file_path)?;
            anyhow::bail!("图片超过 2MB 限制");
        }

        let now = chrono::Utc::now().to_rfc3339();
        let record = ImageTemplateRecord {
            id: next_id,
            name: input.name,
            file_path: file_path.to_string_lossy().to_string(),
            created_at: now,
        };
        templates.push(record.clone());
        save_json(&path, &templates)?;
        Ok(record)
    }

    pub async fn push_image_template(
        &self,
        template_id: i64,
        device_id: &str,
        page_id: u32,
    ) -> anyhow::Result<()> {
        let templates_path = self.data_dir.join("image_templates.json");
        let templates: Vec<ImageTemplateRecord> = load_json(&templates_path)?;
        let template = templates
            .iter()
            .find(|t| t.id == template_id)
            .ok_or_else(|| anyhow::anyhow!("图片模板 {template_id} 未找到"))?;

        let devices_path = self.data_dir.join("devices.json");
        let devices: Vec<DeviceRecord> = load_json(&devices_path)?;
        let device = devices
            .iter()
            .find(|d| d.device_id.eq_ignore_ascii_case(device_id))
            .ok_or_else(|| anyhow::anyhow!("设备 {device_id} 未找到"))?;

        let image_bytes = std::fs::read(&template.file_path)?;
        let api_key = self.get_api_key_by_id(device.api_key_id)?;
        crate::api::client::push_image(&api_key, device_id, image_bytes.clone(), page_id).await?;

        // 推送成功后写入缓存
        let now = chrono::Utc::now().to_rfc3339();
        let thumbnail_filename = format!("thumb_{}_{}.png",
            device_id.replace(':', "_"),
            page_id
        );
        let thumbnail = self.save_image_thumbnail(&image_bytes, &thumbnail_filename)?;

        let record = PageCacheRecord {
            device_id: device_id.to_string(),
            page_id,
            content_type: "image".to_string(),
            thumbnail: Some(thumbnail),
            metadata: Some(serde_json::json!({"width": 400, "height": 300}).to_string()),
            pushed_at: now,
        };
        self.save_page_cache_record(record)?;

        Ok(())
    }

    pub fn get_image_thumbnail(&self, template_id: i64) -> anyhow::Result<String> {
        let templates_path = self.data_dir.join("image_templates.json");
        let templates: Vec<ImageTemplateRecord> = load_json(&templates_path)?;
        let template = templates
            .iter()
            .find(|t| t.id == template_id)
            .ok_or_else(|| anyhow::anyhow!("图片模板 {template_id} 未找到"))?;

        let img = image::open(&template.file_path)
            .map_err(|e| anyhow::anyhow!("无法打开图片: {e}"))?;

        // 缩略图与原图尺寸相同 (400x300)，避免显示模糊
        let thumbnail = img.resize(400, 300, image::imageops::FilterType::Lanczos3);

        let mut buf = std::io::Cursor::new(Vec::new());
        thumbnail
            .write_to(&mut buf, image::ImageFormat::Png)
            .map_err(|e| anyhow::anyhow!("编码 PNG 失败: {e}"))?;

        use base64::Engine;
        Ok(base64::engine::general_purpose::STANDARD.encode(&buf.into_inner()))
    }

    pub fn delete_image_template(&self, template_id: i64) -> anyhow::Result<()> {
        let templates_path = self.data_dir.join("image_templates.json");
        let mut templates: Vec<ImageTemplateRecord> = load_json(&templates_path)?;
        let template = templates
            .iter()
            .find(|t| t.id == template_id)
            .ok_or_else(|| anyhow::anyhow!("图片模板 {template_id} 未找到"))?;

        // 删除图片文件
        if std::fs::metadata(&template.file_path).is_ok() {
            std::fs::remove_file(&template.file_path)?;
        }

        // 从 JSON 中移除记录
        templates.retain(|t| t.id != template_id);
        save_json(&templates_path, &templates)?;
        Ok(())
    }

    pub async fn push_sketch(
        &self,
        data_url: &str,
        device_id: &str,
        page_id: u32,
    ) -> anyhow::Result<()> {
        let devices_path = self.data_dir.join("devices.json");
        let devices: Vec<DeviceRecord> = load_json(&devices_path)?;
        let device = devices
            .iter()
            .find(|d| d.device_id.eq_ignore_ascii_case(device_id))
            .ok_or_else(|| anyhow::anyhow!("设备 {device_id} 未找到"))?;

        // 解码 data URL
        let payload = data_url
            .split_once(',')
            .map(|(_, payload)| payload)
            .ok_or_else(|| anyhow::anyhow!("图片数据格式错误"))?;
        use base64::Engine;
        let image_bytes = base64::engine::general_purpose::STANDARD
            .decode(payload)
            .map_err(|e| anyhow::anyhow!("图片解码失败: {e}"))?;

        let api_key = self.get_api_key_by_id(device.api_key_id)?;
        crate::api::client::push_image(&api_key, device_id, image_bytes.clone(), page_id).await?;

        // 推送成功后写入缓存
        let now = chrono::Utc::now().to_rfc3339();
        let thumbnail_filename = format!("thumb_{}_{}.png",
            device_id.replace(':', "_"),
            page_id
        );
        let thumbnail = self.save_image_thumbnail(&image_bytes, &thumbnail_filename)?;

        let record = PageCacheRecord {
            device_id: device_id.to_string(),
            page_id,
            content_type: "sketch".to_string(),
            thumbnail: Some(thumbnail),
            metadata: Some(serde_json::json!({"width": 400, "height": 300}).to_string()),
            pushed_at: now,
        };
        self.save_page_cache_record(record)?;

        Ok(())
    }

    pub fn scan_image_folder(&self, folder_path: &str) -> anyhow::Result<crate::models::ImageFolderScanResult> {
        let path = std::path::Path::new(folder_path);
        if !path.exists() || !path.is_dir() {
            anyhow::bail!("文件夹不存在或不是有效目录");
        }

        let image_extensions = ["jpg", "jpeg", "png", "gif", "bmp", "webp"];
        let mut image_files: Vec<String> = Vec::new();

        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let file_name = entry.file_name().to_string_lossy().to_string();
            let ext = entry.path()
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase());

            if let Some(e) = ext {
                if image_extensions.contains(&e.as_str()) {
                    image_files.push(file_name);
                }
            }
        }

        image_files.sort();

        let warning = if image_files.is_empty() {
            Some("该文件夹未检测到图片".to_string())
        } else {
            None
        };

        Ok(crate::models::ImageFolderScanResult {
            total_images: image_files.len() as u32,
            image_files,
            warning,
        })
    }

    fn load_image_loop_tasks(&self) -> anyhow::Result<Vec<crate::models::ImageLoopTaskRecord>> {
        let path = self.data_dir.join("image_loop_tasks.json");
        if !path.exists() {
            return Ok(Vec::new());
        }
        crate::storage::load_json(&path)
    }

    fn save_image_loop_tasks(&self, tasks: &Vec<crate::models::ImageLoopTaskRecord>) -> anyhow::Result<()> {
        let path = self.data_dir.join("image_loop_tasks.json");
        crate::storage::save_json(&path, tasks)
    }

    /// Stop all running image loop tasks — called on app startup
    pub fn stop_all_image_loop_tasks_on_boot(&self) -> anyhow::Result<()> {
        let mut tasks = self.load_image_loop_tasks()?;
        let mut changed = false;
        for task in &mut tasks {
            if task.status == "running" {
                task.status = "idle".to_string();
                task.started_at = None;
                task.updated_at = chrono::Utc::now().to_rfc3339();
                changed = true;
            }
        }
        if changed {
            self.save_image_loop_tasks(&tasks)?;
        }
        Ok(())
    }

    pub fn list_image_loop_tasks(&self) -> anyhow::Result<Vec<crate::models::ImageLoopTaskRecord>> {
        self.load_image_loop_tasks()
    }

    pub fn create_image_loop_task(
        &self,
        input: crate::models::ImageLoopTaskInput,
    ) -> anyhow::Result<crate::models::ImageLoopTaskRecord> {
        let mut tasks = self.load_image_loop_tasks()?;
        let next_id = tasks.iter().map(|t| t.id).max().unwrap_or(0) + 1;
        let now = chrono::Utc::now().to_rfc3339();

        let scan_result = self.scan_image_folder(&input.folder_path)?;

        let record = crate::models::ImageLoopTaskRecord {
            id: next_id,
            name: input.name,
            folder_path: input.folder_path,
            device_id: input.device_id,
            page_id: input.page_id,
            interval_seconds: input.interval_seconds,
            duration_type: input.duration_type,
            end_time: input.end_time,
            duration_minutes: input.duration_minutes,
            status: "idle".to_string(),
            current_index: 0,
            total_images: scan_result.total_images,
            started_at: None,
            last_push_at: None,
            error_message: None,
            created_at: now.clone(),
            updated_at: now,
        };

        tasks.push(record.clone());
        self.save_image_loop_tasks(&tasks)?;
        Ok(record)
    }

    pub fn update_image_loop_task(
        &self,
        task_id: i64,
        input: crate::models::ImageLoopTaskInput,
    ) -> anyhow::Result<crate::models::ImageLoopTaskRecord> {
        let mut tasks = self.load_image_loop_tasks()?;
        let task = tasks
            .iter_mut()
            .find(|t| t.id == task_id)
            .ok_or_else(|| anyhow::anyhow!("任务 {task_id} 未找到"))?;

        let scan_result = self.scan_image_folder(&input.folder_path)?;

        task.name = input.name;
        task.folder_path = input.folder_path;
        task.device_id = input.device_id;
        task.page_id = input.page_id;
        task.interval_seconds = input.interval_seconds;
        task.duration_type = input.duration_type;
        task.end_time = input.end_time;
        task.duration_minutes = input.duration_minutes;
        task.total_images = scan_result.total_images;
        task.updated_at = chrono::Utc::now().to_rfc3339();

        let updated = task.clone();
        self.save_image_loop_tasks(&tasks)?;
        Ok(updated)
    }

    pub fn delete_image_loop_task(&self, task_id: i64) -> anyhow::Result<()> {
        let mut tasks = self.load_image_loop_tasks()?;
        tasks.retain(|t| t.id != task_id);
        self.save_image_loop_tasks(&tasks)?;
        Ok(())
    }

    pub fn start_image_loop_task(&self, task_id: i64) -> anyhow::Result<crate::models::ImageLoopTaskRecord> {
        let mut tasks = self.load_image_loop_tasks()?;
        let task = tasks
            .iter_mut()
            .find(|t| t.id == task_id)
            .ok_or_else(|| anyhow::anyhow!("任务 {task_id} 未找到"))?;

        if task.total_images == 0 {
            anyhow::bail!("文件夹中没有图片，无法启动");
        }

        // If task is already running, no need to spawn again
        if task.status == "running" {
            return Ok(task.clone());
        }

        task.status = "running".to_string();
        task.current_index = 0;
        task.started_at = Some(chrono::Utc::now().to_rfc3339());
        task.error_message = None;
        task.updated_at = chrono::Utc::now().to_rfc3339();

        let interval_seconds = task.interval_seconds;
        let updated = task.clone();
        self.save_image_loop_tasks(&tasks)?;

        // Spawn background task for periodic image pushing using Tauri's async runtime
        let data_dir = self.data_dir.clone();
        tauri::async_runtime::spawn(async move {
            // Push first image immediately on start
            let first_result = execute_image_loop_push(data_dir.clone(), task_id).await;
            if let Err(e) = first_result {
                eprintln!("[image-loop] 任务 {} 第一次推送失败: {}", task_id, e);
                return;
            }

            loop {
                // Sleep for the interval duration
                tokio::time::sleep(tokio::time::Duration::from_secs(interval_seconds as u64)).await;

                // Check if task is still running before executing
                let still_running = {
                    let tasks: Vec<crate::models::ImageLoopTaskRecord> = match crate::storage::load_json(
                        &data_dir.join("image_loop_tasks.json")
                    ) {
                        Ok(t) => t,
                        Err(_) => break,
                    };
                    tasks.iter().find(|t| t.id == task_id).map(|t| t.status == "running").unwrap_or(false)
                };

                if !still_running {
                    break;
                }

                // Execute push and check if task should stop
                let result = execute_image_loop_push(data_dir.clone(), task_id).await;
                match result {
                    Ok(updated_task) => {
                        // Check if task completed
                        if updated_task.status != "running" {
                            break;
                        }
                    }
                    Err(e) => {
                        // Log error and stop task
                        eprintln!("[image-loop] 任务 {} 推送失败: {}", task_id, e);
                        break;
                    }
                }
            }
        });

        Ok(updated)
    }

    pub fn stop_image_loop_task(&self, task_id: i64) -> anyhow::Result<crate::models::ImageLoopTaskRecord> {
        let mut tasks = self.load_image_loop_tasks()?;
        let task = tasks
            .iter_mut()
            .find(|t| t.id == task_id)
            .ok_or_else(|| anyhow::anyhow!("任务 {task_id} 未找到"))?;

        task.status = "idle".to_string();
        task.started_at = None;
        task.updated_at = chrono::Utc::now().to_rfc3339();

        let updated = task.clone();
        self.save_image_loop_tasks(&tasks)?;
        Ok(updated)
    }

    pub async fn push_folder_image(&self, task_id: i64) -> anyhow::Result<crate::models::ImageLoopTaskRecord> {
        let mut tasks = self.load_image_loop_tasks()?;
        let task_idx = tasks
            .iter()
            .position(|t| t.id == task_id)
            .ok_or_else(|| anyhow::anyhow!("任务 {task_id} 未找到"))?;

        if tasks[task_idx].status != "running" {
            anyhow::bail!("任务未在运行中");
        }

        // 获取设备信息
        let devices: Vec<crate::models::DeviceRecord> = crate::storage::load_json(
            &self.data_dir.join("devices.json"),
        )?;
        let device = devices
            .iter()
            .find(|d| d.device_id.eq_ignore_ascii_case(&tasks[task_idx].device_id))
            .ok_or_else(|| anyhow::anyhow!("设备 {} 未找到", tasks[task_idx].device_id))?;

        let api_key = self.get_api_key_by_id(device.api_key_id)?;

        // 获取图片列表
        let scan_result = self.scan_image_folder(&tasks[task_idx].folder_path)?;
        if scan_result.image_files.is_empty() {
            tasks[task_idx].status = "completed".to_string();
            tasks[task_idx].error_message = Some("文件夹中没有图片".to_string());
            tasks[task_idx].updated_at = chrono::Utc::now().to_rfc3339();
            let updated = tasks[task_idx].clone();
            self.save_image_loop_tasks(&tasks)?;
            return Ok(updated);
        }

        // 获取当前图片路径
        let image_file = &scan_result.image_files[tasks[task_idx].current_index as usize];
        let image_path = std::path::Path::new(&tasks[task_idx].folder_path).join(image_file);

        // 加载并处理图片为 400x300
        let img = image::open(&image_path)
            .map_err(|e| anyhow::anyhow!("无法打开图片 {}: {}", image_file, e))?;
        // Apply EXIF orientation if present
        let img = apply_exif_orientation(img, &image_path);
        let processed = img.resize_to_fill(400, 300, image::imageops::FilterType::Lanczos3);

        // 编码为 PNG
        let mut buf = std::io::Cursor::new(Vec::new());
        processed
            .write_to(&mut buf, image::ImageFormat::Png)
            .map_err(|e| anyhow::anyhow!("编码 PNG 失败: {}", e))?;
        let image_bytes = buf.into_inner();

        // 推送图片
        let page_id = tasks[task_idx].page_id;
        let device_id = tasks[task_idx].device_id.clone();
        if let Err(e) = crate::api::client::push_image(&api_key, &device_id, image_bytes.clone(), page_id).await {
            tasks[task_idx].status = "error".to_string();
            tasks[task_idx].error_message = Some(e.to_string());
            tasks[task_idx].updated_at = chrono::Utc::now().to_rfc3339();
            self.save_image_loop_tasks(&tasks)?;
            anyhow::bail!("推送失败: {}", e);
        }

        // 更新索引
        tasks[task_idx].current_index = (tasks[task_idx].current_index + 1) % scan_result.total_images;
        tasks[task_idx].last_push_at = Some(chrono::Utc::now().to_rfc3339());
        tasks[task_idx].updated_at = chrono::Utc::now().to_rfc3339();

        // 检查持续时间条件
        let should_complete = self.check_duration_condition(&tasks[task_idx])?;
        if should_complete {
            tasks[task_idx].status = "completed".to_string();
        }

        let updated = tasks[task_idx].clone();
        self.save_image_loop_tasks(&tasks)?;
        Ok(updated)
    }

    pub fn check_duration_condition(&self, task: &crate::models::ImageLoopTaskRecord) -> anyhow::Result<bool> {
        if task.duration_type == "none" {
            return Ok(false);
        }

        let started_at = task
            .started_at
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("任务缺少启动时间"))?;

        let started = chrono::DateTime::parse_from_rfc3339(started_at)?
            .with_timezone(&chrono::Utc);
        let now = chrono::Utc::now();

        if task.duration_type == "until_time" {
            if let Some(end_time) = &task.end_time {
                let parts: Vec<&str> = end_time.split(':').collect();
                if parts.len() == 2 {
                    let hour: u32 = parts[0].parse().unwrap_or(0);
                    let minute: u32 = parts[1].parse().unwrap_or(0);

                    let end_datetime = now
                        .date_naive()
                        .and_hms_opt(hour, minute, 0)
                        .map(|dt| chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(dt, chrono::Utc));

                    if let Some(end) = end_datetime {
                        if now >= end {
                            return Ok(true);
                        }
                    }
                }
            }
        } else if task.duration_type == "for_duration" {
            if let Some(minutes) = task.duration_minutes {
                let elapsed = now - started;
                if elapsed.num_minutes() >= minutes as i64 {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::client::ApiTodo;
    use crate::storage::save_json;

    fn make_local_todo(local_id: &str, id: Option<i64>, status: i32, dirty: bool) -> TodoRecord {
        TodoRecord {
            local_id: local_id.into(),
            id,
            title: "Buy milk".into(),
            description: "".into(),
            due_date: None,
            due_time: None,
            repeat_type: None,
            repeat_weekday: None,
            repeat_month: None,
            repeat_day: None,
            status,
            priority: 1,
            device_id: None,
            dirty,
            deleted: false,
            created_at: "2026-04-23T09:00:00Z".into(),
            updated_at: "2026-04-23T09:00:00Z".into(),
            last_synced_status: id.map(|_| status),
        }
    }

    fn make_api_todo(id: i64, status: i32) -> ApiTodo {
        ApiTodo {
            id,
            title: "Buy milk".into(),
            description: "".into(),
            due_date: None,
            due_time: None,
            repeat_type: None,
            repeat_weekday: None,
            repeat_month: None,
            repeat_day: None,
            status,
            priority: 1,
            completed: status == 1,
            device_id: None,
            device_name: None,
            create_date: Some("2026-04-23T09:00:00Z".into()),
            update_date: None,
        }
    }

    fn test_state(dir: &tempfile::TempDir) -> AppState {
        AppState {
            data_dir: dir.path().to_path_buf(),
        }
    }

    #[test]
    fn create_local_todo_uses_local_id_and_null_remote_id() {
        let dir = tempfile::tempdir().unwrap();
        let state = test_state(&dir);
        let todo = state
            .create_local_todo(TodoUpsertInput {
                title: "Buy milk".into(),
                description: "".into(),
                due_date: None,
                due_time: None,
                repeat_type: None,
                repeat_weekday: None,
                repeat_month: None,
                repeat_day: None,
                priority: Some(1),
                device_id: None,
            })
            .unwrap();

        assert!(!todo.local_id.is_empty());
        assert_eq!(todo.id, None);
        assert!(todo.dirty);
        assert_eq!(todo.last_synced_status, None);
    }

    #[test]
    fn toggle_todo_status_updates_by_local_id() {
        let dir = tempfile::tempdir().unwrap();
        let state = test_state(&dir);
        let path = dir.path().join("todos.json");
        save_json(
            &path,
            &vec![TodoRecord {
                local_id: "todo-local-1".into(),
                id: None,
                title: "Buy milk".into(),
                description: "".into(),
                due_date: None,
                due_time: None,
                repeat_type: None,
                repeat_weekday: None,
                repeat_month: None,
                repeat_day: None,
                status: 0,
                priority: 1,
                device_id: None,
                dirty: false,
                deleted: false,
                created_at: "2026-04-23T09:00:00Z".into(),
                updated_at: "2026-04-23T09:00:00Z".into(),
                last_synced_status: None,
            }],
        )
        .unwrap();

        let updated = state.toggle_todo_status("todo-local-1").unwrap();

        assert_eq!(updated.local_id, "todo-local-1");
        assert_eq!(updated.status, 1);
        assert!(updated.dirty);
    }

    #[test]
    fn loading_legacy_todo_json_migrates_cloud_id_and_generates_local_id() {
        let dir = tempfile::tempdir().unwrap();
        let state = test_state(&dir);
        std::fs::write(
            dir.path().join("todos.json"),
            r#"[{"id":7,"cloud_id":42,"title":"Buy milk","description":"","due_date":null,"due_time":null,"status":0,"priority":1,"device_id":null,"dirty":false,"deleted":false,"created_at":"2026-04-23T09:00:00Z","updated_at":"2026-04-23T09:00:00Z"},{"id":8,"title":"Read book","description":"","due_date":null,"due_time":null,"status":0,"priority":1,"device_id":null,"dirty":false,"deleted":false,"created_at":"2026-04-23T09:10:00Z","updated_at":"2026-04-23T09:10:00Z"}]"#,
        )
        .unwrap();

        let bootstrap = state.load_bootstrap_state().unwrap();
        let todo = &bootstrap.todos[0];
        let todo_without_cloud_id = &bootstrap.todos[1];

        assert!(!todo.local_id.is_empty());
        assert_eq!(todo.id, Some(42));
        assert_eq!(todo.title, "Buy milk");
        assert_eq!(todo.last_synced_status, Some(0));
        assert!(!todo_without_cloud_id.local_id.is_empty());
        assert_eq!(todo_without_cloud_id.id, None);
        assert_eq!(todo_without_cloud_id.title, "Read book");
        assert_eq!(todo_without_cloud_id.last_synced_status, None);
    }

    #[test]
    fn load_bootstrap_state_includes_stock_watchlist() {
        let temp = tempfile::tempdir().unwrap();
        let state = test_state(&temp);

        save_json(
            &temp.path().join("stock_watchlist.json"),
            &vec![StockWatchRecord {
                code: "600519".to_string(),
                created_at: "2026-04-25T10:30:00Z".to_string(),
            }],
        )
        .unwrap();

        let bootstrap = state.load_bootstrap_state().unwrap();

        assert_eq!(bootstrap.stock_watchlist.len(), 1);
        assert_eq!(bootstrap.stock_watchlist[0].code, "600519");
    }

    #[test]
    fn adds_and_removes_stock_watch_records() {
        let temp = tempfile::tempdir().unwrap();
        let state = test_state(&temp);

        let created = state.add_stock_watch("600519").unwrap();
        assert_eq!(created.code, "600519");

        let list = state.list_stock_watchlist().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].code, "600519");

        let duplicate = state.add_stock_watch("600519").unwrap_err().to_string();
        assert!(duplicate.contains("已存在"));

        state.remove_stock_watch("600519").unwrap();
        assert!(state.list_stock_watchlist().unwrap().is_empty());
    }

    #[test]
    fn rejects_invalid_stock_watch_code() {
        let temp = tempfile::tempdir().unwrap();
        let state = test_state(&temp);

        let error = state.add_stock_watch("abc").unwrap_err().to_string();

        assert!(error.contains("6 位数字"));
    }

    #[test]
    fn accepts_any_six_digit_stock_code() {
        let temp = tempfile::tempdir().unwrap();
        let state = test_state(&temp);

        // 任意 6 位数字都可以
        let created = state.add_stock_watch("830000").unwrap();
        assert_eq!(created.code, "830000");
    }

    #[test]
    fn sync_all_backfills_remote_id_after_cloud_create() {
        let mut local_todos = vec![make_local_todo("local-a", None, 0, true)];

        mark_local_todo_synced(&mut local_todos, "local-a", 42, 0);

        assert_eq!(local_todos[0].id, Some(42));
        assert!(!local_todos[0].dirty);
        assert_eq!(local_todos[0].last_synced_status, Some(0));
    }

    #[test]
    fn sync_all_reuses_existing_local_id_for_matching_remote_id() {
        let local_todos = vec![make_local_todo("local-a", Some(42), 0, false)];
        let cloud_todos = vec![make_api_todo(42, 0)];

        let merged = merge_cloud_todos(
            &local_todos,
            cloud_todos,
            HashMap::new(),
            HashSet::new(),
            "2026-04-23T10:00:00Z",
        );

        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].local_id, "local-a");
        assert_eq!(merged[0].id, Some(42));
        assert!(!merged[0].dirty);
        assert_eq!(merged[0].last_synced_status, Some(0));
    }

    #[test]
    fn sync_all_generates_local_id_for_unknown_cloud_todo() {
        let local_todos = vec![];
        let cloud_todos = vec![make_api_todo(99, 0)];

        let merged = merge_cloud_todos(
            &local_todos,
            cloud_todos,
            HashMap::new(),
            HashSet::new(),
            "2026-04-23T10:00:00Z",
        );

        assert_eq!(merged.len(), 1);
        assert!(!merged[0].local_id.is_empty());
        assert_eq!(merged[0].id, Some(99));
        assert_eq!(merged[0].last_synced_status, Some(0));
    }

    #[test]
    fn merge_cloud_todos_keeps_unsynced_local_records_when_cloud_is_missing_them() {
        let local_todos = vec![make_local_todo("local-a", None, 1, true)];

        let merged = merge_cloud_todos(
            &local_todos,
            vec![],
            HashMap::new(),
            HashSet::new(),
            "2026-04-23T10:00:00Z",
        );

        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].local_id, "local-a");
        assert_eq!(merged[0].id, None);
        assert_eq!(merged[0].status, 1);
    }

    #[test]
    fn merge_cloud_todos_keeps_dirty_local_record_when_cloud_has_stale_match() {
        let mut local = make_local_todo("local-a", Some(42), 0, true);
        local.title = "Buy milk locally".into();
        local.last_synced_status = Some(1);
        let local_todos = vec![local];
        let mut cloud = make_api_todo(42, 1);
        cloud.title = "Cloud title".into();
        cloud.description = "cloud".into();
        cloud.priority = 3;

        let merged = merge_cloud_todos(
            &local_todos,
            vec![cloud],
            HashMap::new(),
            HashSet::new(),
            "2026-04-23T10:00:00Z",
        );

        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].local_id, "local-a");
        assert_eq!(merged[0].title, "Buy milk locally");
        assert_eq!(merged[0].status, 0);
        assert_eq!(merged[0].id, Some(42));
    }

    #[test]
    fn unsynced_completed_todo_upload_body_carries_completed_status() {
        let todo = make_local_todo("local-a", None, 1, true);

        let body = todo_upload_body(&todo);

        assert_eq!(body.title, "Buy milk");
    }

    #[test]
    fn synced_completed_transition_requires_completion_endpoint() {
        let mut todo = make_local_todo("local-a", Some(42), 1, true);
        todo.last_synced_status = Some(0);

        let plan = plan_remote_sync(&todo);

        assert!(plan.uses_complete_endpoint);
    }

    #[test]
    fn synced_reopened_todo_does_not_reappear_completed_after_merge() {
        let mut local = make_local_todo("local-a", Some(42), 0, false);
        local.last_synced_status = Some(0);
        let stale_cloud = make_api_todo(42, 1);
        let uploaded = HashMap::from([(42, make_api_todo(42, 0))]);

        let merged = merge_cloud_todos(
            &[local],
            vec![stale_cloud],
            uploaded,
            HashSet::new(),
            "2026-04-23T10:00:00Z",
        );

        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].status, 0);
        assert_eq!(merged[0].last_synced_status, Some(0));
    }

    #[test]
    fn merge_cloud_todos_drops_clean_synced_local_records_missing_from_cloud() {
        let local_todos = vec![make_local_todo("local-a", Some(42), 0, false)];

        let merged = merge_cloud_todos(
            &local_todos,
            vec![],
            HashMap::new(),
            HashSet::new(),
            "2026-04-23T10:00:00Z",
        );

        assert!(merged.is_empty());
    }

    #[test]
    fn loading_mixed_shape_todo_row_rewrites_cloud_id_out_of_file() {
        let dir = tempfile::tempdir().unwrap();
        let state = test_state(&dir);
        std::fs::write(
            dir.path().join("todos.json"),
            r#"[{"localId":"local-a","cloudId":42,"id":null,"title":"Buy milk","description":"","dueDate":null,"dueTime":null,"status":0,"priority":1,"deviceId":null,"dirty":false,"deleted":false,"createdAt":"2026-04-23T09:00:00Z","updatedAt":"2026-04-23T09:00:00Z"}]"#,
        )
        .unwrap();

        let todos = state.load_todos().unwrap();
        let rewritten = std::fs::read_to_string(dir.path().join("todos.json")).unwrap();

        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].id, Some(42));
        assert!(!rewritten.contains("cloudId"));
        assert!(!rewritten.contains("cloud_id"));
    }

    #[test]
    fn dirty_completed_migrated_remote_todo_still_requires_completion_endpoint() {
        let dir = tempfile::tempdir().unwrap();
        let state = test_state(&dir);
        std::fs::write(
            dir.path().join("todos.json"),
            r#"[{"localId":"local-a","cloudId":42,"id":null,"title":"Buy milk","description":"","dueDate":null,"dueTime":null,"status":1,"priority":1,"deviceId":null,"dirty":true,"deleted":false,"createdAt":"2026-04-23T09:00:00Z","updatedAt":"2026-04-23T09:00:00Z"}]"#,
        )
        .unwrap();

        let todos = state.load_todos().unwrap();
        let plan = plan_remote_sync(&todos[0]);

        assert_eq!(todos[0].id, Some(42));
        assert_eq!(todos[0].last_synced_status, None);
        assert!(plan.uses_complete_endpoint);
    }

    #[test]
    fn delete_local_todo_removes_unsynced_record_immediately() {
        let dir = tempfile::tempdir().unwrap();
        let state = test_state(&dir);
        save_json(
            &dir.path().join("todos.json"),
            &vec![make_local_todo("local-a", None, 0, true)],
        )
        .unwrap();

        state.delete_local_todo("local-a").unwrap();

        let todos = state.load_todos().unwrap();
        assert!(todos.is_empty());
    }

    #[test]
    fn delete_local_todo_marks_synced_record_as_dirty_tombstone() {
        let dir = tempfile::tempdir().unwrap();
        let state = test_state(&dir);
        save_json(
            &dir.path().join("todos.json"),
            &vec![make_local_todo("local-a", Some(42), 0, false)],
        )
        .unwrap();

        state.delete_local_todo("local-a").unwrap();

        let todos = state.load_todos().unwrap();
        assert_eq!(todos.len(), 1);
        assert!(todos[0].deleted);
        assert!(todos[0].dirty);
        assert_eq!(todos[0].id, Some(42));
    }

    #[test]
    fn mark_remote_delete_synced_drops_tombstone_from_local_records() {
        let mut local_todos = vec![make_local_todo("local-a", Some(42), 0, true)];
        local_todos[0].deleted = true;

        mark_remote_delete_synced(&mut local_todos, "local-a");

        assert!(local_todos.is_empty());
    }

    #[test]
    fn merge_cloud_todos_does_not_resurrect_just_deleted_remote_id_from_stale_fetch() {
        let stale_cloud = make_api_todo(42, 0);

        let merged = merge_cloud_todos(
            &[],
            vec![stale_cloud],
            HashMap::new(),
            HashSet::from([42]),
            "2026-04-23T10:00:00Z",
        );

        assert!(merged.is_empty());
    }

    #[test]
    fn remove_api_key_fails_when_device_associated() {
        let dir = tempfile::tempdir().unwrap();
        let state = test_state(&dir);

        // 创建 API Key 和设备文件
        save_json(
            &dir.path().join("api_keys.json"),
            &vec![ApiKeyRecord {
                id: 1,
                name: "Test Key".into(),
                key: "zt_test".into(),
                created_at: "2026-04-23T09:00:00Z".into(),
            }],
        )
        .unwrap();

        save_json(
            &dir.path().join("devices.json"),
            &vec![DeviceRecord {
                device_id: "AA:BB:CC:DD:EE:FF".into(),
                alias: "Test Device".into(),
                board: "board".into(),
                cached_at: "2026-04-23T09:00:00Z".into(),
                api_key_id: 1,
            }],
        )
        .unwrap();

        // 尝试删除有设备关联的 API Key
        let result = state.remove_api_key(1);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("关联设备"));
    }

    #[test]
    fn remove_api_key_succeeds_when_no_device_associated() {
        let dir = tempfile::tempdir().unwrap();
        let state = test_state(&dir);

        // 创建 API Key 文件（无设备）
        save_json(
            &dir.path().join("api_keys.json"),
            &vec![ApiKeyRecord {
                id: 1,
                name: "Test Key".into(),
                key: "zt_test".into(),
                created_at: "2026-04-23T09:00:00Z".into(),
            }],
        )
        .unwrap();

        save_json(&dir.path().join("devices.json"), &vec![] as &Vec<DeviceRecord>).unwrap();

        // 删除无设备关联的 API Key 应成功
        state.remove_api_key(1).unwrap();

        let api_keys: Vec<ApiKeyRecord> = load_json(&dir.path().join("api_keys.json")).unwrap();
        assert!(api_keys.is_empty());
    }

    #[test]
    fn remove_device_cache_deletes_associated_todos() {
        let dir = tempfile::tempdir().unwrap();
        let state = test_state(&dir);

        // 创建设备文件
        save_json(
            &dir.path().join("devices.json"),
            &vec![DeviceRecord {
                device_id: "AA:BB:CC:DD:EE:FF".into(),
                alias: "Test Device".into(),
                board: "board".into(),
                cached_at: "2026-04-23T09:00:00Z".into(),
                api_key_id: 1,
            }],
        )
        .unwrap();

        // 创建 todo 文件，包含关联该设备的 todo
        let mut todo1 = make_local_todo("local-a", None, 0, true);
        todo1.device_id = Some("AA:BB:CC:DD:EE:FF".into());
        let mut todo2 = make_local_todo("local-b", None, 0, true);
        todo2.device_id = Some("BB:BB:BB:BB:BB:BB".into()); // 其他设备
        let todo3 = make_local_todo("local-c", None, 0, true); // 无设备

        save_json(
            &dir.path().join("todos.json"),
            &vec![todo1, todo2, todo3],
        )
        .unwrap();

        // 删除设备
        state.remove_device_cache("AA:BB:CC:DD:EE:FF").unwrap();

        // 验证设备已删除
        let devices: Vec<DeviceRecord> = load_json(&dir.path().join("devices.json")).unwrap();
        assert!(devices.is_empty());

        // 验证关联该设备的 todo 已删除，其他 todo 保留
        let todos = state.load_todos().unwrap();
        assert_eq!(todos.len(), 2);
        assert!(todos.iter().all(|t| t.device_id != Some("AA:BB:CC:DD:EE:FF".into())));
    }

    #[test]
    fn scan_image_folder_detects_jpg_and_png_files() {
        let dir = tempfile::tempdir().unwrap();
        let state = test_state(&dir);

        // 创建图片文件
        std::fs::write(dir.path().join("photo1.jpg"), vec![0u8; 100]).unwrap();
        std::fs::write(dir.path().join("photo2.png"), vec![0u8; 100]).unwrap();
        std::fs::write(dir.path().join("readme.txt"), vec![0u8; 50]).unwrap();

        let result = state.scan_image_folder(dir.path().to_str().unwrap()).unwrap();

        assert_eq!(result.total_images, 2);
        assert_eq!(result.image_files, vec!["photo1.jpg", "photo2.png"]);
        assert!(result.warning.is_none());
    }

    #[test]
    fn scan_image_folder_returns_warning_for_empty_folder() {
        let dir = tempfile::tempdir().unwrap();
        let state = test_state(&dir);

        // 只创建非图片文件
        std::fs::write(dir.path().join("readme.txt"), vec![0u8; 50]).unwrap();

        let result = state.scan_image_folder(dir.path().to_str().unwrap()).unwrap();

        assert_eq!(result.total_images, 0);
        assert!(result.warning.is_some());
    }

    #[test]
    fn scan_image_folder_fails_for_nonexistent_path() {
        let dir = tempfile::tempdir().unwrap();
        let state = test_state(&dir);

        let result = state.scan_image_folder("/nonexistent/path");
        assert!(result.is_err());
    }

    #[test]
    fn create_image_loop_task_persists_to_json() {
        let dir = tempfile::tempdir().unwrap();
        let state = test_state(&dir);

        // 创建图片文件夹
        std::fs::write(dir.path().join("photo.jpg"), vec![0u8; 100]).unwrap();

        let input = crate::models::ImageLoopTaskInput {
            name: "测试相册".into(),
            folder_path: dir.path().to_str().unwrap().into(),
            device_id: "AA:BB:CC".into(),
            page_id: 1,
            interval_seconds: 30,
            duration_type: "none".into(),
            end_time: None,
            duration_minutes: None,
        };

        let record = state.create_image_loop_task(input).unwrap();

        assert!(record.id > 0);
        assert_eq!(record.name, "测试相册");
        assert_eq!(record.status, "idle");
        assert_eq!(record.total_images, 1);

        // 验证持久化
        let loaded = state.list_image_loop_tasks().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, record.id);
    }

    #[test]
    fn update_image_loop_task_modifies_existing_record() {
        let dir = tempfile::tempdir().unwrap();
        let state = test_state(&dir);

        std::fs::write(dir.path().join("photo.jpg"), vec![0u8; 100]).unwrap();

        let created = state.create_image_loop_task(crate::models::ImageLoopTaskInput {
            name: "原名".into(),
            folder_path: dir.path().to_str().unwrap().into(),
            device_id: "AA:BB:CC".into(),
            page_id: 1,
            interval_seconds: 30,
            duration_type: "none".into(),
            end_time: None,
            duration_minutes: None,
        }).unwrap();

        let updated = state.update_image_loop_task(created.id, crate::models::ImageLoopTaskInput {
            name: "新名".into(),
            folder_path: dir.path().to_str().unwrap().into(),
            device_id: "AA:BB:CC".into(),
            page_id: 2,
            interval_seconds: 60,
            duration_type: "for_duration".into(),
            end_time: None,
            duration_minutes: Some(30),
        }).unwrap();

        assert_eq!(updated.name, "新名");
        assert_eq!(updated.page_id, 2);
        assert_eq!(updated.interval_seconds, 60);
    }

    #[test]
    fn delete_image_loop_task_removes_from_list() {
        let dir = tempfile::tempdir().unwrap();
        let state = test_state(&dir);

        std::fs::write(dir.path().join("photo.jpg"), vec![0u8; 100]).unwrap();

        let created = state.create_image_loop_task(crate::models::ImageLoopTaskInput {
            name: "测试".into(),
            folder_path: dir.path().to_str().unwrap().into(),
            device_id: "AA:BB:CC".into(),
            page_id: 1,
            interval_seconds: 30,
            duration_type: "none".into(),
            end_time: None,
            duration_minutes: None,
        }).unwrap();

        state.delete_image_loop_task(created.id).unwrap();

        let loaded = state.list_image_loop_tasks().unwrap();
        assert!(loaded.is_empty());
    }

    #[test]
    fn custom_plugin_crud_persists_to_json() {
        let dir = tempfile::tempdir().unwrap();
        let state = test_state(&dir);

        let created = state
            .save_custom_plugin(crate::models::CustomPluginInput {
                id: None,
                name: "天气插件".into(),
                description: "查询天气并推送".into(),
                code: "return { type: 'text', text: 'sunny' };".into(),
            })
            .unwrap();

        assert!(created.id > 0);
        assert_eq!(created.name, "天气插件");

        let updated = state
            .save_custom_plugin(crate::models::CustomPluginInput {
                id: Some(created.id),
                name: "天气插件 v2".into(),
                description: "更新描述".into(),
                code: "return { type: 'text', text: 'cloudy' };".into(),
            })
            .unwrap();

        assert_eq!(updated.id, created.id);
        assert_eq!(updated.name, "天气插件 v2");
        assert!(updated.updated_at >= updated.created_at);

        let listed = state.list_custom_plugins().unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].code, "return { type: 'text', text: 'cloudy' };");

        state.delete_custom_plugin(created.id).unwrap();
        assert!(state.list_custom_plugins().unwrap().is_empty());
    }

    #[tokio::test]
    async fn run_plugin_once_returns_text_output_shape_for_custom_plugin() {
        let dir = tempfile::tempdir().unwrap();
        let state = test_state(&dir);

        let created = state
            .save_custom_plugin(crate::models::CustomPluginInput {
                id: None,
                name: "文本插件".into(),
                description: "返回文本".into(),
                code: "return { type: 'text', text: 'hello' };".into(),
            })
            .unwrap();

        let result = state
            .run_plugin_once("custom", &created.id.to_string())
            .await
            .unwrap();

        assert_eq!(result.output_type, "text");
        assert_eq!(result.text.as_deref(), Some("hello"));
    }

    #[tokio::test]
    async fn run_plugin_once_returns_text_image_preview_shape_for_custom_plugin() {
        let dir = tempfile::tempdir().unwrap();
        let state = test_state(&dir);

        let created = state
            .save_custom_plugin(crate::models::CustomPluginInput {
                id: None,
                name: "图文插件".into(),
                description: "返回图文".into(),
                code: "return { type: 'textImage', text: 'hello plugin' };".into(),
            })
            .unwrap();

        let result = state
            .run_plugin_once("custom", &created.id.to_string())
            .await
            .unwrap();

        assert_eq!(result.output_type, "textImage");
        assert!(result.preview_png_base64.is_some());
    }

    #[test]
    fn plugin_loop_task_crud_persists_to_json() {
        let dir = tempfile::tempdir().unwrap();
        let state = test_state(&dir);

        let created = state
            .create_plugin_loop_task(crate::models::PluginLoopTaskInput {
                plugin_kind: "custom".into(),
                plugin_id: "1".into(),
                name: "天气循环".into(),
                device_id: "AA:BB:CC:DD:EE:FF".into(),
                page_id: 2,
                interval_seconds: 60,
                duration_type: "none".into(),
                end_time: None,
                duration_minutes: None,
            })
            .unwrap();

        assert!(created.id > 0);
        assert_eq!(created.status, "idle");
        assert_eq!(created.page_id, 2);

        let updated = state
            .update_plugin_loop_task(
                created.id,
                crate::models::PluginLoopTaskInput {
                    plugin_kind: "custom".into(),
                    plugin_id: "1".into(),
                    name: "天气循环 v2".into(),
                    device_id: "AA:BB:CC:DD:EE:FF".into(),
                    page_id: 3,
                    interval_seconds: 120,
                    duration_type: "for_duration".into(),
                    end_time: None,
                    duration_minutes: Some(30),
                },
            )
            .unwrap();

        assert_eq!(updated.name, "天气循环 v2");
        assert_eq!(updated.page_id, 3);
        assert_eq!(updated.duration_minutes, Some(30));

        let listed = state.list_plugin_loop_tasks().unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id, created.id);

        state.delete_plugin_loop_task(created.id).unwrap();
        assert!(state.list_plugin_loop_tasks().unwrap().is_empty());
    }

    #[test]
    fn start_plugin_loop_task_sets_running_status() {
        let dir = tempfile::tempdir().unwrap();
        let state = test_state(&dir);
        let task = state
            .create_plugin_loop_task(crate::models::PluginLoopTaskInput {
                plugin_kind: "custom".into(),
                plugin_id: "1".into(),
                name: "循环".into(),
                device_id: "AA:BB:CC".into(),
                page_id: 1,
                interval_seconds: 60,
                duration_type: "none".into(),
                end_time: None,
                duration_minutes: None,
            })
            .unwrap();

        let started = state.start_plugin_loop_task(task.id).unwrap();

        assert_eq!(started.status, "running");
        assert!(started.error_message.is_none());
    }

    #[test]
    fn stop_plugin_loop_task_sets_idle_status() {
        let dir = tempfile::tempdir().unwrap();
        let state = test_state(&dir);
        let task = state
            .create_plugin_loop_task(crate::models::PluginLoopTaskInput {
                plugin_kind: "custom".into(),
                plugin_id: "1".into(),
                name: "循环".into(),
                device_id: "AA:BB:CC".into(),
                page_id: 1,
                interval_seconds: 60,
                duration_type: "none".into(),
                end_time: None,
                duration_minutes: None,
            })
            .unwrap();

        state.start_plugin_loop_task(task.id).unwrap();
        let stopped = state.stop_plugin_loop_task(task.id).unwrap();

        assert_eq!(stopped.status, "idle");
    }

    #[tokio::test]
    async fn start_image_loop_task_sets_running_status() {
        let dir = tempfile::tempdir().unwrap();
        let state = test_state(&dir);

        std::fs::write(dir.path().join("photo.jpg"), vec![0u8; 100]).unwrap();

        let created = state.create_image_loop_task(crate::models::ImageLoopTaskInput {
            name: "测试".into(),
            folder_path: dir.path().to_str().unwrap().into(),
            device_id: "AA:BB:CC".into(),
            page_id: 1,
            interval_seconds: 30,
            duration_type: "none".into(),
            end_time: None,
            duration_minutes: None,
        }).unwrap();

        let started = state.start_image_loop_task(created.id).unwrap();

        assert_eq!(started.status, "running");
        assert!(started.started_at.is_some());
        assert_eq!(started.current_index, 0);
    }

    #[test]
    fn start_image_loop_task_fails_for_empty_folder() {
        let dir = tempfile::tempdir().unwrap();
        let state = test_state(&dir);

        let created = state.create_image_loop_task(crate::models::ImageLoopTaskInput {
            name: "空文件夹".into(),
            folder_path: dir.path().to_str().unwrap().into(),
            device_id: "AA:BB:CC".into(),
            page_id: 1,
            interval_seconds: 30,
            duration_type: "none".into(),
            end_time: None,
            duration_minutes: None,
        }).unwrap();

        let result = state.start_image_loop_task(created.id);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("没有图片"));
    }

    #[tokio::test]
    async fn stop_image_loop_task_sets_idle_status() {
        let dir = tempfile::tempdir().unwrap();
        let state = test_state(&dir);

        std::fs::write(dir.path().join("photo.jpg"), vec![0u8; 100]).unwrap();

        let created = state.create_image_loop_task(crate::models::ImageLoopTaskInput {
            name: "测试".into(),
            folder_path: dir.path().to_str().unwrap().into(),
            device_id: "AA:BB:CC".into(),
            page_id: 1,
            interval_seconds: 30,
            duration_type: "none".into(),
            end_time: None,
            duration_minutes: None,
        }).unwrap();

        state.start_image_loop_task(created.id).unwrap();
        let stopped = state.stop_image_loop_task(created.id).unwrap();

        assert_eq!(stopped.status, "idle");
        assert!(stopped.started_at.is_none());
    }

    #[test]
    fn check_duration_condition_returns_true_when_duration_elapsed() {
        let dir = tempfile::tempdir().unwrap();
        let state = test_state(&dir);

        let started_at = chrono::Utc::now() - chrono::TimeDelta::minutes(35);

        let task = crate::models::ImageLoopTaskRecord {
            id: 1,
            name: "测试".into(),
            folder_path: dir.path().to_str().unwrap().into(),
            device_id: "AA:BB:CC".into(),
            page_id: 1,
            interval_seconds: 30,
            duration_type: "for_duration".into(),
            end_time: None,
            duration_minutes: Some(30),
            status: "running".into(),
            current_index: 0,
            total_images: 1,
            started_at: Some(started_at.to_rfc3339()),
            last_push_at: None,
            error_message: None,
            created_at: started_at.to_rfc3339(),
            updated_at: started_at.to_rfc3339(),
        };

        let result = state.check_duration_condition(&task).unwrap();
        assert!(result);
    }

    #[test]
    fn check_duration_condition_returns_false_when_duration_not_elapsed() {
        let dir = tempfile::tempdir().unwrap();
        let state = test_state(&dir);

        let started_at = chrono::Utc::now() - chrono::TimeDelta::minutes(10);

        let task = crate::models::ImageLoopTaskRecord {
            id: 1,
            name: "测试".into(),
            folder_path: dir.path().to_str().unwrap().into(),
            device_id: "AA:BB:CC".into(),
            page_id: 1,
            interval_seconds: 30,
            duration_type: "for_duration".into(),
            end_time: None,
            duration_minutes: Some(30),
            status: "running".into(),
            current_index: 0,
            total_images: 1,
            started_at: Some(started_at.to_rfc3339()),
            last_push_at: None,
            error_message: None,
            created_at: started_at.to_rfc3339(),
            updated_at: started_at.to_rfc3339(),
        };

        let result = state.check_duration_condition(&task).unwrap();
        assert!(!result);
    }
}
