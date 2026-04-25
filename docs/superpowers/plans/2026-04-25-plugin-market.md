# Plugin Market Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a plugin market foundation where custom JS plugins run in the Rust backend, return a validated output object, and can be pushed once or on a recurring loop.

**Architecture:** The frontend owns plugin management UI and previews; the Rust backend owns plugin persistence, JS execution, output validation, text-to-image rendering, push dispatch, page cache updates, and loop task scheduling. Custom and future built-in plugins share the same output contract and push pipeline.

**Tech Stack:** React 19, Tauri 2, Rust, Vitest, Testing Library, `reqwest`, `image`, `base64`, `rquickjs`, `rquickjs-serde`, `fontdue`.

---

## File Structure

Create these backend files:

- `src-tauri/src/commands/plugins.rs`: Tauri command wrappers for custom plugins, plugin runs, plugin pushes, plugin loop tasks, and text-image preview.
- `src-tauri/src/plugin_output.rs`: Serde models and validation for `text`, `textImage`, and `image` plugin outputs.
- `src-tauri/src/plugin_runtime.rs`: Embedded JavaScript execution with injected `fetchJson` and `fetchText` helpers.
- `src-tauri/src/text_image.rs`: Render text plus style into a `400x300` PNG.
- `src-tauri/src/plugin_tasks.rs`: Start/stop logic and one-shot execution for recurring plugin loop tasks.

Modify these backend files:

- `src-tauri/Cargo.toml`: Add JS runtime and font rasterization dependencies.
- `src-tauri/src/models.rs`: Add custom plugin, plugin run, output preview, and plugin loop task types.
- `src-tauri/src/state.rs`: Add persistence and push orchestration methods.
- `src-tauri/src/lib.rs`: Register plugin commands.
- `src-tauri/src/commands/mod.rs`: Export `plugins`.

Create these frontend files:

- `src/features/plugins/plugin-market-page.tsx`: Plugin market UI.
- `src/features/plugins/plugin-market-page.test.tsx`: UI behavior tests.

Modify these frontend files:

- `src/lib/tauri.ts`: TypeScript types and invoke wrappers.
- `src/app/App.tsx`: Route `/plugins`, bootstrap plugin data, and pass callbacks.
- `src/components/layout/app-sidebar.tsx`: Add “插件市场” nav item.
- `src/app/App.test.tsx`: Sidebar and route coverage.

Data files in the user data directory:

- `custom_plugins.json`
- `plugin_loop_tasks.json`

---

### Task 1: Add Backend Models And Persistence

**Files:**
- Modify: `src-tauri/src/models.rs`
- Modify: `src-tauri/src/state.rs`
- Test: `src-tauri/src/state.rs`

- [ ] **Step 1: Write failing persistence tests**

Add these tests inside the existing `#[cfg(test)] mod tests` in `src-tauri/src/state.rs`:

```rust
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
```

- [ ] **Step 2: Run tests to verify they fail**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml custom_plugin_crud_persists_to_json plugin_loop_task_crud_persists_to_json
```

Expected: FAIL because `CustomPluginInput`, `PluginLoopTaskInput`, and persistence methods do not exist.

- [ ] **Step 3: Add model types**

In `src-tauri/src/models.rs`, extend `BootstrapState`:

```rust
#[serde(default)]
pub custom_plugins: Vec<CustomPluginRecord>,
#[serde(default)]
pub plugin_loop_tasks: Vec<PluginLoopTaskRecord>,
```

Add these model types near the existing stock and image loop models:

```rust
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
}
```

- [ ] **Step 4: Implement persistence methods**

In `src-tauri/src/state.rs`, import the new types:

```rust
CustomPluginInput, CustomPluginRecord, PluginLoopTaskInput, PluginLoopTaskRecord,
```

Add these methods inside `impl AppState`:

```rust
fn load_custom_plugins(&self) -> anyhow::Result<Vec<CustomPluginRecord>> {
    load_json(&self.data_dir.join("custom_plugins.json"))
}

pub fn list_custom_plugins(&self) -> anyhow::Result<Vec<CustomPluginRecord>> {
    self.load_custom_plugins()
}

fn save_custom_plugins(&self, records: &[CustomPluginRecord]) -> anyhow::Result<()> {
    save_json(&self.data_dir.join("custom_plugins.json"), &records)
}

pub fn save_custom_plugin(&self, input: CustomPluginInput) -> anyhow::Result<CustomPluginRecord> {
    let mut records = self.load_custom_plugins()?;
    let now = chrono::Utc::now().to_rfc3339();

    if let Some(id) = input.id {
        let record = records
            .iter_mut()
            .find(|plugin| plugin.id == id)
            .ok_or_else(|| anyhow::anyhow!("插件 {id} 不存在"))?;
        record.name = input.name.trim().to_string();
        record.description = input.description.trim().to_string();
        record.code = input.code;
        record.updated_at = now;
        let updated = record.clone();
        self.save_custom_plugins(&records)?;
        return Ok(updated);
    }

    let next_id = records.iter().map(|plugin| plugin.id).max().unwrap_or(0) + 1;
    let record = CustomPluginRecord {
        id: next_id,
        name: input.name.trim().to_string(),
        description: input.description.trim().to_string(),
        code: input.code,
        created_at: now.clone(),
        updated_at: now,
    };
    records.push(record.clone());
    self.save_custom_plugins(&records)?;
    Ok(record)
}

pub fn delete_custom_plugin(&self, plugin_id: i64) -> anyhow::Result<()> {
    let mut records = self.load_custom_plugins()?;
    records.retain(|plugin| plugin.id != plugin_id);
    self.save_custom_plugins(&records)
}

fn load_plugin_loop_tasks(&self) -> anyhow::Result<Vec<PluginLoopTaskRecord>> {
    load_json(&self.data_dir.join("plugin_loop_tasks.json"))
}

pub fn list_plugin_loop_tasks(&self) -> anyhow::Result<Vec<PluginLoopTaskRecord>> {
    self.load_plugin_loop_tasks()
}

fn save_plugin_loop_tasks(&self, records: &[PluginLoopTaskRecord]) -> anyhow::Result<()> {
    save_json(&self.data_dir.join("plugin_loop_tasks.json"), &records)
}

pub fn create_plugin_loop_task(
    &self,
    input: PluginLoopTaskInput,
) -> anyhow::Result<PluginLoopTaskRecord> {
    let mut records = self.load_plugin_loop_tasks()?;
    let next_id = records.iter().map(|task| task.id).max().unwrap_or(0) + 1;
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
        status: "idle".into(),
        last_push_at: None,
        error_message: None,
        created_at: now.clone(),
        updated_at: now,
    };
    records.push(record.clone());
    self.save_plugin_loop_tasks(&records)?;
    Ok(record)
}

