# 图片循环推送功能实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 在本地图库页面新增循环相册功能，支持选择文件夹、配置循环间隔和持续时间、定时推送图片到墨水屏设备。

**Architecture:** 混合方案 - 后端管理任务配置和状态持久化，前端通过定时器触发推送命令。复用现有图片处理和推送逻辑。

**Tech Stack:** Tauri 2 + React + TypeScript + Rust

---

## 文件结构

**后端新增/修改：**
- `src-tauri/src/models.rs` - 新增 ImageLoopTaskRecord, ImageLoopTaskInput, ImageFolderScanResult 类型
- `src-tauri/src/commands/image_loop.rs` - 新建循环任务命令模块
- `src-tauri/src/commands/mod.rs` - 注册 image_loop 模块
- `src-tauri/src/state.rs` - 新增任务管理方法
- `src-tauri/src/lib.rs` - 注册新命令到 invoke_handler

**前端新增/修改：**
- `src/lib/tauri.ts` - 新增 Tauri 命令封装和类型定义
- `src/features/images/use-image-loop-runner.ts` - 新增定时器管理 Hook
- `src/features/images/image-loop-task-card.tsx` - 新增任务卡片组件
- `src/features/images/image-loop-task-list.tsx` - 新增任务列表组件
- `src/features/images/image-loop-task-dialog.tsx` - 新增新建/编辑对话框
- `src/features/images/image-templates-page.tsx` - 集成循环相册区域
- `src/features/images/image-loop-task-list.test.tsx` - 新增前端测试

---

## Task 1: 后端数据模型定义

**Files:**
- Modify: `src-tauri/src/models.rs`

- [ ] **Step 1: 在 models.rs 中添加新类型定义**

在文件末尾添加：

```rust
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
```

- [ ] **Step 2: 运行 Rust 编译检查**

```bash
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: 编译通过，无新增错误

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/models.rs
git commit -m "feat(models): add ImageLoopTaskRecord and ImageFolderScanResult types"
```

---

## Task 2: 后端 scan_image_folder 命令

**Files:**
- Create: `src-tauri/src/commands/image_loop.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/state.rs`
- Modify: `src-tauri/src/models.rs` (添加 Default for BootstrapState)

- [ ] **Step 1: 在 state.rs 中添加 scan_image_folder 方法**

在 `AppState` impl 中添加：

```rust
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
```

- [ ] **Step 2: 创建 commands/image_loop.rs 文件**

```rust
#[tauri::command]
pub fn scan_image_folder(
    state: tauri::State<'_, crate::state::AppState>,
    folder_path: String,
) -> Result<crate::models::ImageFolderScanResult, String> {
    state.scan_image_folder(&folder_path).map_err(|e| e.to_string())
}
```

- [ ] **Step 3: 在 commands/mod.rs 中注册模块**

在文件末尾添加：

```rust
pub mod image_loop;
```

- [ ] **Step 4: 在 state.rs 测试模块中添加测试**

在 `#[cfg(test)] mod tests` 中添加：

```rust
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
```

- [ ] **Step 5: 运行测试验证**

```bash
cargo test --manifest-path src-tauri/Cargo.toml scan_image_folder
```

