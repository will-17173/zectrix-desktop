# 页面管理功能实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现页面管理功能，展示每个设备的5个墨水屏页面内容，支持查看缩略图预览和删除页面内容。

**Architecture:** 后端使用 JSON 文件存储页面缓存数据（与现有 todos/templates 存储方式一致）。推送成功后自动写入缓存，删除时先删本地再调云端 API。前端新增独立页面组件，通过 BootstrapState 获取数据。

**Tech Stack:** Tauri (Rust backend) + React (TypeScript frontend) + reqwest (HTTP client)

---

## Task 1: 添加 PageCacheRecord 类型定义

**Files:**
- Modify: `src-tauri/src/models.rs`
- Modify: `src-tauri/src/models.rs` (BootstrapState)

- [ ] **Step 1: 在 models.rs 添加 PageCacheRecord 类型**

```rust
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
```

位置：在 `ImageTemplateSaveInput` 结构体之后添加。

- [ ] **Step 2: 在 BootstrapState 添加 page_cache 字段**

修改 `BootstrapState` 结构体，添加：

```rust
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
```

- [ ] **Step 3: 编译验证**

Run: `cd src-tauri && cargo check`
Expected: 编译成功，无错误

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/models.rs
git commit -m "feat(models): add PageCacheRecord type for page manager feature"
```

---

## Task 2: 添加删除页面 API 函数

**Files:**
- Modify: `src-tauri/src/api/client.rs`

- [ ] **Step 1: 添加 delete_page 函数**

在 `push_image` 函数之后添加：

```rust
pub async fn delete_page(
    api_key: &str,
    device_id: &str,
    page_id: Option<u32>,
) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let url = if let Some(pid) = page_id {
        format!("{BASE_URL}/devices/{device_id}/display/pages/{pid}")
    } else {
        format!("{BASE_URL}/devices/{device_id}/display/pages")
    };
    
    let resp = client
        .delete(&url)
        .header("X-API-Key", api_key)
        .send()
        .await?;
    
    let status = resp.status();
    if !status.is_success() {
        anyhow::bail!("删除页面失败: {status}");
    }
    
    let api_resp: ApiResponse<serde_json::Value> = resp.json().await?;
    if api_resp.code != 0 {
        anyhow::bail!(
            "删除页面失败: {}",
            api_resp.msg.unwrap_or_else(|| "未知错误".to_string())
        );
    }
    
    Ok(())
}
```

- [ ] **Step 2: 编译验证**

Run: `cd src-tauri && cargo check`
Expected: 编译成功

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/api/client.rs
git commit -m "feat(api): add delete_page function for cloud API"
```

---

## Task 3: 在 AppState 添加页面缓存方法

**Files:**
- Modify: `src-tauri/src/state.rs`

- [ ] **Step 1: 导入 PageCacheRecord**

修改 state.rs 顶部的 import：

```rust
use crate::models::{
    ApiKeyRecord, AppConfig, BootstrapState, CropRect, DeviceRecord, ImageEditInput,
    ImageTemplateRecord, ImageTemplateSaveInput, PageCacheRecord, TextTemplateInput, TextTemplateRecord, TodoRecord,
    TodoUpsertInput,
};
```

- [ ] **Step 2: 添加 load_page_cache 方法**

在 `remove_device_cache` 方法之后添加：

```rust
fn load_page_cache(&self) -> anyhow::Result<Vec<PageCacheRecord>> {
    load_json(&self.data_dir.join("page_cache.json"))
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
        if rec.content_type == "sketch" || rec.content_type == "image" {
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
```

- [ ] **Step 3: 修改 load_bootstrap_state 返回 page_cache**

修改 `load_bootstrap_state` 方法：

```rust
pub fn load_bootstrap_state(&self) -> anyhow::Result<BootstrapState> {
    let config: AppConfig = load_json(&self.data_dir.join("config.json"))?;
    let api_keys = load_json(&self.data_dir.join("api_keys.json"))?;
    let devices = load_json(&self.data_dir.join("devices.json"))?;
    let todos = self.load_todos()?;
    let text_templates = load_json(&self.data_dir.join("text_templates.json"))?;
    let image_templates = load_json(&self.data_dir.join("image_templates.json"))?;
    let page_cache = self.load_page_cache()?;

    Ok(BootstrapState {
        api_keys,
        devices,
        todos,
        text_templates,
        image_templates,
        last_sync_time: config.last_sync_time,
        page_cache,
    })
}
```