pub fn update_plugin_loop_task(
    &self,
    task_id: i64,
    input: PluginLoopTaskInput,
) -> anyhow::Result<PluginLoopTaskRecord> {
    let mut records = self.load_plugin_loop_tasks()?;
    let task = records
        .iter_mut()
        .find(|task| task.id == task_id)
        .ok_or_else(|| anyhow::anyhow!("插件循环任务 {task_id} 不存在"))?;
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
    self.save_plugin_loop_tasks(&records)?;
    Ok(updated)
}

pub fn delete_plugin_loop_task(&self, task_id: i64) -> anyhow::Result<()> {
    let mut records = self.load_plugin_loop_tasks()?;
    records.retain(|task| task.id != task_id);
    self.save_plugin_loop_tasks(&records)
}
```

Update `load_bootstrap_state` to load and return the new lists.

- [ ] **Step 5: Run tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml custom_plugin_crud_persists_to_json plugin_loop_task_crud_persists_to_json
```

Expected: PASS.

- [ ] **Step 6: Commit**

```powershell
git add src-tauri/src/models.rs src-tauri/src/state.rs
git commit -m "feat: persist plugin records"
```

---

### Task 2: Implement Plugin Output Validation

**Files:**
- Create: `src-tauri/src/plugin_output.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/src/plugin_output.rs`

- [ ] **Step 1: Write output validation tests**