Expected: 3 tests pass

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/commands/image_loop.rs src-tauri/src/commands/mod.rs src-tauri/src/state.rs
git commit -m "feat(commands): add scan_image_folder command for loop task"
```

---

## Task 3: 后端任务 CRUD 命令

**Files:**
- Modify: `src-tauri/src/state.rs`
- Modify: `src-tauri/src/commands/image_loop.rs`

- [ ] **Step 1: 在 state.rs 中添加任务加载和保存方法**

在 `AppState` impl 中添加：

```rust
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
```

- [ ] **Step 2: 在 commands/image_loop.rs 中添加 CRUD 命令**

添加：

```rust
#[tauri::command]
pub fn list_image_loop_tasks(
    state: tauri::State<'_, crate::state::AppState>,
) -> Result<Vec<crate::models::ImageLoopTaskRecord>, String> {
    state.list_image_loop_tasks().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_image_loop_task(
    state: tauri::State<'_, crate::state::AppState>,
    input: crate::models::ImageLoopTaskInput,
) -> Result<crate::models::ImageLoopTaskRecord, String> {
    state.create_image_loop_task(input).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_image_loop_task(
    state: tauri::State<'_, crate::state::AppState>,
    task_id: i64,
    input: crate::models::ImageLoopTaskInput,
) -> Result<crate::models::ImageLoopTaskRecord, String> {
    state.update_image_loop_task(task_id, input).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_image_loop_task(
    state: tauri::State<'_, crate::state::AppState>,
    task_id: i64,
) -> Result<(), String> {
    state.delete_image_loop_task(task_id).map_err(|e| e.to_string())
}
```

- [ ] **Step 3: 在 state.rs 测试模块中添加测试**

添加：

```rust
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
```

- [ ] **Step 4: 运行测试验证**

```bash
cargo test --manifest-path src-tauri/Cargo.toml image_loop_task
```

Expected: 3 tests pass

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/state.rs src-tauri/src/commands/image_loop.rs
git commit -m "feat(state): add CRUD operations for image loop tasks"
```

---

## Task 4: 后端 start/stop 命令

**Files:**
- Modify: `src-tauri/src/state.rs`
- Modify: `src-tauri/src/commands/image_loop.rs`

- [ ] **Step 1: 在 state.rs 中添加 start/stop 方法**

添加：

```rust
pub fn start_image_loop_task(&self, task_id: i64) -> anyhow::Result<crate::models::ImageLoopTaskRecord> {
    let mut tasks = self.load_image_loop_tasks()?;
    let task = tasks
        .iter_mut()
        .find(|t| t.id == task_id)
        .ok_or_else(|| anyhow::anyhow!("任务 {task_id} 未找到"))?;

    if task.total_images == 0 {
        anyhow::bail!("文件夹中没有图片，无法启动");
    }

    task.status = "running".to_string();
    task.current_index = 0;
    task.started_at = Some(chrono::Utc::now().to_rfc3339());
    task.error_message = None;
    task.updated_at = chrono::Utc::now().to_rfc3339();

    let updated = task.clone();
    self.save_image_loop_tasks(&tasks)?;
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
```

- [ ] **Step 2: 在 commands/image_loop.rs 中添加命令**

添加：

```rust
#[tauri::command]
pub fn start_image_loop_task(
    state: tauri::State<'_, crate::state::AppState>,
    task_id: i64,
) -> Result<crate::models::ImageLoopTaskRecord, String> {
    state.start_image_loop_task(task_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn stop_image_loop_task(
    state: tauri::State<'_, crate::state::AppState>,
    task_id: i64,
) -> Result<crate::models::ImageLoopTaskRecord, String> {
    state.stop_image_loop_task(task_id).map_err(|e| e.to_string())
}
```

- [ ] **Step 3: 在 state.rs 测试模块中添加测试**

添加：

```rust
#[test]
fn start_image_loop_task_sets_running_status() {
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

    // 不创建任何图片
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

#[test]
fn stop_image_loop_task_sets_idle_status() {
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
```

- [ ] **Step 4: 运行测试验证**

```bash
cargo test --manifest-path src-tauri/Cargo.toml start_image_loop
cargo test --manifest-path src-tauri/Cargo.toml stop_image_loop
```

Expected: 3 tests pass

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/state.rs src-tauri/src/commands/image_loop.rs
git commit -m "feat(state): add start/stop commands for image loop tasks"
```

---

## Task 5: 后端 push_folder_image 命令

**Files:**
- Modify: `src-tauri/src/state.rs`
- Modify: `src-tauri/src/commands/image_loop.rs`

- [ ] **Step 1: 在 state.rs 中添加 push_folder_image 方法**

添加：

```rust
pub async fn push_folder_image(&self, task_id: i64) -> anyhow::Result<crate::models::ImageLoopTaskRecord> {
    let mut tasks = self.load_image_loop_tasks()?;
    let task = tasks
        .iter_mut()
        .find(|t| t.id == task_id)
        .ok_or_else(|| anyhow::anyhow!("任务 {task_id} 未找到"))?;

    if task.status != "running" {
        anyhow::bail!("任务未在运行中");
    }

    // 获取设备信息
    let devices: Vec<crate::models::DeviceRecord> = crate::storage::load_json(
        &self.data_dir.join("devices.json")
    )?;
    let device = devices
        .iter()
        .find(|d| d.device_id.eq_ignore_ascii_case(&task.device_id))
        .ok_or_else(|| anyhow::anyhow!("设备 {} 未找到", task.device_id))?;

    let api_key = self.get_api_key_by_id(device.api_key_id)?;

    // 获取图片列表
    let scan_result = self.scan_image_folder(&task.folder_path)?;
    if scan_result.image_files.is_empty() {
        task.status = "completed".to_string();
        task.error_message = Some("文件夹中没有图片".to_string());
        task.updated_at = chrono::Utc::now().to_rfc3339();
        let updated = task.clone();
        self.save_image_loop_tasks(&tasks)?;
        return Ok(updated);
    }

    // 获取当前图片路径
    let image_file = &scan_result.image_files[task.current_index as usize];
    let image_path = std::path::Path::new(&task.folder_path).join(image_file);

    // 加载并处理图片为 400x300
    let img = image::open(&image_path)
        .map_err(|e| anyhow::anyhow!("无法打开图片 {}: {}", image_file, e))?;
    let processed = img.resize_to_fill(400, 300, image::imageops::FilterType::Lanczos3);

    // 编码为 PNG
    let mut buf = std::io::Cursor::new(Vec::new());
    processed.write_to(&mut buf, image::ImageFormat::Png)
        .map_err(|e| anyhow::anyhow!("编码 PNG 失败: {}", e))?;
    let image_bytes = buf.into_inner();

    // 推送图片
    crate::api::client::push_image(&api_key, &task.device_id, image_bytes.clone(), task.page_id)
        .await
        .map_err(|e| {
            task.status = "error".to_string();
            task.error_message = Some(e.to_string());
            task.updated_at = chrono::Utc::now().to_rfc3339();
            let _ = self.save_image_loop_tasks(&tasks);
            anyhow::anyhow!("推送失败: {}", e)
        })?;

    // 更新索引
    task.current_index = (task.current_index + 1) % scan_result.total_images;
    task.last_push_at = Some(chrono::Utc::now().to_rfc3339());
    task.updated_at = chrono::Utc::now().to_rfc3339();

    // 检查持续时间条件
    let should_complete = self.check_duration_condition(task)?;
    if should_complete {
        task.status = "completed".to_string();
    }

    let updated = task.clone();
    self.save_image_loop_tasks(&tasks)?;
    Ok(updated)
}

fn check_duration_condition(&self, task: &crate::models::ImageLoopTaskRecord) -> anyhow::Result<bool> {
    if task.duration_type == "none" {
        return Ok(false);
    }

    let started_at = task.started_at
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("任务缺少启动时间"))?;

    let started = chrono::DateTime::parse_from_rfc3339(started_at)?
        .with_timezone(&chrono::Utc);
    let now = chrono::Utc::now();

    if task.duration_type == "until_time" {
        if let Some(end_time) = &task.end_time {
            // 解析 HH:MM 格式
            let parts: Vec<&str> = end_time.split(':').collect();
            if parts.len() == 2 {
                let hour: u32 = parts[0].parse().unwrap_or(0);
                let minute: u32 = parts[1].parse().unwrap_or(0);

                let end_datetime = now.date_naive()
                    .and_hms_opt(hour, minute, 0)
                    .map(|dt| chrono::DateTime::from_naive_utc_and_offset(dt, chrono::Utc));

                if let Some(end) = end_datetime {
                    // 如果结束时间已过（比如设定的是今天的 18:00，但现在是 20:00，
                    // 且任务是在今天之前启动的），则检查是否需要完成
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
```

- [ ] **Step 2: 在 commands/image_loop.rs 中添加命令**

添加：

```rust
#[tauri::command]
pub async fn push_folder_image(
    state: tauri::State<'_, crate::state::AppState>,
    task_id: i64,
) -> Result<crate::models::ImageLoopTaskRecord, String> {
    state.push_folder_image(task_id).await.map_err(|e| e.to_string())
}
```

- [ ] **Step 3: 在 state.rs 测试模块中添加测试**

添加：

```rust
#[test]
fn check_duration_condition_returns_true_when_duration_elapsed() {
    let dir = tempfile::tempdir().unwrap();
    let state = test_state(&dir);

    let started_at = chrono::Utc::now() - chrono::Duration::minutes(35);

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

    let started_at = chrono::Utc::now() - chrono::Duration::minutes(10);

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
```

- [ ] **Step 4: 运行测试验证**

```bash
cargo test --manifest-path src-tauri/Cargo.toml check_duration
```

Expected: 2 tests pass

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/state.rs src-tauri/src/commands/image_loop.rs
git commit -m "feat(state): add push_folder_image command with duration check"
```

---

## Task 6: 后端 select_folder_dialog 命令

**Files:**
- Modify: `src-tauri/src/commands/image_loop.rs`

- [ ] **Step 1: 在 commands/image_loop.rs 中添加命令**

添加：

```rust
#[tauri::command]
pub fn select_folder_dialog(app: tauri::AppHandle) -> Result<Option<String>, String> {
    use tauri::dialog::blocking::FileDialogBuilder;

    let home = dirs::home_dir()
        .ok_or_else(|| "无法获取用户主目录".to_string())?;

    let result = FileDialogBuilder::new()
        .set_directory(home)
        .pick_folder();

    Ok(result.map(|p| p.to_string_lossy().to_string()))
}
```

- [ ] **Step 2: 运行编译检查**

```bash
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: 编译通过

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/commands/image_loop.rs
git commit -m "feat(commands): add select_folder_dialog command"
```

---

## Task 7: 后端注册命令到 lib.rs

**Files:**
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 在 lib.rs 的 invoke_handler 中注册新命令**

找到 `invoke_handler` 部分，添加新命令：

```rust
.invoke_handler(tauri::generate_handler![
    // ... 现有命令
    crate::commands::image_loop::scan_image_folder,
    crate::commands::image_loop::list_image_loop_tasks,
    crate::commands::image_loop::create_image_loop_task,
    crate::commands::image_loop::update_image_loop_task,
    crate::commands::image_loop::delete_image_loop_task,
    crate::commands::image_loop::start_image_loop_task,
    crate::commands::image_loop::stop_image_loop_task,
    crate::commands::image_loop::push_folder_image,
    crate::commands::image_loop::select_folder_dialog,
])
```

- [ ] **Step 2: 运行完整后端测试**

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected: All tests pass

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat(lib): register image_loop commands in invoke_handler"
```

---

## Task 8: 后端扩展 BootstrapState

**Files:**
- Modify: `src-tauri/src/models.rs`
- Modify: `src-tauri/src/state.rs`

- [ ] **Step 1: 在 models.rs 的 BootstrapState 中添加字段**

在 `BootstrapState` 结构体中添加：

```rust
pub image_loop_tasks: Vec<ImageLoopTaskRecord>,
```

- [ ] **Step 2: 在 state.rs 的 load_bootstrap_state 中加载循环任务**

在 `load_bootstrap_state` 方法中添加：

```rust
let image_loop_tasks = self.load_image_loop_tasks()?;
```

并在返回的 `BootstrapState` 中添加字段：

```rust
Ok(BootstrapState {
    // ... 现有字段
    image_loop_tasks,
})
```

- [ ] **Step 3: 为 BootstrapState 添加 Default trait**

在 models.rs 中确保 BootstrapState 有 Default 实现：

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapState {
    // ... 所有字段需要 #[serde(default)] 或有默认值
    #[serde(default)]
    pub image_loop_tasks: Vec<ImageLoopTaskRecord>,
}
```

- [ ] **Step 4: 运行编译和测试**

```bash
cargo test --manifest-path src-tauri/Cargo.toml load_bootstrap
```

Expected: 编译通过，相关测试 pass

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/models.rs src-tauri/src/state.rs
git commit -m "feat(state): add image_loop_tasks to BootstrapState"
```

---

## Task 9: 前端 Tauri 命令封装

**Files:**
- Modify: `src/lib/tauri.ts`

- [ ] **Step 1: 在 tauri.ts 中添加类型定义**

在文件开头类型定义区域添加：

```typescript
export type ImageLoopTask = {
  id: number;
  name: string;
  folderPath: string;
  deviceId: string;
  pageId: number;
  intervalSeconds: number;
  durationType: "none" | "until_time" | "for_duration";
  endTime?: string;
  durationMinutes?: number;
  status: "idle" | "running" | "completed" | "error";
  currentIndex: number;
  totalImages: number;
  startedAt?: string;
  lastPushAt?: string;
  errorMessage?: string;
  createdAt: string;
  updatedAt: string;
};

export type ImageLoopTaskInput = {
  name: string;
  folderPath: string;
  deviceId: string;
  pageId: number;
  intervalSeconds: number;
  durationType: "none" | "until_time" | "for_duration";
  endTime?: string;
  durationMinutes?: number;
};

export type ImageFolderScanResult = {
  totalImages: number;
  imageFiles: string[];
  warning?: string;
};
```

- [ ] **Step 2: 在 tauri.ts 中添加命令函数**

在文件末尾添加：

```typescript
export async function listImageLoopTasks(): Promise<ImageLoopTask[]> {
  return invoke<ImageLoopTask[]>("list_image_loop_tasks");
}

export async function createImageLoopTask(input: ImageLoopTaskInput): Promise<ImageLoopTask> {
  return invoke<ImageLoopTask>("create_image_loop_task", { input });
}

export async function updateImageLoopTask(taskId: number, input: ImageLoopTaskInput): Promise<ImageLoopTask> {
  return invoke<ImageLoopTask>("update_image_loop_task", { taskId, input });
}

export async function deleteImageLoopTask(taskId: number): Promise<void> {
  return invoke("delete_image_loop_task", { taskId });
}

export async function startImageLoopTask(taskId: number): Promise<ImageLoopTask> {
  return invoke<ImageLoopTask>("start_image_loop_task", { taskId });
}

export async function stopImageLoopTask(taskId: number): Promise<ImageLoopTask> {
  return invoke<ImageLoopTask>("stop_image_loop_task", { taskId });
}

export async function pushFolderImage(taskId: number): Promise<ImageLoopTask> {
  return invoke<ImageLoopTask>("push_folder_image", { taskId });
}

export async function scanImageFolder(folderPath: string): Promise<ImageFolderScanResult> {
  return invoke<ImageFolderScanResult>("scan_image_folder", { folderPath });
}

export async function selectFolderDialog(): Promise<string | null> {
  return invoke<string | null>("select_folder_dialog");
}
```

- [ ] **Step 3: 运行 TypeScript 类型检查**

```bash
pnpm tsc --noEmit
```

Expected: 无类型错误

- [ ] **Step 4: Commit**

```bash
git add src/lib/tauri.ts
git commit -m "feat(tauri): add image loop task commands and types"
```

---

## Task 10: 前端定时器 Hook

**Files:**
- Create: `src/features/images/use-image-loop-runner.ts`

- [ ] **Step 1: 创建 use-image-loop-runner.ts 文件**

```typescript
import { useEffect, useRef } from "react";
import type { ImageLoopTask } from "@/lib/tauri";

type PushFolderImageFn = (taskId: number) => Promise<ImageLoopTask>;

export function useImageLoopRunner(
  tasks: ImageLoopTask[],
  pushFolderImage: PushFolderImageFn,
  onTaskUpdate: (task: ImageLoopTask) => void,
) {
  const timersRef = useRef<Map<number, NodeJS.Timeout>>(new Map());

  useEffect(() => {
    const runningTasks = tasks.filter((t) => t.status === "running");
    const runningIds = new Set(runningTasks.map((t) => t.id));

    // 清理已停止任务的定时器
    for (const [taskId, timer] of timersRef.current.entries()) {
      if (!runningIds.has(taskId)) {
        clearInterval(timer);
        timersRef.current.delete(taskId);
      }
    }

    // 为运行中的任务创建定时器
    for (const task of runningTasks) {
      if (!timersRef.current.has(task.id)) {
        const timer = setInterval(async () => {
          try {
            const updated = await pushFolderImage(task.id);
            onTaskUpdate(updated);
            if (updated.status !== "running") {
              clearInterval(timer);
              timersRef.current.delete(task.id);
            }
          } catch (e) {
            // 错误已由后端记录，清理定时器
            clearInterval(timer);
            timersRef.current.delete(task.id);
          }
        }, task.intervalSeconds * 1000);

        timersRef.current.set(task.id, timer);
      }
    }

    return () => {
      for (const timer of timersRef.current.values()) {
        clearInterval(timer);
      }
      timersRef.current.clear();
    };
  }, [tasks, pushFolderImage, onTaskUpdate]);
}
```

- [ ] **Step 2: Commit**

```bash
git add src/features/images/use-image-loop-runner.ts
git commit -m "feat(images): add useImageLoopRunner hook for timer management"
```

---

## Task 11: 前端任务卡片组件

**Files:**
- Create: `src/features/images/image-loop-task-card.tsx`

- [ ] **Step 1: 创建 image-loop-task-card.tsx 文件**

```typescript
import { useState } from "react";
import type { ImageLoopTask, DeviceRecord } from "@/lib/tauri";

type Props = {
  task: ImageLoopTask;
  devices: DeviceRecord[];
  onStart: (taskId: number) => Promise<void>;
  onStop: (taskId: number) => Promise<void>;
  onEdit: (task: ImageLoopTask) => void;
  onDelete: (taskId: number) => Promise<void>;
};

const STATUS_COLORS: Record<string, string> = {
  idle: "bg-gray-400",
  running: "bg-green-500",
  completed: "bg-yellow-500",
  error: "bg-red-500",
};

const STATUS_TEXT: Record<string, string> = {
  idle: "未启动",
  running: "运行中",
  completed: "已完成",
  error: "错误",
};

export function ImageLoopTaskCard({
  task,
  devices,
  onStart,
  onStop,
  onEdit,
  onDelete,
}: Props) {
  const [expanded, setExpanded] = useState(false);
  const [loading, setLoading] = useState(false);

  const device = devices.find((d) => d.deviceId === task.deviceId);
  const deviceLabel = device ? `${device.alias} (${device.deviceId.slice(0, 8)})` : task.deviceId;

  const handleStart = async () => {
    setLoading(true);
    try {
      await onStart(task.id);
    } finally {
      setLoading(false);
    }
  };

  const handleStop = async () => {
    setLoading(true);
    try {
      await onStop(task.id);
    } finally {
      setLoading(false);
    }
  };

  const handleDelete = async () => {
    if (!window.confirm(`确定要删除任务 "${task.name}" 吗？`)) return;
    await onDelete(task.id);
  };

  const durationLabel = () => {
    if (task.durationType === "none") return "无限制";
    if (task.durationType === "until_time") return `运行至 ${task.endTime}`;
    if (task.durationType === "for_duration") return `运行 ${task.durationMinutes} 分钟`;
    return "";
  };

  const runningInfo = () => {
    if (task.status !== "running" || !task.startedAt) return null;
    const started = new Date(task.startedAt);
    const now = new Date();
    const elapsedMinutes = Math.floor((now.getTime() - started.getTime()) / 60000);
    const rounds = Math.floor(task.currentIndex / task.totalImages);
    return `已运行 ${elapsedMinutes} 分钟，播放 ${rounds} 轮`;
  };

  return (
    <div className="rounded-lg border border-gray-200 bg-white shadow-sm overflow-hidden">
      <div className="p-3 flex items-center gap-3">
        <span className={`w-3 h-3 rounded-full ${STATUS_COLORS[task.status]}`} />
        <span className="font-medium flex-1">任务名称: {task.name}</span>
        <div className="flex gap-2">
          {task.status === "idle" || task.status === "error" ? (
            <button
              type="button"
              onClick={handleStart}
              disabled={loading}
              className="px-2 py-1 text-sm bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50"
            >
              ▶
            </button>
          ) : task.status === "running" ? (
            <button
              type="button"
              onClick={handleStop}
              disabled={loading}
              className="px-2 py-1 text-sm bg-gray-600 text-white rounded hover:bg-gray-700 disabled:opacity-50"
            >
              ⏹
            </button>
          ) : null}
        </div>
      </div>
      <div className="px-3 pb-2 text-sm text-gray-600">
        {task.status === "running"
          ? `当前: 第 ${task.currentIndex + 1}/${task.totalImages} 张`
          : task.status === "error"
          ? task.errorMessage || "错误"
          : STATUS_TEXT[task.status]}
      </div>
      <button
        type="button"
        onClick={() => setExpanded(!expanded)}
        className="w-full px-3 pb-2 text-sm text-blue-600 hover:text-blue-800"
      >
        {expanded ? "▼ 收起" : "▶ 展开"}
      </button>

      {expanded && (
        <div className="border-t border-gray-200 p-3 text-sm space-y-1">
          <div>文件夹: {task.folderPath}</div>
          <div>目标: {deviceLabel} / 第 {task.pageId} 页</div>
          <div>间隔: {task.intervalSeconds} 秒</div>
          <div>持续: {durationLabel()}</div>
          {runningInfo() && <div>{runningInfo()}</div>}
          {task.lastPushAt && <div>最后推送: {new Date(task.lastPushAt).toLocaleTimeString()}</div>}
          <hr className="my-2 border-gray-200" />
          <div className="flex gap-2">
            <button
              type="button"
              onClick={() => onEdit(task)}
              className="px-2 py-1 text-sm border border-gray-300 rounded hover:bg-gray-100"
            >
              编辑
            </button>
            <button
              type="button"
              onClick={handleDelete}
              className="px-2 py-1 text-sm border border-red-300 text-red-600 rounded hover:bg-red-50"
            >
              删除
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
```

- [ ] **Step 2: Commit**

```bash
git add src/features/images/image-loop-task-card.tsx
git commit -m "feat(images): add ImageLoopTaskCard component"
```

---

## Task 12: 前端任务列表组件

**Files:**
- Create: `src/features/images/image-loop-task-list.tsx`

- [ ] **Step 1: 创建 image-loop-task-list.tsx 文件**

```typescript
import type { ImageLoopTask, DeviceRecord } from "@/lib/tauri";
import { ImageLoopTaskCard } from "./image-loop-task-card";

type Props = {
  tasks: ImageLoopTask[];
  devices: DeviceRecord[];
  onStart: (taskId: number) => Promise<void>;
  onStop: (taskId: number) => Promise<void>;
  onEdit: (task: ImageLoopTask) => void;
  onDelete: (taskId: number) => Promise<void>;
};

export function ImageLoopTaskList({
  tasks,
  devices,
  onStart,
  onStop,
  onEdit,
  onDelete,
}: Props) {
  if (tasks.length === 0) {
    return (
      <p className="text-sm text-gray-500 py-4">
        暂无循环相册任务，点击上方"新建任务"创建。
      </p>
    );
  }

  return (
    <ul className="space-y-3">
      {tasks.map((task) => (
        <li key={task.id}>
          <ImageLoopTaskCard
            task={task}
            devices={devices}
            onStart={onStart}
            onStop={onStop}
            onEdit={onEdit}
            onDelete={onDelete}
          />
        </li>
      ))}
    </ul>
  );
}
```

- [ ] **Step 2: Commit**

```bash
git add src/features/images/image-loop-task-list.tsx
git commit -m "feat(images): add ImageLoopTaskList component"
```

---

## Task 13: 前端新建/编辑对话框

**Files:**
- Create: `src/features/images/image-loop-task-dialog.tsx`

- [ ] **Step 1: 创建 image-loop-task-dialog.tsx 文件**

```typescript
import { useState, useEffect } from "react";
import type { ImageLoopTask, ImageLoopTaskInput, DeviceRecord, ImageFolderScanResult } from "@/lib/tauri";
import { scanImageFolder, selectFolderDialog } from "@/lib/tauri";

type Props = {
  open: boolean;
  devices: DeviceRecord[];
  editingTask?: ImageLoopTask;
  onSave: (input: ImageLoopTaskInput) => Promise<void>;
  onClose: () => void;
};

const INTERVAL_OPTIONS = [
  { value: 10, label: "10 秒" },
  { value: 30, label: "30 秒" },
  { value: 60, label: "1 分钟" },
  { value: 300, label: "5 分钟" },
  { value: 600, label: "10 分钟" },
];

const PAGE_OPTIONS = [
  { value: 1, label: "第 1 页" },
  { value: 2, label: "第 2 页" },
  { value: 3, label: "第 3 页" },
  { value: 4, label: "第 4 页" },
  { value: 5, label: "第 5 页" },
];

export function ImageLoopTaskDialog({
  open,
  devices,
  editingTask,
  onSave,
  onClose,
}: Props) {
  const [name, setName] = useState("");
  const [folderPath, setFolderPath] = useState("");
  const [deviceId, setDeviceId] = useState("");
  const [pageId, setPageId] = useState(1);
  const [intervalSeconds, setIntervalSeconds] = useState(30);
  const [durationType, setDurationType] = useState<"none" | "until_time" | "for_duration">("none");
  const [endTime, setEndTime] = useState("");
  const [durationMinutes, setDurationMinutes] = useState(60);
  const [scanResult, setScanResult] = useState<ImageFolderScanResult | null>(null);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    if (editingTask) {
      setName(editingTask.name);
      setFolderPath(editingTask.folderPath);
      setDeviceId(editingTask.deviceId);
      setPageId(editingTask.pageId);
      setIntervalSeconds(editingTask.intervalSeconds);
      setDurationType(editingTask.durationType);
      setEndTime(editingTask.endTime || "");
      setDurationMinutes(editingTask.durationMinutes || 60);
      scanFolder(editingTask.folderPath);
    } else {
      resetForm();
    }
  }, [editingTask, open]);

  const resetForm = () => {
    setName("");
    setFolderPath("");
    setDeviceId(devices[0]?.deviceId || "");
    setPageId(1);
    setIntervalSeconds(30);
    setDurationType("none");
    setEndTime("");
    setDurationMinutes(60);
    setScanResult(null);
  };

  const handleSelectFolder = async () => {
    const path = await selectFolderDialog();
    if (path) {
      setFolderPath(path);
      await scanFolder(path);
    }
  };

  const scanFolder = async (path: string) => {
    try {
      const result = await scanImageFolder(path);
      setScanResult(result);
    } catch {
      setScanResult(null);
    }
  };

  const handleSave = async () => {
    if (!name.trim()) {
      alert("请输入任务名称");
      return;
    }
    if (!folderPath) {
      alert("请选择图片文件夹");
      return;
    }
    if (!deviceId) {
      alert("请选择目标设备");
      return;
    }

    setSaving(true);
    try {
      await onSave({
        name: name.trim(),
        folderPath,
        deviceId,
        pageId,
        intervalSeconds,
        durationType,
        endTime: durationType === "until_time" ? endTime : undefined,
        durationMinutes: durationType === "for_duration" ? durationMinutes : undefined,
      });
      onClose();
    } catch (e) {
      alert(`保存失败: ${e instanceof Error ? e.message : String(e)}`);
    } finally {
      setSaving(false);
    }
  };

  if (!open) return null;

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg shadow-xl w-full max-w-md p-4">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold">
            {editingTask ? "编辑循环相册任务" : "新建循环相册任务"}
          </h3>
          <button type="button" onClick={onClose} className="text-gray-500 hover:text-gray-700">
            ✕
          </button>
        </div>

        <div className="space-y-4">
          <div>
            <label className="block text-sm font-medium mb-1">任务名称</label>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 rounded-md"
              placeholder="例如: 周末旅行相册"
            />
          </div>

          <div>
            <label className="block text-sm font-medium mb-1">图片文件夹</label>
            <div className="flex gap-2">
              <input
                type="text"
                value={folderPath}
                onChange={(e) => setFolderPath(e.target.value)}
                className="flex-1 px-3 py-2 border border-gray-300 rounded-md text-sm"
                readOnly
              />
              <button
                type="button"
                onClick={handleSelectFolder}
                className="px-3 py-2 bg-gray-100 border border-gray-300 rounded-md text-sm hover:bg-gray-200"
              >
                选择
              </button>
            </div>
            {scanResult && (
              <p className="mt-1 text-sm text-gray-600">
                {scanResult.warning ? (
                  <span className="text-yellow-600">{scanResult.warning}</span>
                ) : (
                  `已检测到 ${scanResult.totalImages} 张图片`
                )}
              </p>
            )}
          </div>

          <div>
            <label className="block text-sm font-medium mb-1">目标设备</label>
            <select
              value={deviceId}
              onChange={(e) => setDeviceId(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 rounded-md"
            >
              {devices.map((d) => (
                <option key={d.deviceId} value={d.deviceId}>
                  {d.alias} ({d.deviceId.slice(0, 8)})
                </option>
              ))}
            </select>
          </div>

          <div>
            <label className="block text-sm font-medium mb-1">目标页面</label>
            <select
              value={pageId}
              onChange={(e) => setPageId(Number(e.target.value))}
              className="w-full px-3 py-2 border border-gray-300 rounded-md"
            >
              {PAGE_OPTIONS.map((opt) => (
                <option key={opt.value} value={opt.value}>
                  {opt.label}
                </option>
              ))}
            </select>
          </div>

          <div>
            <label className="block text-sm font-medium mb-1">循环间隔</label>
            <select
              value={intervalSeconds}
              onChange={(e) => setIntervalSeconds(Number(e.target.value))}
              className="w-full px-3 py-2 border border-gray-300 rounded-md"
            >
              {INTERVAL_OPTIONS.map((opt) => (
                <option key={opt.value} value={opt.value}>
                  {opt.label}
                </option>
              ))}
            </select>
          </div>

          <div>
            <label className="block text-sm font-medium mb-1">持续时间</label>
            <div className="space-y-2">
              <label className="flex items-center gap-2">
                <input
                  type="radio"
                  name="duration"
                  checked={durationType === "none"}
                  onChange={() => setDurationType("none")}
                />
                <span className="text-sm">无限制</span>
              </label>
              <label className="flex items-center gap-2">
                <input
                  type="radio"
                  name="duration"
                  checked={durationType === "until_time"}
                  onChange={() => setDurationType("until_time")}
                />
                <span className="text-sm">运行至指定时间</span>
                <input
                  type="time"
                  value={endTime}
                  onChange={(e) => setEndTime(e.target.value)}
                  disabled={durationType !== "until_time"}
                  className="px-2 py-1 border border-gray-300 rounded-md text-sm"
                />
              </label>
              <label className="flex items-center gap-2">
                <input
                  type="radio"
                  name="duration"
                  checked={durationType === "for_duration"}
                  onChange={() => setDurationType("for_duration")}
                />
                <span className="text-sm">运行指定时长</span>
                <input
                  type="number"
                  value={durationMinutes}
                  onChange={(e) => setDurationMinutes(Number(e.target.value))}
                  disabled={durationType !== "for_duration"}
                  className="w-16 px-2 py-1 border border-gray-300 rounded-md text-sm"
                  min={1}
                />
                <span className="text-sm">分钟</span>
              </label>
            </div>
          </div>
        </div>

        <div className="mt-4 flex justify-end gap-2">
          <button
            type="button"
            onClick={onClose}
            className="px-4 py-2 border border-gray-300 rounded-md hover:bg-gray-100"
          >
            取消
          </button>
          <button
            type="button"
            onClick={handleSave}
            disabled={saving}
            className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50"
          >
            {saving ? "保存中..." : "保存任务"}
          </button>
        </div>
      </div>
    </div>
  );
}
```

- [ ] **Step 2: Commit**

```bash
git add src/features/images/image-loop-task-dialog.tsx
git commit -m "feat(images): add ImageLoopTaskDialog component"
```

---

## Task 14: 前端集成到 image-templates-page.tsx

**Files:**
- Modify: `src/features/images/image-templates-page.tsx`

- [ ] **Step 1: 修改 image-templates-page.tsx 集成循环相册**

在文件开头添加导入：

```typescript
import { useState, useCallback } from "react";
import {
  listImageLoopTasks,
  createImageLoopTask,
  updateImageLoopTask,
  deleteImageLoopTask,
  startImageLoopTask,
  stopImageLoopTask,
  pushFolderImage,
  type ImageLoopTask,
  type ImageLoopTaskInput,
  type DeviceRecord,
} from "@/lib/tauri";
import { ImageLoopTaskList } from "./image-loop-task-list";
import { ImageLoopTaskDialog } from "./image-loop-task-dialog";
import { useImageLoopRunner } from "./use-image-loop-runner";
```

修改 Props 类型：

```typescript
type Props = {
  templates: ImageTemplateRecord[];
  devices: DeviceRecord[];
  imageLoopTasks: ImageLoopTask[];
  onSaveTemplate: (input: {
    name: string;
    sourcePath?: string;
    sourceDataUrl?: string;
    crop: { x: number; y: number; width: number; height: number };
    rotation: number;
    flipX: boolean;
    flipY: boolean;
  }) => Promise<ImageTemplateRecord>;
  onPushTemplate: (templateId: number, deviceId: string, pageId: number) => Promise<void>;
  onDeleteTemplate: (templateId: number) => Promise<void>;
  onLoadThumbnail?: (templateId: number) => Promise<string>;
  onCreateLoopTask: (input: ImageLoopTaskInput) => Promise<void>;
  onUpdateLoopTask: (taskId: number, input: ImageLoopTaskInput) => Promise<void>;
  onDeleteLoopTask: (taskId: number) => Promise<void>;
  onStartLoopTask: (taskId: number) => Promise<ImageLoopTask>;
  onStopLoopTask: (taskId: number) => Promise<ImageLoopTask>;
  onRefreshLoopTasks: () => Promise<void>;
};
```

在组件内部添加状态和逻辑：

```typescript
// 在现有 state 定义后添加
const [loopTasks, setLoopTasks] = useState<ImageLoopTask[]>(imageLoopTasks);
const [dialogOpen, setDialogOpen] = useState(false);
const [editingTask, setEditingTask] = useState<ImageLoopTask | undefined>();

// 定时器管理
const handleTaskUpdate = useCallback((updated: ImageLoopTask) => {
  setLoopTasks((prev) => prev.map((t) => (t.id === updated.id ? updated : t)));
}, []);

useImageLoopRunner(loopTasks, pushFolderImage, handleTaskUpdate);

// 任务操作
const handleCreateTask = async (input: ImageLoopTaskInput) => {
  await onCreateLoopTask(input);
  await onRefreshLoopTasks();
};

const handleUpdateTask = async (input: ImageLoopTaskInput) => {
  if (editingTask) {
    await onUpdateLoopTask(editingTask.id, input);
    await onRefreshLoopTasks();
    setEditingTask(undefined);
  }
};

const handleDeleteTask = async (taskId: number) => {
  await onDeleteLoopTask(taskId);
  setLoopTasks((prev) => prev.filter((t) => t.id !== taskId));
};

const handleStartTask = async (taskId: number) => {
  const started = await onStartLoopTask(taskId);
  handleTaskUpdate(started);
};

const handleStopTask = async (taskId: number) => {
  const stopped = await onStopLoopTask(taskId);
  handleTaskUpdate(stopped);
};

const handleEditTask = (task: ImageLoopTask) => {
  setEditingTask(task);
  setDialogOpen(true);
};
```

在 JSX 中添加循环相册区域（在现有模板列表后）：

```tsx
{/* 循环相册区域 */}
<section className="mt-8 space-y-4">
  <div className="flex items-center justify-between">
    <h2 className="text-lg font-semibold">循环相册</h2>
    <button
      type="button"
      onClick={() => {
        setEditingTask(undefined);
        setDialogOpen(true);
      }}
      className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700"
    >
      新建任务
    </button>
  </div>

  <ImageLoopTaskList
    tasks={loopTasks}
    devices={devices}
    onStart={handleStartTask}
    onStop={handleStopTask}
    onEdit={handleEditTask}
    onDelete={handleDeleteTask}
  />
</section>

{/* 任务对话框 */}
<ImageLoopTaskDialog
  open={dialogOpen}
  devices={devices}
  editingTask={editingTask}
  onSave={editingTask ? handleUpdateTask : handleCreateTask}
  onClose={() => {
    setDialogOpen(false);
    setEditingTask(undefined);
  }}
/>
```

- [ ] **Step 2: Commit**

```bash
git add src/features/images/image-templates-page.tsx
git commit -m "feat(images): integrate loop album into ImageTemplatesPage"
```

---

## Task 15: 前端测试

**Files:**
- Create: `src/features/images/image-loop-task-list.test.tsx`

- [ ] **Step 1: 创建测试文件**

```typescript
import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { ImageLoopTaskList } from "./image-loop-task-list";
import type { ImageLoopTask, DeviceRecord } from "@/lib/tauri";

const mockDevice: DeviceRecord = {
  deviceId: "AA:BB:CC:DD:EE:FF",
  alias: "测试设备",
  board: "test",
  cachedAt: "2026-04-24T10:00:00Z",
  apiKeyId: 1,
};

const mockTask: ImageLoopTask = {
  id: 1,
  name: "周末旅行",
  folderPath: "/Users/test/Pictures/travel",
  deviceId: "AA:BB:CC:DD:EE:FF",
  pageId: 1,
  intervalSeconds: 30,
  durationType: "none",
  status: "idle",
  currentIndex: 0,
  totalImages: 12,
  createdAt: "2026-04-24T10:00:00Z",
  updatedAt: "2026-04-24T10:00:00Z",
};

describe("ImageLoopTaskList", () => {
  it("shows empty message when no tasks", () => {
    render(
      <ImageLoopTaskList
        tasks={[]}
        devices={[mockDevice]}
        onStart={vi.fn()}
        onStop={vi.fn()}
        onEdit={vi.fn()}
        onDelete={vi.fn()}
      />
    );

    expect(screen.getByText(/暂无循环相册任务/)).toBeInTheDocument();
  });

  it("renders task cards for each task", () => {
    render(
      <ImageLoopTaskList
        tasks={[mockTask]}
        devices={[mockDevice]}
        onStart={vi.fn()}
        onStop={vi.fn()}
        onEdit={vi.fn()}
        onDelete={vi.fn()}
      />
    );

    expect(screen.getByText("任务名称: 周末旅行")).toBeInTheDocument();
    expect(screen.getByText("未启动")).toBeInTheDocument();
  });

  it("shows running status for running task", () => {
    const runningTask = { ...mockTask, status: "running" as const };
    render(
      <ImageLoopTaskList
        tasks={[runningTask]}
        devices={[mockDevice]}
        onStart={vi.fn()}
        onStop={vi.fn()}
        onEdit={vi.fn()}
        onDelete={vi.fn()}
      />
    );

    expect(screen.getByText(/当前: 第/)).toBeInTheDocument();
  });
});
```

- [ ] **Step 2: 运行测试**

```bash
pnpm vitest run src/features/images/image-loop-task-list.test.tsx
```

Expected: 3 tests pass

- [ ] **Step 3: Commit**

```bash
git add src/features/images/image-loop-task-list.test.tsx
git commit -m "test(images): add tests for ImageLoopTaskList"
```

---

## Task 16: App.tsx 集成

**Files:**
- Modify: `src/app/App.tsx`

- [ ] **Step 1: 在 App.tsx 中添加导入**

在现有导入列表中添加：

```typescript
import {
  // ... 现有导入
  createImageLoopTask,
  updateImageLoopTask,
  deleteImageLoopTask,
  startImageLoopTask,
  stopImageLoopTask,
  pushFolderImage,
  listImageLoopTasks,
  type ImageLoopTask,
  type ImageLoopTaskInput,
} from "../lib/tauri";
```

- [ ] **Step 2: 修改 emptyState 添加 imageLoopTasks 字段**

```typescript
const emptyState: BootstrapState = {
  apiKeys: [],
  devices: [],
  todos: [],
  textTemplates: [],
  imageTemplates: [],
  lastSyncTime: null,
  pageCache: [],
  imageLoopTasks: [],
};
```

- [ ] **Step 3: 添加循环任务刷新函数**

在 `reload()` 函数后添加：

```typescript
async function refreshLoopTasks() {
  const tasks = await listImageLoopTasks();
  setState((prev) => ({ ...prev, imageLoopTasks: tasks }));
}
```

- [ ] **Step 4: 修改 ImageTemplatesPage 调用传递新 props**

找到 `/image-push` 的 renderContent 分支，修改为：

```typescript
if (path === "/image-push") {
  return (
    <ImageTemplatesPage
      templates={state.imageTemplates}
      devices={state.devices}
      imageLoopTasks={state.imageLoopTasks}
      onSaveTemplate={async (input) => {
        const t = await saveImageTemplate(input);
        reload();
        return t;
      }}
      onPushTemplate={(templateId, deviceId, pageId) =>
        pushImageTemplate(templateId, deviceId, pageId)
      }
      onDeleteTemplate={async (templateId) => {
        await deleteImageTemplate(templateId);
        reload();
      }}
      onLoadThumbnail={getImageThumbnail}
      onCreateLoopTask={async (input: ImageLoopTaskInput) => {
        await createImageLoopTask(input);
      }}
      onUpdateLoopTask={async (taskId: number, input: ImageLoopTaskInput) => {
        await updateImageLoopTask(taskId, input);
      }}
      onDeleteLoopTask={async (taskId: number) => {
        await deleteImageLoopTask(taskId);
      }}
      onStartLoopTask={async (taskId: number) => {
        return await startImageLoopTask(taskId);
      }}
      onStopLoopTask={async (taskId: number) => {
        return await stopImageLoopTask(taskId);
      }}
      onRefreshLoopTasks={refreshLoopTasks}
    />
  );
}
```

- [ ] **Step 5: 运行 TypeScript 类型检查**

```bash
pnpm tsc --noEmit
```

Expected: 无类型错误

- [ ] **Step 6: Commit**

```bash
git add src/app/App.tsx
git commit -m "feat(app): integrate imageLoopTasks into App flow"
```

---

## Task 17: 最终验证

- [ ] **Step 1: 运行完整后端测试**

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected: All tests pass

- [ ] **Step 2: 运行完整前端测试**

```bash
pnpm vitest run
```

Expected: All tests pass

- [ ] **Step 3: 启动开发模式进行手动测试**

```bash
pnpm tauri dev
```

手动测试要点：
1. 创建新循环任务
2. 选择文件夹，验证图片检测
3. 启动任务，观察推送
4. 停止任务
5. 编辑任务
6. 删除任务

- [ ] **Step 4: 最终 Commit**

```bash
git add -A
git commit -m "feat: complete image loop task feature implementation"
```