- [ ] **Step 4: 添加 save_page_cache 辅助方法**

在 `load_page_cache` 之后添加：

```rust
fn save_page_cache(&self, cache: &[PageCacheRecord]) -> anyhow::Result<()> {
    save_json(&self.data_dir.join("page_cache.json"), cache)
}
```

- [ ] **Step 5: 添加写入缓存的方法**

添加各类型推送写入缓存的方法：

```rust
fn save_page_cache_record(&self, record: PageCacheRecord) -> anyhow::Result<()> {
    let path = self.data_dir.join("page_cache.json");
    let mut cache: Vec<PageCacheRecord> = if path.exists() {
        self.load_page_cache()?
    } else {
        Vec::new()
    };
    
    // 移除同设备同页面的旧记录
    cache.retain(|p| 
        !p.device_id.eq_ignore_ascii_case(&record.device_id) || p.page_id != record.page_id
    );
    
    // 如果有旧的缩略图文件，删除它
    let old = cache.iter().find(|p| 
        p.device_id.eq_ignore_ascii_case(&record.device_id) && p.page_id == record.page_id
    );
    if let Some(old_rec) = old {
        if old_rec.content_type == "sketch" || old_rec.content_type == "image" {
            if let Some(thumbnail_path) = &old_rec.thumbnail {
                let thumb_path = self.data_dir.join("thumbnails").join(thumbnail_path);
                if std::fs::metadata(&thumb_path).is_ok() {
                    std::fs::remove_file(&thumb_path)?;
                }
            }
        }
    }
    
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
```

- [ ] **Step 6: 编译验证**

Run: `cd src-tauri && cargo check`
Expected: 编译成功

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/state.rs
git commit -m "feat(state): add page cache methods to AppState"
```

---

## Task 4: 创建页面缓存命令模块

**Files:**
- Create: `src-tauri/src/commands/page_cache.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 创建 page_cache.rs**

```rust
#[tauri::command]
pub fn get_page_cache_list(
    state: tauri::State<'_, crate::state::AppState>,
    device_id: String,
) -> Result<Vec<crate::models::PageCacheRecord>, String> {
    state
        .get_page_cache_list(&device_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_page_cache(
    state: tauri::State<'_, crate::state::AppState>,
    device_id: String,
    page_id: u32,
) -> Result<(), String> {
    state
        .delete_page_cache(&device_id, page_id)
        .await
        .map_err(|e| e.to_string())
}
```

- [ ] **Step 2: 修改 commands/mod.rs**

添加：

```rust
pub mod page_cache;
```

- [ ] **Step 3: 修改 lib.rs 注册命令**

在 `invoke_handler` 中添加：

```rust
commands::page_cache::get_page_cache_list,
commands::page_cache::delete_page_cache,
```

- [ ] **Step 4: 编译验证**

Run: `cd src-tauri && cargo check`
Expected: 编译成功

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands/page_cache.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs
git commit -m "feat(commands): add page_cache command module"
```

---

## Task 5: 修改推送方法写入缓存

**Files:**
- Modify: `src-tauri/src/state.rs`

- [ ] **Step 1: 修改 push_sketch 写入缓存**

修改 `push_sketch` 方法：

```rust
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
    let thumbnail_filename = format!("sketch_{}_{}.png", device_id.replace(':', "_"), page_id);
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
```

- [ ] **Step 2: 修改 push_image_template 写入缓存**

修改 `push_image_template` 方法：

```rust
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
    let thumbnail_filename = format!("image_{}_{}.png", device_id.replace(':', "_"), page_id);
    let thumbnail = self.save_image_thumbnail(&image_bytes, &thumbnail_filename)?;
    
    let record = PageCacheRecord {
        device_id: device_id.to_string(),
        page_id,
        content_type: "image".to_string(),
        thumbnail: Some(thumbnail),
        metadata: Some(serde_json::json!({"name": template.name, "width": 400, "height": 300}).to_string()),
        pushed_at: now,
    };
    self.save_page_cache_record(record)?;
    
    Ok(())
}
```

- [ ] **Step 3: 修改 push_text 写入缓存**

修改 `push_text` 方法：

```rust
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
```

- [ ] **Step 4: 修改 push_structured_text 写入缓存**

修改 `push_structured_text` 方法：

```rust
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
        
        let record = PageCacheRecord {
            device_id: device_id.to_string(),
            page_id: pid,
            content_type: "structured_text".to_string(),
            thumbnail: Some(format!("{}{}", title, if body_preview.is_empty() { "" } else { "\n" } + &body_preview)),
            metadata: Some(serde_json::json!({"title": title, "bodyPreview": body_preview}).to_string()),
            pushed_at: now,
        };
        self.save_page_cache_record(record)?;
    }
    
    Ok(())
}
```

- [ ] **Step 5: 编译验证**

Run: `cd src-tauri && cargo check`
Expected: 编译成功

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/state.rs
git commit -m "feat(state): write page cache on push success"
```