Create `src-tauri/src/plugin_output.rs` with the tests first:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_text_output_with_default_font_size() {
        let raw = serde_json::json!({ "type": "text", "text": "hello" });

        let parsed = parse_plugin_output(raw).unwrap();

        match parsed {
            PluginOutput::Text(output) => {
                assert_eq!(output.text, "hello");
                assert_eq!(output.font_size, 20);
            }
            _ => panic!("expected text output"),
        }
    }

    #[test]
    fn parses_text_image_output_with_default_style() {
        let raw = serde_json::json!({ "type": "textImage", "text": "第一行\n第二行" });

        let parsed = parse_plugin_output(raw).unwrap();

        match parsed {
            PluginOutput::TextImage(output) => {
                assert_eq!(output.text, "第一行\n第二行");
                assert_eq!(output.style.font_size, 20);
                assert_eq!(output.style.align, "left");
                assert_eq!(output.style.vertical_align, "top");
            }
            _ => panic!("expected textImage output"),
        }
    }

    #[test]
    fn parses_image_data_url() {
        let raw = serde_json::json!({
            "type": "image",
            "imageDataUrl": "data:image/png;base64,iVBORw0KGgo="
        });

        let parsed = parse_plugin_output(raw).unwrap();

        match parsed {
            PluginOutput::Image(output) => {
                assert!(output.image_data_url.starts_with("data:image/png;base64,"));
            }
            _ => panic!("expected image output"),
        }
    }

    #[test]
    fn rejects_missing_type() {
        let raw = serde_json::json!({ "text": "hello" });

        let err = parse_plugin_output(raw).unwrap_err().to_string();

        assert!(err.contains("缺少 type"));
    }

    #[test]
    fn rejects_empty_text() {
        let raw = serde_json::json!({ "type": "text", "text": "   " });

        let err = parse_plugin_output(raw).unwrap_err().to_string();

        assert!(err.contains("text 不能为空"));
    }

    #[test]
    fn rejects_invalid_alignment() {
        let raw = serde_json::json!({
            "type": "textImage",
            "text": "hello",
            "style": { "align": "justify" }
        });

        let err = parse_plugin_output(raw).unwrap_err().to_string();

        assert!(err.contains("align"));
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml plugin_output
```

Expected: FAIL because production types and `parse_plugin_output` do not exist.

- [ ] **Step 3: Implement output parser**

Replace `src-tauri/src/plugin_output.rs` with:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextImageStyle {
    #[serde(default = "default_font_size")]
    pub font_size: u32,
    #[serde(default = "default_line_height")]
    pub line_height: f32,
    #[serde(default = "default_padding")]
    pub padding: u32,
    #[serde(default = "default_align")]
    pub align: String,
    #[serde(default = "default_vertical_align")]
    pub vertical_align: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextPluginOutput {
    pub text: String,
    #[serde(default = "default_font_size")]
    pub font_size: u32,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextImagePluginOutput {
    pub text: String,
    #[serde(default)]
    pub style: TextImageStyle,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImagePluginOutput {
    pub image_data_url: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum PluginOutput {
    #[serde(rename = "text")]
    Text(TextPluginOutput),
    #[serde(rename = "textImage")]
    TextImage(TextImagePluginOutput),
    #[serde(rename = "image")]
    Image(ImagePluginOutput),
}

impl Default for TextImageStyle {
    fn default() -> Self {
        Self {
            font_size: default_font_size(),
            line_height: default_line_height(),
            padding: default_padding(),
            align: default_align(),
            vertical_align: default_vertical_align(),
        }
    }
}

fn default_font_size() -> u32 {
    20
}

fn default_line_height() -> f32 {
    1.25
}

fn default_padding() -> u32 {
    16
}

fn default_align() -> String {
    "left".into()
}

fn default_vertical_align() -> String {
    "top".into()
}

fn validate_font_size(font_size: u32) -> anyhow::Result<()> {
    if !(8..=72).contains(&font_size) {
        anyhow::bail!("fontSize 必须在 8 到 72 之间");
    }
    Ok(())
}

fn validate_style(style: &TextImageStyle) -> anyhow::Result<()> {
    validate_font_size(style.font_size)?;
    if !(0.8..=3.0).contains(&style.line_height) {
        anyhow::bail!("lineHeight 必须在 0.8 到 3.0 之间");
    }
    if style.padding > 120 {
        anyhow::bail!("padding 不能超过 120");
    }
    if !matches!(style.align.as_str(), "left" | "center" | "right") {
        anyhow::bail!("align 只能是 left、center 或 right");
    }
    if !matches!(style.vertical_align.as_str(), "top" | "middle") {
        anyhow::bail!("verticalAlign 只能是 top 或 middle");
    }
    Ok(())
}

fn require_text(text: &str) -> anyhow::Result<()> {
    if text.trim().is_empty() {
        anyhow::bail!("text 不能为空");
    }
    if text.len() > 256 * 1024 {
        anyhow::bail!("text 超过 256KB 限制");
    }
    Ok(())
}

pub fn parse_plugin_output(raw: serde_json::Value) -> anyhow::Result<PluginOutput> {
    let output_type = raw
        .get("type")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("插件输出缺少 type"))?;

    match output_type {
        "text" => {
            let output: TextPluginOutput = serde_json::from_value(raw)?;
            require_text(&output.text)?;
            validate_font_size(output.font_size)?;
            Ok(PluginOutput::Text(output))
        }
        "textImage" => {
            let output: TextImagePluginOutput = serde_json::from_value(raw)?;
            require_text(&output.text)?;
            validate_style(&output.style)?;
            Ok(PluginOutput::TextImage(output))
        }
        "image" => {
            let output: ImagePluginOutput = serde_json::from_value(raw)?;
            if !output.image_data_url.starts_with("data:image/") {
                anyhow::bail!("imageDataUrl 必须是 data:image/...;base64 格式");
            }
            if !output.image_data_url.contains(";base64,") {
                anyhow::bail!("imageDataUrl 必须包含 base64 图片数据");
            }
            Ok(PluginOutput::Image(output))
        }
        other => anyhow::bail!("不支持的插件输出 type: {other}"),
    }
}
```

Keep the tests from Step 1 at the bottom of the file.

- [ ] **Step 4: Register module**

In `src-tauri/src/lib.rs`, add:

```rust
pub mod plugin_output;
```

- [ ] **Step 5: Run tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml plugin_output
```

Expected: PASS.

- [ ] **Step 6: Commit**

```powershell
git add src-tauri/src/plugin_output.rs src-tauri/src/lib.rs
git commit -m "feat: validate plugin outputs"
```

---

### Task 3: Add Text-To-Image Rendering

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Create: `src-tauri/src/text_image.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/src/text_image.rs`

- [ ] **Step 1: Add dependencies**

Run:

```powershell
cargo add fontdue --manifest-path src-tauri/Cargo.toml
```

Expected: `fontdue` appears under `[dependencies]`.

- [ ] **Step 2: Add a Chinese-capable font asset**

Add an open-source font file at:

```text
assets/fonts/NotoSansSC-Regular.otf
```

Use Noto Sans SC Regular from Google Fonts or another open-source Chinese font. Add a small license note:

```text
assets/fonts/README.md
```

with content:

```markdown
# Font Assets

`NotoSansSC-Regular.otf` is used by the plugin text-image renderer for Chinese text. It is distributed under the SIL Open Font License.
```

- [ ] **Step 3: Write failing renderer tests**

Create `src-tauri/src/text_image.rs` with tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin_output::TextImageStyle;

    #[test]
    fn renders_text_to_png_bytes() {
        let png = render_text_to_png("第一行\n第二行", &TextImageStyle::default()).unwrap();

        assert!(png.starts_with(&[137, 80, 78, 71]));
        assert!(png.len() > 1000);
    }

    #[test]
    fn accepts_center_alignment() {
        let style = TextImageStyle {
            align: "center".into(),
            ..TextImageStyle::default()
        };

        let png = render_text_to_png("居中", &style).unwrap();

        assert!(png.starts_with(&[137, 80, 78, 71]));
    }

    #[test]
    fn truncates_long_text_without_failing() {
        let text = "很长的文字".repeat(500);

        let png = render_text_to_png(&text, &TextImageStyle::default()).unwrap();

        assert!(png.starts_with(&[137, 80, 78, 71]));
    }
}
```

- [ ] **Step 4: Run tests to verify they fail**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml text_image
```

Expected: FAIL because `render_text_to_png` does not exist.

- [ ] **Step 5: Implement renderer**

Implement `src-tauri/src/text_image.rs`:

```rust
use crate::plugin_output::TextImageStyle;
use fontdue::Font;
use image::{ImageBuffer, ImageFormat, Rgba};
use std::io::Cursor;

const WIDTH: u32 = 400;
const HEIGHT: u32 = 300;

static FONT_BYTES: &[u8] = include_bytes!("../../assets/fonts/NotoSansSC-Regular.otf");

pub fn render_text_to_png(text: &str, style: &TextImageStyle) -> anyhow::Result<Vec<u8>> {
    let font = Font::from_bytes(FONT_BYTES, fontdue::FontSettings::default())
        .map_err(|e| anyhow::anyhow!("字体加载失败: {e:?}"))?;
    let mut image = ImageBuffer::from_pixel(WIDTH, HEIGHT, Rgba([255, 255, 255, 255]));
    let font_size = style.font_size as f32;
    let line_height = (font_size * style.line_height).ceil() as i32;
    let padding = style.padding as i32;
    let max_width = WIDTH as i32 - padding * 2;
    let max_height = HEIGHT as i32 - padding * 2;

    let lines = wrap_text(text, &font, font_size, max_width.max(1));
    let max_lines = (max_height / line_height.max(1)).max(1) as usize;
    let mut visible = lines.into_iter().take(max_lines).collect::<Vec<_>>();
    if visible.len() == max_lines && text_width(visible.last().map(String::as_str).unwrap_or(""), &font, font_size) > max_width {
        if let Some(last) = visible.last_mut() {
            trim_to_width(last, &font, font_size, max_width);
        }
    }
    if visible.len() == max_lines {
        if let Some(last) = visible.last_mut() {
            append_ellipsis(last, &font, font_size, max_width);
        }
    }

    let total_height = visible.len() as i32 * line_height;
    let start_y = if style.vertical_align == "middle" {
        ((HEIGHT as i32 - total_height) / 2).max(padding)
    } else {
        padding
    };

    for (line_index, line) in visible.iter().enumerate() {
        let width = text_width(line, &font, font_size);
        let x = match style.align.as_str() {
            "center" => padding + ((max_width - width) / 2).max(0),
            "right" => padding + (max_width - width).max(0),
            _ => padding,
        };
        let baseline_y = start_y + line_index as i32 * line_height;
        draw_line(&mut image, &font, font_size, x, baseline_y, line);
    }

    let mut cursor = Cursor::new(Vec::new());
    image.write_to(&mut cursor, ImageFormat::Png)?;
    Ok(cursor.into_inner())
}

fn wrap_text(text: &str, font: &Font, font_size: f32, max_width: i32) -> Vec<String> {
    let mut lines = Vec::new();
    for paragraph in text.lines() {
        let mut current = String::new();
        for ch in paragraph.chars() {
            let candidate = format!("{current}{ch}");
            if !current.is_empty() && text_width(&candidate, font, font_size) > max_width {
                lines.push(current);
                current = ch.to_string();
            } else {
                current = candidate;
            }
        }
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

fn text_width(text: &str, font: &Font, font_size: f32) -> i32 {
    text.chars()
        .map(|ch| font.metrics(ch, font_size).advance_width.ceil() as i32)
        .sum()
}

fn trim_to_width(text: &mut String, font: &Font, font_size: f32, max_width: i32) {
    while !text.is_empty() && text_width(text, font, font_size) > max_width {
        text.pop();
    }
}

fn append_ellipsis(text: &mut String, font: &Font, font_size: f32, max_width: i32) {
    let ellipsis = "...";
    while !text.is_empty() && text_width(&format!("{text}{ellipsis}"), font, font_size) > max_width {
        text.pop();
    }
    text.push_str(ellipsis);
}

fn draw_line(
    image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    font: &Font,
    font_size: f32,
    start_x: i32,
    start_y: i32,
    text: &str,
) {
    let mut cursor_x = start_x;
    for ch in text.chars() {
        let (metrics, bitmap) = font.rasterize(ch, font_size);
        let glyph_x = cursor_x + metrics.xmin;
        let glyph_y = start_y + metrics.ymin + font_size as i32;
        for y in 0..metrics.height {
            for x in 0..metrics.width {
                let alpha = bitmap[y * metrics.width + x];
                if alpha == 0 {
                    continue;
                }
                let px = glyph_x + x as i32;
                let py = glyph_y + y as i32;
                if px >= 0 && py >= 0 && px < WIDTH as i32 && py < HEIGHT as i32 {
                    image.put_pixel(px as u32, py as u32, Rgba([0, 0, 0, alpha]));
                }
            }
        }
        cursor_x += metrics.advance_width.ceil() as i32;
    }
}
```

- [ ] **Step 6: Register module and run tests**

In `src-tauri/src/lib.rs`, add:

```rust
pub mod text_image;
```

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml text_image
```

Expected: PASS.

- [ ] **Step 7: Commit**

```powershell
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/src/text_image.rs src-tauri/src/lib.rs assets/fonts
git commit -m "feat: render plugin text images"
```

---

### Task 4: Implement Backend JS Runtime

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Create: `src-tauri/src/plugin_runtime.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/src/plugin_runtime.rs`

- [ ] **Step 1: Add JS dependencies**

Run:

```powershell
cargo add rquickjs rquickjs-serde --manifest-path src-tauri/Cargo.toml
```

Expected: `rquickjs` and `rquickjs-serde` appear under `[dependencies]`.

- [ ] **Step 2: Write runtime tests**

Create `src-tauri/src/plugin_runtime.rs` with tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn runs_sync_returning_plugin_code() {
        let output = run_plugin_code("return { type: 'text', text: 'hello' };")
            .await
            .unwrap();

        assert_eq!(output["type"], "text");
        assert_eq!(output["text"], "hello");
    }

    #[tokio::test]
    async fn allows_await_on_plain_helper_values() {
        let output = run_plugin_code("const value = await echoJson({ ok: true }); return { type: 'text', text: String(value.ok) };")
            .await
            .unwrap();

        assert_eq!(output["text"], "true");
    }

    #[tokio::test]
    async fn reports_js_errors() {
        let err = run_plugin_code("throw new Error('boom');")
            .await
            .unwrap_err()
            .to_string();

        assert!(err.contains("boom"));
    }
}
```

- [ ] **Step 3: Run tests to verify they fail**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml plugin_runtime
```

Expected: FAIL because `run_plugin_code` does not exist.

- [ ] **Step 4: Implement runtime skeleton**

Implement `src-tauri/src/plugin_runtime.rs` with this public API. Keep `run_plugin_code` as the only function used by the rest of the app:

```rust
use rquickjs::{Context, Function, Runtime};
use std::time::Duration;

pub async fn run_plugin_code(code: &str) -> anyhow::Result<serde_json::Value> {
    let code = code.to_string();
    tokio::time::timeout(Duration::from_secs(20), tokio::task::spawn_blocking(move || {
        run_plugin_code_blocking(&code)
    }))
    .await
    .map_err(|_| anyhow::anyhow!("插件执行超时"))?
    .map_err(|e| anyhow::anyhow!("插件执行失败: {e}"))?
}

fn run_plugin_code_blocking(code: &str) -> anyhow::Result<serde_json::Value> {
    let runtime = Runtime::new()?;
    let context = Context::full(&runtime)?;
    context.with(|ctx| {
        let globals = ctx.globals();
        let echo = Function::new(ctx.clone(), |value: rquickjs::Value| value)?;
        globals.set("echoJson", echo)?;

        let wrapped = format!(
            "(async function() {{\n{code}\n}})()"
        );
        let value: rquickjs::Value = ctx.eval(wrapped)?;
        let json = rquickjs_serde::from_value(value)?;
        Ok(json)
    })
}
```

Then add `fetchJson` and `fetchText` as blocking helper functions inside the QuickJS context:

```rust
fn fetch_json_blocking(url: String) -> anyhow::Result<serde_json::Value> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()?;
    let response = client.get(url).send()?;
    let status = response.status();
    if !status.is_success() {
        anyhow::bail!("HTTP 请求失败: {status}");
    }
    Ok(response.json()?)
}

fn fetch_text_blocking(url: String) -> anyhow::Result<String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()?;
    let response = client.get(url).send()?;
    let status = response.status();
    if !status.is_success() {
        anyhow::bail!("HTTP 请求失败: {status}");
    }
    Ok(response.text()?)
}
```

If `reqwest::blocking` is unavailable, enable the blocking feature:

```toml
reqwest = { version = "0.12", features = ["json", "rustls-tls", "multipart", "blocking"] }
```

- [ ] **Step 5: Register module and run tests**

In `src-tauri/src/lib.rs`, add:

```rust
pub mod plugin_runtime;
```

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml plugin_runtime
```

Expected: PASS.

- [ ] **Step 6: Commit**

```powershell
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/src/plugin_runtime.rs src-tauri/src/lib.rs
git commit -m "feat: run custom plugin javascript"
```

---

### Task 5: Implement Plugin Push Pipeline And Commands

**Files:**
- Create: `src-tauri/src/commands/plugins.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/state.rs`
- Test: `src-tauri/src/state.rs`

- [ ] **Step 1: Add state-level run and preview methods**

In `src-tauri/src/state.rs`, add:

```rust
fn find_custom_plugin(&self, plugin_id: &str) -> anyhow::Result<CustomPluginRecord> {
    let id: i64 = plugin_id.parse().map_err(|_| anyhow::anyhow!("插件 id 无效"))?;
    self.load_custom_plugins()?
        .into_iter()
        .find(|plugin| plugin.id == id)
        .ok_or_else(|| anyhow::anyhow!("插件 {plugin_id} 不存在"))
}

pub async fn run_plugin_once(
    &self,
    plugin_kind: &str,
    plugin_id: &str,
) -> anyhow::Result<crate::models::PluginRunResult> {
    let plugin = match plugin_kind {
        "custom" => self.find_custom_plugin(plugin_id)?,
        "builtin" => anyhow::bail!("内置插件尚未接入"),
        _ => anyhow::bail!("未知插件类型: {plugin_kind}"),
    };
    let raw = crate::plugin_runtime::run_plugin_code(&plugin.code).await?;
    let output = crate::plugin_output::parse_plugin_output(raw)?;
    self.plugin_output_to_run_result(&plugin, output)
}
```

Add `PluginRunResult` to `models.rs`:

```rust
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginRunResult {
    pub output_type: String,
    pub title: Option<String>,
    pub text: Option<String>,
    pub image_data_url: Option<String>,
    pub preview_png_base64: Option<String>,
    pub metadata: Option<serde_json::Value>,
}
```

- [ ] **Step 2: Add command wrappers**

Create `src-tauri/src/commands/plugins.rs`:

```rust
#[tauri::command]
pub fn list_custom_plugins(
    state: tauri::State<'_, crate::state::AppState>,
) -> Result<Vec<crate::models::CustomPluginRecord>, String> {
    state.list_custom_plugins().map_err(|e| e.to_string())
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
) -> Result<crate::models::PluginRunResult, String> {
    state
        .run_plugin_once(&plugin_kind, &plugin_id)
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
) -> Result<(), String> {
    state
        .push_plugin_once(&plugin_kind, &plugin_id, &device_id, page_id)
        .await
        .map_err(|e| e.to_string())
}
```

Export it in `src-tauri/src/commands/mod.rs`:

```rust
pub mod plugins;
```

Register commands in `src-tauri/src/lib.rs` inside `tauri::generate_handler!`.

- [ ] **Step 3: Implement push orchestration**

In `src-tauri/src/state.rs`, implement:

```rust
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
        _ => anyhow::bail!("未知插件类型: {plugin_kind}"),
    };
    let raw = crate::plugin_runtime::run_plugin_code(&plugin.code).await?;
    let output = crate::plugin_output::parse_plugin_output(raw)?;
    self.push_plugin_output(&plugin, plugin_kind, plugin_id, output, device_id, page_id)
        .await
}
```

Add this helper shape for `push_plugin_output`:

```rust
async fn push_plugin_output(
    &self,
    plugin: &CustomPluginRecord,
    plugin_kind: &str,
    plugin_id: &str,
    output: crate::plugin_output::PluginOutput,
    device_id: &str,
    page_id: u32,
) -> anyhow::Result<()> {
    let device = self
        .list_devices()?
        .into_iter()
        .find(|device| device.device_id.eq_ignore_ascii_case(device_id))
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
            self.save_page_cache_record(PageCacheRecord {
                device_id: device_id.to_string(),
                page_id,
                content_type: "plugin_text".into(),
                thumbnail: Some(text.text.chars().take(100).collect()),
                metadata: Some(plugin_cache_metadata(plugin, plugin_kind, plugin_id, "text", &text.title, &text.metadata)?),
                pushed_at: now,
            })?;
        }
        crate::plugin_output::PluginOutput::TextImage(text_image) => {
            let png = crate::text_image::render_text_to_png(&text_image.text, &text_image.style)?;
            crate::api::client::push_image(&api_key, device_id, png.clone(), page_id).await?;
            self.save_page_cache_record(PageCacheRecord {
                device_id: device_id.to_string(),
                page_id,
                content_type: "plugin_image".into(),
                thumbnail: Some(self.save_image_thumbnail(&png, &format!("plugin-{page_id}.png"))?),
                metadata: Some(plugin_cache_metadata(plugin, plugin_kind, plugin_id, "textImage", &text_image.title, &text_image.metadata)?),
                pushed_at: now,
            })?;
        }
        crate::plugin_output::PluginOutput::Image(image) => {
            let png = decode_plugin_image_to_png(&image.image_data_url)?;
            crate::api::client::push_image(&api_key, device_id, png.clone(), page_id).await?;
            self.save_page_cache_record(PageCacheRecord {
                device_id: device_id.to_string(),
                page_id,
                content_type: "plugin_image".into(),
                thumbnail: Some(self.save_image_thumbnail(&png, &format!("plugin-{page_id}.png"))?),
                metadata: Some(plugin_cache_metadata(plugin, plugin_kind, plugin_id, "image", &image.title, &image.metadata)?),
                pushed_at: now,
            })?;
        }
    }

    Ok(())
}

