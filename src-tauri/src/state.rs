use crate::models::{
    ApiKeyRecord, AppConfig, BootstrapState, CropRect, DeviceRecord, ImageEditInput,
    ImageTemplateRecord, ImageTemplateSaveInput, TextTemplateInput, TextTemplateRecord, TodoRecord,
    TodoUpsertInput,
};
use crate::storage::{load_json, save_json};
use image::GenericImageView;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

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
        status: todo.status,
        priority: todo.priority,
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

        Ok(BootstrapState {
            api_keys,
            devices,
            todos,
            text_templates,
            image_templates,
            last_sync_time: config.last_sync_time,
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
            status: 0,
            priority: input.priority,
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
        todo.priority = input.priority;
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
        crate::api::client::push_text(&api_key, device_id, text, font_size, page_id).await
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
        image::open(source_path).map_err(|e| anyhow::anyhow!("无法打开图片: {e}"))
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
        crate::api::client::push_image(&api_key, device_id, image_bytes, page_id).await
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
        crate::api::client::push_image(&api_key, device_id, image_bytes, page_id).await
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
                priority: 1,
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

        assert_eq!(body.status, 1);
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
}