---

## Task 6: 前端添加类型定义和调用函数

**Files:**
- Modify: `src/lib/tauri.ts`

- [ ] **Step 1: 添加 PageCacheRecord 类型**

在文件末尾添加：

```typescript
export type PageCacheRecord = {
  deviceId: string;
  pageId: number;
  contentType: "sketch" | "image" | "text" | "structured_text";
  thumbnail: string | null;
  metadata: Record<string, unknown> | null;
  pushedAt: string;
};

export async function getPageCacheList(deviceId: string): Promise<PageCacheRecord[]> {
  return invoke<PageCacheRecord[]>("get_page_cache_list", { deviceId });
}

export async function deletePageCache(deviceId: string, pageId: number): Promise<void> {
  return invoke("delete_page_cache", { deviceId, pageId });
}
```

- [ ] **Step 2: 修改 BootstrapState 类型**

在 `BootstrapState` 类型中添加 `pageCache` 字段：

```typescript
export type BootstrapState = {
  apiKeys: ApiKeyRecord[];
  devices: DeviceRecord[];
  todos: Array<TodoRecord>;
  textTemplates: Array<TextTemplateRecord>;
  imageTemplates: Array<ImageTemplateRecord>;
  lastSyncTime: string | null;
  pageCache: Array<PageCacheRecord>;
};
```

- [ ] **Step 3: 编译验证**

Run: `npm run build` 或 `pnpm build`
Expected: 编译成功

- [ ] **Step 4: Commit**

```bash
git add src/lib/tauri.ts
git commit -m "feat(tauri): add PageCacheRecord types and functions"
```

---

## Task 7: 创建 PageManagerPage 组件

**Files:**
- Create: `src/features/page-manager/page-manager-page.tsx`

- [ ] **Step 1: 创建组件文件**