fn plugin_cache_metadata(
    plugin: &CustomPluginRecord,
    plugin_kind: &str,
    plugin_id: &str,
    output_type: &str,
    title: &Option<String>,
    metadata: &Option<serde_json::Value>,
) -> anyhow::Result<String> {
    Ok(serde_json::to_string(&serde_json::json!({
        "pluginName": plugin.name,
        "pluginKind": plugin_kind,
        "pluginId": plugin_id,
        "outputType": output_type,
        "title": title,
        "metadata": metadata,
        "runAt": chrono::Utc::now().to_rfc3339(),
    }))?)
}
```

Add `decode_plugin_image_to_png` in `state.rs` or a small plugin image helper:

```rust
fn decode_plugin_image_to_png(data_url: &str) -> anyhow::Result<Vec<u8>> {
    let (_, encoded) = data_url
        .split_once(";base64,")
        .ok_or_else(|| anyhow::anyhow!("imageDataUrl 必须包含 base64 图片数据"))?;
    let bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, encoded)?;
    let image = image::load_from_memory(&bytes)?;
    let processed = image.resize_to_fill(400, 300, image::imageops::FilterType::Lanczos3);
    let mut cursor = std::io::Cursor::new(Vec::new());
    processed.write_to(&mut cursor, image::ImageFormat::Png)?;
    Ok(cursor.into_inner())
}
```

- [ ] **Step 4: Run backend tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml plugin_output text_image plugin_runtime
```

Expected: PASS.

- [ ] **Step 5: Commit**

```powershell
git add src-tauri/src/commands/plugins.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs src-tauri/src/models.rs src-tauri/src/state.rs
git commit -m "feat: add plugin run commands"
```

---

### Task 6: Implement Plugin Loop Tasks

**Files:**
- Create: `src-tauri/src/plugin_tasks.rs`
- Modify: `src-tauri/src/commands/plugins.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/state.rs`
- Test: `src-tauri/src/state.rs`

- [ ] **Step 1: Write loop status tests**

Add tests in `src-tauri/src/state.rs`:

```rust
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
```

- [ ] **Step 2: Implement status methods**

In `src-tauri/src/state.rs`, add:

```rust
pub fn start_plugin_loop_task(&self, task_id: i64) -> anyhow::Result<PluginLoopTaskRecord> {
    let mut tasks = self.load_plugin_loop_tasks()?;
    let task = tasks
        .iter_mut()
        .find(|task| task.id == task_id)
        .ok_or_else(|| anyhow::anyhow!("插件循环任务 {task_id} 不存在"))?;
    task.status = "running".into();
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
    task.status = "idle".into();
    task.updated_at = chrono::Utc::now().to_rfc3339();
    let updated = task.clone();
    self.save_plugin_loop_tasks(&tasks)?;
    Ok(updated)
}
```

- [ ] **Step 3: Add command wrappers**

Extend `src-tauri/src/commands/plugins.rs`:

```rust
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
```

Register these commands in `src-tauri/src/lib.rs`.

- [ ] **Step 4: Implement background runner**

Create `src-tauri/src/plugin_tasks.rs` with:

```rust
use std::path::PathBuf;

pub async fn run_plugin_loop_task_tick(data_dir: PathBuf, task_id: i64) -> anyhow::Result<()> {
    let state = crate::state::AppState { data_dir };
    let task = state
        .list_plugin_loop_tasks()?
        .into_iter()
        .find(|task| task.id == task_id)
        .ok_or_else(|| anyhow::anyhow!("插件循环任务 {task_id} 不存在"))?;

    if task.status != "running" {
        return Ok(());
    }

    state
        .push_plugin_once(&task.plugin_kind, &task.plugin_id, &task.device_id, task.page_id)
        .await?;
    state.mark_plugin_loop_task_pushed(task_id)?;
    Ok(())
}
```