```typescript
import { useState, useEffect } from "react";
import { Trash2, Image, Type, PenTool } from "lucide-react";
import { getPageCacheList, deletePageCache, type PageCacheRecord } from "../../lib/tauri";

type Device = { deviceId: string; alias: string; board: string };

const PAGE_IDS = [1, 2, 3, 4, 5];

function contentTypeIcon(type: string) {
  switch (type) {
    case "sketch":
      return PenTool;
    case "image":
      return Image;
    case "text":
    case "structured_text":
      return Type;
    default:
      return Type;
  }
}

function contentTypeLabel(type: string) {
  switch (type) {
    case "sketch":
      return "涂鸦";
    case "image":
      return "图片";
    case "text":
      return "文本";
    case "structured_text":
      return "结构化文本";
    default:
      return "未知";
  }
}

type Props = {
  devices: Device[];
};

export function PageManagerPage({ devices }: Props) {
  const [selectedDeviceId, setSelectedDeviceId] = useState<string>(
    devices[0]?.deviceId || ""
  );
  const [pageList, setPageList] = useState<PageCacheRecord[]>([]);
  const [loading, setLoading] = useState(false);
  const [deletingPageId, setDeletingPageId] = useState<number | null>(null);

  useEffect(() => {
    if (!selectedDeviceId) return;
    setLoading(true);
    getPageCacheList(selectedDeviceId)
      .then(setPageList)
      .finally(() => setLoading(false));
  }, [selectedDeviceId]);

  async function handleDelete(pageId: number) {
    setDeletingPageId(pageId);
    try {
      await deletePageCache(selectedDeviceId, pageId);
      const updated = await getPageCacheList(selectedDeviceId);
      setPageList(updated);
    } catch (e) {
      console.error("删除页面失败:", e);
    } finally {
      setDeletingPageId(null);
    }
  }

  function getPageData(pageId: number): PageCacheRecord | undefined {
    return pageList.find((p) => p.pageId === pageId);
  }

  function renderThumbnail(record: PageCacheRecord) {
    if (record.contentType === "sketch" || record.contentType === "image") {
      // 图片缩略图：thumbnail 是文件名，需要通过 tauri 读取
      // 这里简化处理，显示图标占位
      const Icon = contentTypeIcon(record.contentType);
      return (
        <div className="flex items-center justify-center h-24 bg-gray-100 rounded">
          <Icon size={32} className="text-gray-400" />
        </div>
      );
    }
    // 文本类型：thumbnail 是文本预览
    return (
      <div className="h-24 bg-gray-100 rounded p-2 text-sm text-gray-600 overflow-hidden">
        {record.thumbnail || ""}
      </div>
    );
  }

  return (
    <section className="space-y-6">
      <div>
        <h2 className="text-lg font-semibold">页面管理</h2>
        <p className="text-sm text-gray-500">
          查看和管理每个设备的墨水屏页面内容。
        </p>
      </div>

      {devices.length === 0 && (
        <div className="text-sm text-gray-500">请先在设置中添加设备。</div>
      )}

      {devices.length > 0 && (
        <div className="space-y-4">
          <div className="space-y-2">
            <label htmlFor="device-select" className="block text-sm font-medium">
              选择设备
            </label>
            <select
              id="device-select"
              value={selectedDeviceId}
              onChange={(e) => setSelectedDeviceId(e.target.value)}
              className="px-3 py-2 border border-gray-300 rounded-md dark:border-gray-600 dark:bg-gray-700"
            >
              {devices.map((d) => (
                <option key={d.deviceId} value={d.deviceId}>
                  {d.alias || d.deviceId}
                </option>
              ))}
            </select>
          </div>

          {loading && <div className="text-sm text-gray-500">加载中...</div>}

          {!loading && (
            <div className="grid grid-cols-5 gap-4">
              {PAGE_IDS.map((pageId) => {
                const record = getPageData(pageId);
                const Icon = record ? contentTypeIcon(record.contentType) : null;
                const isDeleting = deletingPageId === pageId;

                return (
                  <div
                    key={pageId}
                    className="p-4 rounded-xl border border-gray-200 bg-white/85 shadow-sm space-y-2"
                  >
                    <div className="flex items-center justify-between">
                      <span className="text-sm font-medium">第 {pageId} 页</span>
                      {Icon && (
                        <Icon size={16} className="text-gray-400" />
                      )}
                    </div>

                    {record ? (
                      <>
                        {renderThumbnail(record)}
                        <div className="text-xs text-gray-500">
                          {contentTypeLabel(record.contentType)}
                        </div>
                        <button
                          type="button"
                          onClick={() => handleDelete(pageId)}
                          disabled={isDeleting}
                          className="w-full flex items-center justify-center gap-1 px-2 py-1 text-sm text-red-600 hover:bg-red-50 rounded disabled:opacity-50"
                        >
                          <Trash2 size={14} />
                          {isDeleting ? "删除中..." : "删除"}
                        </button>
                      </>
                    ) : (
                      <div className="flex items-center justify-center h-32 text-sm text-gray-400">
                        暂无内容
                      </div>
                    )}
                  </div>
                );
              })}
            </div>
          )}
        </div>
      )}
    </section>
  );
}
```

- [ ] **Step 2: 编译验证**

Run: `npm run build` 或 `pnpm build`
Expected: 编译成功

- [ ] **Step 3: Commit**

```bash
git add src/features/page-manager/page-manager-page.tsx
git commit -m "feat(page-manager): create PageManagerPage component"
```

---

## Task 8: 添加页面管理路由和菜单项

**Files:**
- Modify: `src/components/layout/app-sidebar.tsx`
- Modify: `src/app/App.tsx`

- [ ] **Step 1: 在 app-sidebar.tsx 添加菜单项**

修改 import：

```typescript
import { NavLink } from "react-router-dom";
import { CheckSquare, FileText, Image, Settings, PenTool, LayoutGrid, Layers } from "lucide-react";
import { useWindowDrag } from "../../hooks/use-window-drag";
```

修改 `primaryNavItems`：

```typescript
const primaryNavItems = [
  { label: "待办事项", icon: CheckSquare, href: "/" },
  { label: "涂鸦推送", icon: PenTool, href: "/sketch-push" },
  { label: "图片推送", icon: Image, href: "/image-push" },
  { label: "自由排版", icon: LayoutGrid, href: "/free-layout" },
  { label: "文本推送", icon: FileText, href: "/text-push" },
  { label: "页面管理", icon: Layers, href: "/page-manager" },
];
```

- [ ] **Step 2: 在 App.tsx 添加路由**

修改 import：

```typescript
import { useEffect, useState } from "react";
import { useLocation } from "react-router-dom";
import { AppSidebar } from "../components/layout/app-sidebar";
import { AppToolbar } from "../components/layout/app-toolbar";
import { FreeLayoutPage } from "../features/free-layout/free-layout-page";
import { ImageTemplatesPage } from "../features/images/image-templates-page";
import { PageManagerPage } from "../features/page-manager/page-manager-page";
import { SettingsPage } from "../features/settings/settings-page";
import { SketchPage } from "../features/sketch/sketch-page";
import { TextTemplatesPage } from "../features/templates/text-templates-page";
import { TodoListPage } from "../features/todos/todo-list-page";
import {
  addDeviceCache,
  addApiKey,
  removeApiKey,
  createLocalTodo,
  deleteLocalTodo,
  deleteImageTemplate,
  getImageThumbnail,
  loadBootstrapState,
  pushFreeLayoutText,
  pushImageTemplate,
  pushSketch,
  pushText,
  pushTodoToDevice,
  removeDeviceCache,
  saveImageTemplate,
  syncAll,
  toggleTodoStatus,
  updateLocalTodo,
  type BootstrapState,
} from "../lib/tauri";
import type { SyncState } from "../features/sync/sync-status";
```

修改 `sectionTitles`：

```typescript
const sectionTitles: Record<string, string> = {
  "/": "待办事项",
  "/sketch-push": "涂鸦推送",
  "/image-push": "图片推送",
  "/free-layout": "自由排版",
  "/text-push": "文本推送",
  "/page-manager": "页面管理",
  "/settings": "设置",
};
```

在 `renderContent` 函数中添加路由：

```typescript
function renderContent() {
  if (path === "/settings") {
    return (
      <SettingsPage
        apiKeys={state.apiKeys}
        devices={state.devices}
        onAddApiKey={async (name, key) => {
          const record = await addApiKey(name, key);
          reload();
          return record;
        }}
        onRemoveApiKey={async (id) => {
          await removeApiKey(id);
          reload();
        }}
        onAddDevice={async (id, apiKeyId) => {
          const device = await addDeviceCache(id, apiKeyId);
          reload();
          return device;
        }}
        onRemoveDevice={async (id) => {
          await removeDeviceCache(id);
          reload();
        }}
      />
    );
  }
  if (path === "/page-manager") {
    return (
      <PageManagerPage
        devices={state.devices}
      />
    );
  }
  if (path === "/text-push") {
    // ... 现有代码
  }
  // ... 其他路由
}
```

- [ ] **Step 3: 编译验证**

Run: `npm run build` 或 `pnpm build`
Expected: 编译成功

- [ ] **Step 4: Commit**

```bash
git add src/components/layout/app-sidebar.tsx src/app/App.tsx
git commit -m "feat: add page manager route and sidebar menu"
```

---

## Task 9: 端到端测试验证

**Files:**
- 无文件修改，仅运行验证

- [ ] **Step 1: 启动开发服务器**

Run: `pnpm tauri dev`
Expected: 应用启动成功

- [ ] **Step 2: 手动测试流程**

测试步骤：
1. 打开应用，确认侧边栏有"页面管理"菜单
2. 点击进入页面管理，确认显示设备选择器和5个空页面卡片
3. 切换到涂鸦推送，绘制内容，推送到第1页
4. 返回页面管理，确认第1页显示"涂鸦"标识
5. 点击删除按钮，确认第1页变空
6. 切换到文本推送，推送到第2页
7. 返回页面管理，确认第2页显示文本预览

- [ ] **Step 3: 最终 Commit**

```bash
git add -A
git commit -m "feat: complete page manager feature implementation"
```

---

## Scope Check Summary

| 设计文档要求 | 对应任务 |
|------------|---------|
| 新增 page_cache 数据结构 | Task 1, 3 |
| 新增 get_page_cache_list 命令 | Task 3, 4 |
| 新增 delete_page_cache 命令 | Task 2, 3, 4 |
| 推送成功写入缓存 | Task 5 |
| load_bootstrap_state 返回 pageCache | Task 3 |
| 前端 PageManagerPage 组件 | Task 7 |
| 侧边栏菜单项 | Task 8 |
| 路由配置 | Task 8 |
| 前端类型定义 | Task 6 |