Add `mark_plugin_loop_task_pushed` and `mark_plugin_loop_task_error` methods to `AppState`, mirroring the existing image loop status update style.

- [ ] **Step 5: Run tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml start_plugin_loop_task_sets_running_status stop_plugin_loop_task_sets_idle_status
```

Expected: PASS.

- [ ] **Step 6: Commit**

```powershell
git add src-tauri/src/plugin_tasks.rs src-tauri/src/commands/plugins.rs src-tauri/src/lib.rs src-tauri/src/state.rs
git commit -m "feat: add plugin loop tasks"
```

---

### Task 7: Add Frontend Tauri Types And App Wiring

**Files:**
- Modify: `src/lib/tauri.ts`
- Modify: `src/app/App.tsx`
- Modify: `src/components/layout/app-sidebar.tsx`
- Modify: `src/app/App.test.tsx`

- [ ] **Step 1: Write route/sidebar test**

In `src/app/App.test.tsx`, add:

```tsx
test("renders plugin market in the sidebar", async () => {
  render(
    <MemoryRouter initialEntries={["/"]}>
      <App />
    </MemoryRouter>
  );

  expect(await screen.findByText("插件市场")).toBeInTheDocument();
});
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```powershell
pnpm vitest run src/app/App.test.tsx
```

Expected: FAIL because the sidebar has no plugin market item.

- [ ] **Step 3: Add Tauri TypeScript types and wrappers**

In `src/lib/tauri.ts`, add:

```ts
export type CustomPluginRecord = {
  id: number;
  name: string;
  description: string;
  code: string;
  createdAt: string;
  updatedAt: string;
};

export type CustomPluginInput = {
  id?: number;
  name: string;
  description: string;
  code: string;
};

export type PluginRunResult = {
  outputType: "text" | "textImage" | "image";
  title?: string;
  text?: string;
  imageDataUrl?: string;
  previewPngBase64?: string;
  metadata?: unknown;
};

export type PluginLoopTask = {
  id: number;
  pluginKind: "builtin" | "custom";
  pluginId: string;
  name: string;
  deviceId: string;
  pageId: number;
  intervalSeconds: number;
  durationType: "none" | "until_time" | "for_duration";
  endTime?: string;
  durationMinutes?: number;
  status: "idle" | "running" | "completed" | "error";
  lastPushAt?: string;
  errorMessage?: string;
  createdAt: string;
  updatedAt: string;
};

export type PluginLoopTaskInput = Omit<
  PluginLoopTask,
  "id" | "status" | "lastPushAt" | "errorMessage" | "createdAt" | "updatedAt"
>;
```

Extend `BootstrapState`:

```ts
customPlugins: Array<CustomPluginRecord>;
pluginLoopTasks: Array<PluginLoopTask>;
```

Add wrappers:

```ts
export async function saveCustomPlugin(input: CustomPluginInput): Promise<CustomPluginRecord> {
  return invoke<CustomPluginRecord>("save_custom_plugin", { input });
}

export async function deleteCustomPlugin(pluginId: number): Promise<void> {
  return invoke("delete_custom_plugin", { pluginId });
}

export async function runPluginOnce(pluginKind: "builtin" | "custom", pluginId: string): Promise<PluginRunResult> {
  return invoke<PluginRunResult>("run_plugin_once", { pluginKind, pluginId });
}

export async function pushPluginOnce(pluginKind: "builtin" | "custom", pluginId: string, deviceId: string, pageId: number): Promise<void> {
  return invoke("push_plugin_once", { pluginKind, pluginId, deviceId, pageId });
}
```

Add loop task wrappers:

```ts
export async function createPluginLoopTask(input: PluginLoopTaskInput): Promise<PluginLoopTask> {
  return invoke<PluginLoopTask>("create_plugin_loop_task", { input });
}

export async function updatePluginLoopTask(taskId: number, input: PluginLoopTaskInput): Promise<PluginLoopTask> {
  return invoke<PluginLoopTask>("update_plugin_loop_task", { taskId, input });
}

export async function deletePluginLoopTask(taskId: number): Promise<void> {
  return invoke("delete_plugin_loop_task", { taskId });
}

export async function startPluginLoopTask(taskId: number): Promise<PluginLoopTask> {
  return invoke<PluginLoopTask>("start_plugin_loop_task", { taskId });
}

export async function stopPluginLoopTask(taskId: number): Promise<PluginLoopTask> {
  return invoke<PluginLoopTask>("stop_plugin_loop_task", { taskId });
}
```

- [ ] **Step 4: Wire route and sidebar**

In `src/components/layout/app-sidebar.tsx`, import a lucide icon such as `Blocks` and add:

```ts
{ label: "插件市场", icon: Blocks, href: "/plugins" },
```

In `src/app/App.tsx`, extend `emptyState`, `sectionTitles`, imports, and `renderContent` with `/plugins`. Initially render a placeholder:

```tsx
if (path === "/plugins") {
  return <div>插件市场</div>;
}
```

- [ ] **Step 5: Run test**

Run:

```powershell
pnpm vitest run src/app/App.test.tsx
```

Expected: PASS.

- [ ] **Step 6: Commit**

```powershell
git add src/lib/tauri.ts src/app/App.tsx src/components/layout/app-sidebar.tsx src/app/App.test.tsx
git commit -m "feat: wire plugin market route"
```

---

### Task 8: Build Plugin Market Page

**Files:**
- Create: `src/features/plugins/plugin-market-page.tsx`
- Create: `src/features/plugins/plugin-market-page.test.tsx`
- Modify: `src/app/App.tsx`

- [ ] **Step 1: Write UI behavior tests**

Create `src/features/plugins/plugin-market-page.test.tsx`:

```tsx
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { vi, test, expect } from "vitest";
import { PluginMarketPage } from "./plugin-market-page";

const devices = [
  {
    deviceId: "AA:BB:CC:DD:EE:FF",
    alias: "Desk",
    board: "board",
    cachedAt: "2026-04-25T00:00:00Z",
    apiKeyId: 1,
  },
];

test("creates a custom plugin", async () => {
  const save = vi.fn().mockResolvedValue({
    id: 1,
    name: "天气",
    description: "天气插件",
    code: "return { type: 'text', text: 'sunny' };",
    createdAt: "2026-04-25T00:00:00Z",
    updatedAt: "2026-04-25T00:00:00Z",
  });

  render(
    <PluginMarketPage
      devices={devices}
      customPlugins={[]}
      pluginLoopTasks={[]}
      onSavePlugin={save}
      onDeletePlugin={vi.fn()}
      onRunPlugin={vi.fn()}
      onPushPlugin={vi.fn()}
      onCreateLoopTask={vi.fn()}
      onUpdateLoopTask={vi.fn()}
      onDeleteLoopTask={vi.fn()}
      onStartLoopTask={vi.fn()}
      onStopLoopTask={vi.fn()}
    />
  );

  await userEvent.click(screen.getByRole("button", { name: "新建插件" }));
  await userEvent.type(screen.getByLabelText("插件名称"), "天气");
  await userEvent.type(screen.getByLabelText("插件描述"), "天气插件");
  await userEvent.type(screen.getByLabelText("插件代码"), "return {{ type: 'text', text: 'sunny' }};");
  await userEvent.click(screen.getByRole("button", { name: "保存插件" }));

  expect(save).toHaveBeenCalled();
});

test("pushes a custom plugin to selected page", async () => {
  const push = vi.fn().mockResolvedValue(undefined);

  render(
    <PluginMarketPage
      devices={devices}
      customPlugins={[{
        id: 7,
        name: "天气",
        description: "天气插件",
        code: "return { type: 'text', text: 'sunny' };",
        createdAt: "2026-04-25T00:00:00Z",
        updatedAt: "2026-04-25T00:00:00Z",
      }]}
      pluginLoopTasks={[]}
      onSavePlugin={vi.fn()}
      onDeletePlugin={vi.fn()}
      onRunPlugin={vi.fn()}
      onPushPlugin={push}
      onCreateLoopTask={vi.fn()}
      onUpdateLoopTask={vi.fn()}
      onDeleteLoopTask={vi.fn()}
      onStartLoopTask={vi.fn()}
      onStopLoopTask={vi.fn()}
    />
  );

  await userEvent.click(screen.getByRole("button", { name: "推送一次" }));

  expect(push).toHaveBeenCalledWith("custom", "7", "AA:BB:CC:DD:EE:FF", 1);
});
```

- [ ] **Step 2: Run tests to verify they fail**

Run:

```powershell
pnpm vitest run src/features/plugins/plugin-market-page.test.tsx
```

Expected: FAIL because `PluginMarketPage` does not exist.

- [ ] **Step 3: Implement page**

Create `src/features/plugins/plugin-market-page.tsx` with:

```tsx
import { useState } from "react";
import { toast } from "../../components/ui/toast";
import type {
  CustomPluginInput,
  CustomPluginRecord,
  DeviceRecord,
  PluginLoopTask,
  PluginLoopTaskInput,
  PluginRunResult,
} from "../../lib/tauri";

type Props = {
  devices: DeviceRecord[];
  customPlugins: CustomPluginRecord[];
  pluginLoopTasks: PluginLoopTask[];
  onSavePlugin: (input: CustomPluginInput) => Promise<CustomPluginRecord>;
  onDeletePlugin: (pluginId: number) => Promise<void>;
  onRunPlugin: (pluginKind: "builtin" | "custom", pluginId: string) => Promise<PluginRunResult>;
  onPushPlugin: (pluginKind: "builtin" | "custom", pluginId: string, deviceId: string, pageId: number) => Promise<void>;
  onCreateLoopTask: (input: PluginLoopTaskInput) => Promise<PluginLoopTask>;
  onUpdateLoopTask: (taskId: number, input: PluginLoopTaskInput) => Promise<PluginLoopTask>;
  onDeleteLoopTask: (taskId: number) => Promise<void>;
  onStartLoopTask: (taskId: number) => Promise<PluginLoopTask>;
  onStopLoopTask: (taskId: number) => Promise<PluginLoopTask>;
};

export function PluginMarketPage({
  devices,
  customPlugins,
  pluginLoopTasks,
  onSavePlugin,
  onDeletePlugin,
  onRunPlugin,
  onPushPlugin,
}: Props) {
  const [editing, setEditing] = useState<CustomPluginRecord | null>(null);
  const [draft, setDraft] = useState<CustomPluginInput>({
    name: "",
    description: "",
    code: "return { type: \"text\", text: \"hello\" };",
  });
  const [runResult, setRunResult] = useState<PluginRunResult | null>(null);

  const firstDevice = devices[0];

  async function handleSave() {
    const saved = await onSavePlugin(draft);
    toast.success("插件已保存");
    setEditing(saved);
  }

  async function handleRun(plugin: CustomPluginRecord) {
    const result = await onRunPlugin("custom", String(plugin.id));
    setRunResult(result);
  }

  async function handlePush(plugin: CustomPluginRecord) {
    if (!firstDevice) {
      toast.error("请先在设置中添加设备");
      return;
    }
    await onPushPlugin("custom", String(plugin.id), firstDevice.deviceId, 1);
    toast.success("插件已推送");
  }

  return (
    <section className="space-y-6">
      <header>
        <h2 className="text-lg font-semibold">插件市场</h2>
        <p className="text-sm text-gray-500">创建 JS 插件，返回符合规范的文本或图片输出并推送到设备。</p>
      </header>

      <section className="space-y-3">
        <h3 className="font-medium">内置插件</h3>
        <div className="text-sm text-gray-500">暂无内置插件</div>
      </section>

      <section className="space-y-3">
        <div className="flex items-center justify-between">
          <h3 className="font-medium">自定义插件</h3>
          <button
            type="button"
            className="rounded border px-3 py-2 text-sm"
            onClick={() => {
              setEditing(null);
              setDraft({ name: "", description: "", code: "return { type: \"text\", text: \"hello\" };" });
            }}
          >
            新建插件
          </button>
        </div>

        <div className="grid gap-3">
          {customPlugins.map((plugin) => (
            <article key={plugin.id} className="rounded border bg-white p-4">
              <div className="font-medium">{plugin.name}</div>
              <div className="text-sm text-gray-500">{plugin.description}</div>
              <div className="mt-3 flex gap-2">
                <button type="button" className="rounded border px-3 py-2 text-sm" onClick={() => handleRun(plugin)}>测试运行</button>
                <button type="button" className="rounded border px-3 py-2 text-sm" onClick={() => handlePush(plugin)}>推送一次</button>
                <button type="button" className="rounded border px-3 py-2 text-sm" onClick={() => onDeletePlugin(plugin.id)}>删除</button>
              </div>
            </article>
          ))}
        </div>
      </section>

      <section className="space-y-3 rounded border bg-white p-4">
        <label className="block text-sm font-medium" htmlFor="plugin-name">插件名称</label>
        <input id="plugin-name" aria-label="插件名称" className="w-full rounded border px-3 py-2" value={draft.name} onChange={(e) => setDraft((prev) => ({ ...prev, name: e.target.value }))} />
        <label className="block text-sm font-medium" htmlFor="plugin-description">插件描述</label>
        <input id="plugin-description" aria-label="插件描述" className="w-full rounded border px-3 py-2" value={draft.description} onChange={(e) => setDraft((prev) => ({ ...prev, description: e.target.value }))} />
        <label className="block text-sm font-medium" htmlFor="plugin-code">插件代码</label>
        <textarea id="plugin-code" aria-label="插件代码" className="min-h-40 w-full rounded border px-3 py-2 font-mono text-sm" value={draft.code} onChange={(e) => setDraft((prev) => ({ ...prev, code: e.target.value }))} />
        <button type="button" className="rounded border px-3 py-2 text-sm" onClick={handleSave}>保存插件</button>
      </section>

      {runResult && (
        <section className="rounded border bg-white p-4">
          <h3 className="font-medium">运行结果</h3>
          {runResult.text && <pre className="mt-2 whitespace-pre-wrap text-sm">{runResult.text}</pre>}
          {runResult.previewPngBase64 && <img alt="文本图片预览" src={`data:image/png;base64,${runResult.previewPngBase64}`} />}
          {runResult.imageDataUrl && <img alt="图片预览" src={runResult.imageDataUrl} />}
        </section>
      )}

      <section className="space-y-3">
        <h3 className="font-medium">插件循环任务</h3>
        {pluginLoopTasks.length === 0 ? <div className="text-sm text-gray-500">暂无循环任务</div> : null}
      </section>
    </section>
  );
}
```

- [ ] **Step 4: Wire page into `App.tsx`**

Import `PluginMarketPage` and pass real callbacks. After save/delete/create loop actions, call `reload()` so bootstrap state refreshes.

- [ ] **Step 5: Run tests**

Run:

```powershell
pnpm vitest run src/features/plugins/plugin-market-page.test.tsx src/app/App.test.tsx
```

Expected: PASS.

- [ ] **Step 6: Commit**

```powershell
git add src/features/plugins src/app/App.tsx
git commit -m "feat: add plugin market page"
```

---

### Task 9: Add Loop Task UI

**Files:**
- Modify: `src/features/plugins/plugin-market-page.tsx`
- Modify: `src/features/plugins/plugin-market-page.test.tsx`

- [ ] **Step 1: Add loop creation test**

In `src/features/plugins/plugin-market-page.test.tsx`, add:

```tsx
test("creates a loop task for a custom plugin", async () => {
  const createLoopTask = vi.fn().mockResolvedValue({
    id: 1,
    pluginKind: "custom",
    pluginId: "7",
    name: "天气循环",
    deviceId: "AA:BB:CC:DD:EE:FF",
    pageId: 1,
    intervalSeconds: 60,
    durationType: "none",
    status: "idle",
    createdAt: "2026-04-25T00:00:00Z",
    updatedAt: "2026-04-25T00:00:00Z",
  });

  render(
    <PluginMarketPage
      devices={devices}
      customPlugins={[{
        id: 7,
        name: "天气",
        description: "天气插件",
        code: "return { type: 'text', text: 'sunny' };",
        createdAt: "2026-04-25T00:00:00Z",
        updatedAt: "2026-04-25T00:00:00Z",
      }]}
      pluginLoopTasks={[]}
      onSavePlugin={vi.fn()}
      onDeletePlugin={vi.fn()}
      onRunPlugin={vi.fn()}
      onPushPlugin={vi.fn()}
      onCreateLoopTask={createLoopTask}
      onUpdateLoopTask={vi.fn()}
      onDeleteLoopTask={vi.fn()}
      onStartLoopTask={vi.fn()}
      onStopLoopTask={vi.fn()}
    />
  );

  await userEvent.click(screen.getByRole("button", { name: "创建循环任务" }));

  expect(createLoopTask).toHaveBeenCalledWith(expect.objectContaining({
    pluginKind: "custom",
    pluginId: "7",
    deviceId: "AA:BB:CC:DD:EE:FF",
    pageId: 1,
    intervalSeconds: 60,
    durationType: "none",
  }));
});
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```powershell
pnpm vitest run src/features/plugins/plugin-market-page.test.tsx
```

Expected: FAIL because the button and handler do not exist.

- [ ] **Step 3: Add loop controls**

In each custom plugin card, add:

```tsx
<button type="button" className="rounded border px-3 py-2 text-sm" onClick={() => handleCreateLoop(plugin)}>
  创建循环任务
</button>
```

Add handler:

```tsx
async function handleCreateLoop(plugin: CustomPluginRecord) {
  if (!firstDevice) {
    toast.error("请先在设置中添加设备");
    return;
  }
  await onCreateLoopTask({
    pluginKind: "custom",
    pluginId: String(plugin.id),
    name: `${plugin.name} 循环`,
    deviceId: firstDevice.deviceId,
    pageId: 1,
    intervalSeconds: 60,
    durationType: "none",
  });
  toast.success("循环任务已创建");
}
```

Render existing loop tasks:

```tsx
{pluginLoopTasks.map((task) => (
  <article key={task.id} className="rounded border bg-white p-4">
    <div className="font-medium">{task.name}</div>
    <div className="text-sm text-gray-500">第 {task.pageId} 页 · 每 {task.intervalSeconds} 秒 · {task.status}</div>
    {task.errorMessage && <div className="text-sm text-red-600">{task.errorMessage}</div>}
    <div className="mt-3 flex gap-2">
      <button type="button" className="rounded border px-3 py-2 text-sm" onClick={() => onStartLoopTask(task.id)}>启动</button>
      <button type="button" className="rounded border px-3 py-2 text-sm" onClick={() => onStopLoopTask(task.id)}>停止</button>
      <button type="button" className="rounded border px-3 py-2 text-sm" onClick={() => onDeleteLoopTask(task.id)}>删除</button>
    </div>
  </article>
))}
```

- [ ] **Step 4: Run tests**

Run:

```powershell
pnpm vitest run src/features/plugins/plugin-market-page.test.tsx
```

Expected: PASS.

- [ ] **Step 5: Commit**

```powershell
git add src/features/plugins/plugin-market-page.tsx src/features/plugins/plugin-market-page.test.tsx
git commit -m "feat: manage plugin loop tasks"
```

---

### Task 10: Final Verification

**Files:**
- Review all changed files.

- [ ] **Step 1: Run frontend tests**

Run:

```powershell
pnpm vitest run
```

Expected: PASS.

- [ ] **Step 2: Run frontend build**

Run:

```powershell
pnpm build
```

Expected: PASS.

- [ ] **Step 3: Run Rust tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected: PASS.

- [ ] **Step 4: Manual smoke test**

Run:

```powershell
pnpm tauri dev
```

Manual checks:

- Sidebar shows “插件市场”.
- Create a custom plugin with code `return { type: "text", text: "hello" };`.
- Test run shows `hello`.
- Push once sends the plugin output to page 1 of the first device.
- Create a loop task for the plugin.
- Start and stop the loop task.
- Create a `textImage` plugin and confirm preview image renders.

- [ ] **Step 5: Commit verification fixes**

If verification required small fixes:

```powershell
git add <fixed-files>
git commit -m "fix: polish plugin market flow"
```

If no fixes were needed, do not create an empty commit.
