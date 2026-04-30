# macOS 日历同步 Implementation Plan

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** 在设置页面新增"日历同步"功能板块，通过 Swift EventKit CLI 桥接将待办与 macOS 日历事件 / 提醒事项进行双向同步。

**Architecture:** Swift CLI 工具 `calendar-bridge` 通过 stdout/stderr 以 JSON 交互，Rust 层封装 `std::process::Command` 调用并处理同步逻辑，前端 React 组件提供配置 UI 和手动触发入口。

**Tech Stack:** Swift + EventKit，Rust（Tauri 2 命令），React + Tailwind CSS 4，shadcn/ui Select，Vitest + Testing Library

---

## 背景：代码库关键约定

- 数据目录：`~/.zectrix-note/`，通过 `state.data_dir` 访问
- JSON 持久化：`src-tauri/src/storage/mod.rs` 的 `load_json<T>` / `save_json`，`T: Default` 时文件不存在返回默认值
- Tauri 命令结构：`src-tauri/src/commands/<module>.rs` → 在 `commands/mod.rs` pub mod 声明 → 在 `lib.rs` `invoke_handler!` 注册
- 前端 Tauri 调用全部通过 `src/lib/tauri.ts` 的 `invoke` 封装函数
- **禁用原生 `<select>`**：目标日历下拉必须用 `src/components/ui/select.tsx` 的 shadcn/ui Select
- 测试文件和源文件并列，命名 `*.test.tsx`；Tauri 模块 mock `vi.mock("../../lib/tauri", ...)`

---

## Task 1: 在 TodoRecord 新增日历同步字段

**Objective:** 为 TodoRecord 加上 `calendar_external_id` 和 `calendar_synced_at`，并保持向后兼容。

**Files:**
- Modify: `src-tauri/src/models.rs:53-80`（TodoRecord struct）

**Step 1: 在 TodoRecord 末尾添加两个新字段**

打开 `src-tauri/src/models.rs`，在 `last_synced_status` 字段**之后**，结构体闭合 `}` **之前**加入：

```rust
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
```

**Step 2: 确认编译通过**

```bash
cargo build --manifest-path src-tauri/Cargo.toml 2>&1 | tail -5
```
Expected: `Compiling zectrix-note-4 ...` 并以 `Finished` 结束，无 error。

**Step 3: Commit**

```bash
git add src-tauri/src/models.rs
git commit -m "feat: add calendar_external_id and calendar_synced_at to TodoRecord"
```

---

## Task 2: 新增日历同步数据模型

**Objective:** 在 `models.rs` 中定义 `CalendarSyncConfig`、`CalendarInfo`、`SyncResult` 三个新结构体。

**Files:**
- Modify: `src-tauri/src/models.rs`（在文件末尾追加）

**Step 1: 追加以下代码到 `src-tauri/src/models.rs` 末尾**

```rust
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
```

**Step 2: 确认编译通过**

```bash
cargo build --manifest-path src-tauri/Cargo.toml 2>&1 | tail -5
```
Expected: `Finished`，无 error。

**Step 3: Commit**

```bash
git add src-tauri/src/models.rs
git commit -m "feat: add CalendarSyncConfig, CalendarInfo, SyncResult models"
```

---

## Task 3: 编写 Swift CalendarBridge CLI

**Objective:** 创建 Swift CLI 工具，通过 EventKit 操作 macOS 日历 / 提醒事项，以 JSON 格式输出结果。

**Files:**
- Create: `src-tauri/src/swift/CalendarBridge.swift`

**Step 1: 创建目录并写入 Swift 源码**

```bash
mkdir -p src-tauri/src/swift
```

新建文件 `src-tauri/src/swift/CalendarBridge.swift`：

```swift
import Foundation
import EventKit

// MARK: - Output helpers

func outputJSON<T: Encodable>(_ value: T) -> Never {
    let encoder = JSONEncoder()
    encoder.dateEncodingStrategy = .iso8601
    let data = try! encoder.encode(value)
    print(String(data: data, encoding: .utf8)!)
    exit(0)
}

func outputError(_ message: String) -> Never {
    fputs("ERROR: \(message)\n", stderr)
    exit(1)
}

// MARK: - Data types

struct CalendarItem: Codable {
    var externalId: String
    var title: String
    var dueDate: String?
    var isCompleted: Bool
    var lastModified: String
}

struct CalendarInfoOutput: Codable {
    var id: String
    var title: String
    var color: String?
}

struct SuccessResponse: Codable {
    var success: Bool
}

// MARK: - EventKit helpers

let store = EKEventStore()

func requestPermission(type: EKEntityType) {
    let semaphore = DispatchSemaphore(value: 0)
    var granted = false
    if #available(macOS 14.0, *) {
        store.requestFullAccessToEvents { ok, _ in granted = ok; semaphore.signal() }
    } else {
        store.requestAccess(to: type) { ok, _ in granted = ok; semaphore.signal() }
    }
    semaphore.wait()
    if !granted { outputError("permission denied") }
}

func isoString(_ date: Date?) -> String? {
    guard let d = date else { return nil }
    return ISO8601DateFormatter().string(from: d)
}

// MARK: - Commands

func cmdRequestPermission() {
    let sem = DispatchSemaphore(value: 0)
    var calGranted = false
    var remGranted = false

    if #available(macOS 14.0, *) {
        store.requestFullAccessToEvents { ok, _ in calGranted = ok; sem.signal() }
        sem.wait()
        store.requestFullAccessToReminders { ok, _ in remGranted = ok; sem.signal() }
        sem.wait()
    } else {
        store.requestAccess(to: .event) { ok, _ in calGranted = ok; sem.signal() }
        sem.wait()
        store.requestAccess(to: .reminder) { ok, _ in remGranted = ok; sem.signal() }
        sem.wait()
    }

    outputJSON(SuccessResponse(success: calGranted || remGranted))
}

func cmdListCalendars(type: String) {
    let ekType: EKEntityType = type == "reminder" ? .reminder : .event
    requestPermission(type: ekType)
    let cals = store.calendars(for: ekType)
    let result = cals.map { cal in
        let hex = cal.cgColor.map { color -> String in
            guard let components = color.components, components.count >= 3 else { return "#888888" }
            return String(format: "#%02X%02X%02X",
                Int(components[0] * 255),
                Int(components[1] * 255),
                Int(components[2] * 255))
        }
        return CalendarInfoOutput(id: cal.calendarIdentifier, title: cal.title, color: hex)
    }
    outputJSON(result)
}

func cmdListItems(calendarId: String) {
    // Try event first, then reminder
    if let cal = store.calendar(withIdentifier: calendarId), cal.allowedEntityTypes.contains(.event) {
        requestPermission(type: .event)
        let start = Date(timeIntervalSinceNow: -60 * 60 * 24 * 365)
        let end = Date(timeIntervalSinceNow: 60 * 60 * 24 * 365)
        let pred = store.predicateForEvents(withStart: start, end: end, calendars: [cal])
        let events = store.events(matching: pred)
        let items = events.map { ev -> CalendarItem in
            CalendarItem(
                externalId: ev.eventIdentifier,
                title: ev.title ?? "",
                dueDate: isoString(ev.startDate),
                isCompleted: ev.status == .done,
                lastModified: isoString(ev.lastModifiedDate ?? ev.startDate) ?? ""
            )
        }
        outputJSON(items)
    } else {
        requestPermission(type: .reminder)
        guard let cal = store.calendar(withIdentifier: calendarId) else {
            outputError("calendar not found: \(calendarId)")
        }
        let sem = DispatchSemaphore(value: 0)
        var reminders: [EKReminder] = []
        let pred = store.predicateForReminders(in: [cal])
        store.fetchReminders(matching: pred) { result in
            reminders = result ?? []
            sem.signal()
        }
        sem.wait()
        let items = reminders.map { rem -> CalendarItem in
            let due = rem.dueDateComponents?.date
            return CalendarItem(
                externalId: rem.calendarItemIdentifier,
                title: rem.title ?? "",
                dueDate: isoString(due),
                isCompleted: rem.isCompleted,
                lastModified: isoString(rem.lastModifiedDate ?? Date()) ?? ""
            )
        }
        outputJSON(items)
    }
}

struct CreateItemInput: Decodable {
    var calendarId: String
    var title: String
    var dueDate: String?
    var isCompleted: Bool
    var targetType: String // "calendar" | "reminder"
}

func cmdCreateItem(data: String) {
    let decoder = JSONDecoder()
    guard let jsonData = data.data(using: .utf8),
          let input = try? decoder.decode(CreateItemInput.self, from: jsonData)
    else { outputError("invalid JSON input") }

    guard let cal = store.calendar(withIdentifier: input.calendarId) else {
        outputError("calendar not found: \(input.calendarId)")
    }

    var externalId: String
    if input.targetType == "calendar" {
        requestPermission(type: .event)
        let ev = EKEvent(eventStore: store)
        ev.title = input.title
        ev.calendar = cal
        if let ds = input.dueDate, let date = ISO8601DateFormatter().date(from: ds) {
            ev.startDate = date
            ev.endDate = date.addingTimeInterval(3600)
        } else {
            let now = Date()
            ev.startDate = now
            ev.endDate = now.addingTimeInterval(3600)
        }
        try! store.save(ev, span: .thisEvent)
        externalId = ev.eventIdentifier
    } else {
        requestPermission(type: .reminder)
        let rem = EKReminder(eventStore: store)
        rem.title = input.title
        rem.calendar = cal
        if let ds = input.dueDate, let date = ISO8601DateFormatter().date(from: ds) {
            var comps = Calendar.current.dateComponents([.year, .month, .day, .hour, .minute], from: date)
            rem.dueDateComponents = comps
        }
        rem.isCompleted = input.isCompleted
        try! store.save(rem, commit: true)
        externalId = rem.calendarItemIdentifier
    }
    outputJSON(["externalId": externalId])
}

struct UpdateItemInput: Decodable {
    var title: String
    var dueDate: String?
    var isCompleted: Bool
}

func cmdUpdateItem(externalId: String, data: String) {
    let decoder = JSONDecoder()
    guard let jsonData = data.data(using: .utf8),
          let input = try? decoder.decode(UpdateItemInput.self, from: jsonData)
    else { outputError("invalid JSON input") }

    if let ev = store.calendarItem(withIdentifier: externalId) as? EKEvent {
        requestPermission(type: .event)
        ev.title = input.title
        if let ds = input.dueDate, let date = ISO8601DateFormatter().date(from: ds) {
            ev.startDate = date
            ev.endDate = date.addingTimeInterval(3600)
        }
        try! store.save(ev, span: .thisEvent)
    } else if let rem = store.calendarItem(withIdentifier: externalId) as? EKReminder {
        requestPermission(type: .reminder)
        rem.title = input.title
        rem.isCompleted = input.isCompleted
        if let ds = input.dueDate, let date = ISO8601DateFormatter().date(from: ds) {
            rem.dueDateComponents = Calendar.current.dateComponents([.year, .month, .day, .hour, .minute], from: date)
        }
        try! store.save(rem, commit: true)
    } else {
        outputError("item not found: \(externalId)")
    }
    outputJSON(SuccessResponse(success: true))
}

func cmdDeleteItem(externalId: String) {
    if let ev = store.calendarItem(withIdentifier: externalId) as? EKEvent {
        requestPermission(type: .event)
        try! store.remove(ev, span: .thisEvent)
    } else if let rem = store.calendarItem(withIdentifier: externalId) as? EKReminder {
        requestPermission(type: .reminder)
        try! store.remove(rem, commit: true)
    } else {
        outputError("item not found: \(externalId)")
    }
    outputJSON(SuccessResponse(success: true))
}

// MARK: - CLI dispatch

let args = CommandLine.arguments
guard args.count >= 2 else {
    fputs("Usage: calendar-bridge <command> [options]\n", stderr)
    exit(1)
}

switch args[1] {
case "request-permission":
    cmdRequestPermission()
case "list-calendars":
    let typeArg = args.first(where: { $0.hasPrefix("--type=") })?.dropFirst(7) ?? "reminder"
    cmdListCalendars(type: String(typeArg))
case "list-items":
    guard let idArg = args.first(where: { $0.hasPrefix("--calendar-id=") }) else {
        outputError("missing --calendar-id")
    }
    cmdListItems(calendarId: String(idArg.dropFirst("--calendar-id=".count)))
case "create-item":
    guard let dataArg = args.first(where: { $0.hasPrefix("--data=") }) else {
        outputError("missing --data")
    }
    cmdCreateItem(data: String(dataArg.dropFirst("--data=".count)))
case "update-item":
    guard let idArg = args.first(where: { $0.hasPrefix("--external-id=") }),
          let dataArg = args.first(where: { $0.hasPrefix("--data=") })
    else { outputError("missing --external-id or --data") }
    cmdUpdateItem(
        externalId: String(idArg.dropFirst("--external-id=".count)),
        data: String(dataArg.dropFirst("--data=".count))
    )
case "delete-item":
    guard let idArg = args.first(where: { $0.hasPrefix("--external-id=") }) else {
        outputError("missing --external-id")
    }
    cmdDeleteItem(externalId: String(idArg.dropFirst("--external-id=".count)))
default:
    outputError("unknown command: \(args[1])")
}
```

**Step 2: Commit Swift 源码（不编译）**

```bash
git add src-tauri/src/swift/CalendarBridge.swift
git commit -m "feat: add CalendarBridge Swift CLI source"
```

---

## Task 4: 配置 build.rs 编译 Swift CLI

**Objective:** 在 Tauri 构建阶段自动将 Swift 源码编译为 `resources/calendar-bridge` 二进制。

**Files:**
- Modify: `src-tauri/build.rs`

**Step 1: 读取现有 build.rs**

```bash
cat src-tauri/build.rs
```

当前内容仅有：
```rust
fn main() {
    tauri_build::build()
}
```

**Step 2: 替换 build.rs 为以下内容**

```rust
fn main() {
    #[cfg(target_os = "macos")]
    {
        compile_calendar_bridge();
    }
    tauri_build::build()
}

#[cfg(target_os = "macos")]
fn compile_calendar_bridge() {
    use std::process::Command;

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let swift_src = format!("{}/src/swift/CalendarBridge.swift", manifest_dir);
    let out_dir = format!("{}/resources", manifest_dir);
    let out_bin = format!("{}/calendar-bridge", out_dir);

    std::fs::create_dir_all(&out_dir).expect("failed to create resources dir");

    let status = Command::new("swiftc")
        .args([
            &swift_src,
            "-o",
            &out_bin,
            "-framework",
            "EventKit",
            "-framework",
            "Foundation",
        ])
        .status()
        .expect("swiftc not found — install Xcode Command Line Tools");

    if !status.success() {
        panic!("CalendarBridge.swift compilation failed");
    }

    println!("cargo:rerun-if-changed={}", swift_src);
}
```

**Step 3: 更新 tauri.conf.json 添加 resources 条目**

找到 `tauri.conf.json` 中 `"bundle"` 对象，在 `"icon"` 数组同级添加 `"resources"`：

```json
"bundle": {
  "active": true,
  "targets": "all",
  "icon": [
    "icons/32x32.png",
    "icons/128x128.png",
    "icons/128x128@2x.png",
    "icons/icon.icns",
    "icons/icon.ico"
  ],
  "resources": ["resources/calendar-bridge"]
}
```

**Step 4: 在 Tauri 构建能力中允许运行子进程**

检查 `src-tauri/capabilities/` 目录：

```bash
ls src-tauri/capabilities/
```

若存在 `default.json`，在其 `"permissions"` 数组中确保有 `"core:default"`（通常已包含 subprocess 权限，Tauri 2 core 默认允许）。若无任何 capabilities 文件，跳过此步骤（Tauri 2 默认允许）。

**Step 5: 验证编译（仅 macOS）**

```bash
cd src-tauri && cargo build 2>&1 | tail -10
```
Expected: 在 `resources/` 目录下出现 `calendar-bridge` 可执行文件。

```bash
ls -la src-tauri/resources/calendar-bridge
./src-tauri/resources/calendar-bridge 2>&1 | head -3
```
Expected: 打印 `Usage: calendar-bridge <command> [options]` 到 stderr，退出码 1。

**Step 6: Commit**

```bash
git add src-tauri/build.rs src-tauri/tauri.conf.json
git commit -m "feat: compile CalendarBridge Swift CLI via build.rs, bundle as resource"
```

---

## Task 5: 创建 Rust calendar_sync 命令模块

**Objective:** 新建 `src-tauri/src/commands/calendar_sync.rs`，实现四个 Tauri 命令。

**Files:**
- Create: `src-tauri/src/commands/calendar_sync.rs`

**Step 1: 创建文件**

新建 `src-tauri/src/commands/calendar_sync.rs`：

```rust
use crate::models::{CalendarInfo, CalendarSyncConfig, CalendarTargetType, SyncDirection, SyncResult, TodoRecord};
use crate::storage::{load_json, save_json};
use serde::Deserialize;
use std::process::Command;

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
    let type_arg = if target_type == "CalendarEvent" { "calendar" } else { "reminder" };
    let arg = format!("--type={}", type_arg);
    let output = run_bridge(&bridge, &["list-calendars", &arg])?;

    #[derive(Deserialize)]
    struct BridgeCalendar {
        id: String,
        title: String,
        color: Option<String>,
    }
    let items: Vec<BridgeCalendar> = serde_json::from_str(&output)
        .map_err(|e| format!("parse error: {e}"))?;
    Ok(items
        .into_iter()
        .map(|c| CalendarInfo { id: c.id, title: c.title, color: c.color })
        .collect())
}

#[tauri::command]
pub async fn sync_calendar(
    app: tauri::AppHandle,
    state: tauri::State<'_, crate::state::AppState>,
) -> Result<SyncResult, String> {
    let config: CalendarSyncConfig = load_json(&config_path(&state.data_dir))
        .map_err(|e| e.to_string())?;

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
        due_date: Option<String>,
        is_completed: bool,
        last_modified: String,
    }

    let remote_items: Vec<BridgeItem> = serde_json::from_str(&items_json)
        .map_err(|e| format!("parse remote items: {e}"))?;

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
    let should_push = matches!(config.direction, SyncDirection::ToCalendar | SyncDirection::Bidirectional);
    // Handle FromCalendar / Bidirectional: pull remote → local
    let should_pull = matches!(config.direction, SyncDirection::FromCalendar | SyncDirection::Bidirectional);

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
                let remote_newer = remote_map.get(ext_id).map_or(false, |r| {
                    r.last_modified.as_str() > todo.updated_at.as_str()
                });
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
                        struct Created { #[serde(rename = "externalId")] external_id: String }
                        if let Ok(c) = serde_json::from_str::<Created>(&json) {
                            todo.calendar_external_id = Some(c.external_id);
                            todo.calendar_synced_at = Some(chrono::Utc::now().to_rfc3339());
                            result.created += 1;
                        }
                    }
                    Err(_) => { result.skipped += 1; }
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
                    let remote_newer = remote.last_modified.as_str() > todo.updated_at.as_str();
                    if remote_newer || matches!(config.direction, SyncDirection::FromCalendar) {
                        todo.title = remote.title.clone();
                        todo.status = if remote.is_completed { 1 } else { 0 };
                        if let Some(ref ds) = remote.due_date {
                            // Extract date and time from ISO8601
                            let parts: Vec<&str> = ds.splitn(2, 'T').collect();
                            todo.due_date = Some(parts[0].to_string());
                            if parts.len() > 1 {
                                let time = parts[1].trim_end_matches('Z').trim_end_matches("+00:00");
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
                let new_local_id = format!("todo-cal-{}", remote.external_id.replace(['/', ':'], "-"));
                let parts: Vec<&str> = remote.due_date.as_deref().unwrap_or("").splitn(2, 'T').collect();
                let due_date = if parts[0].is_empty() { None } else { Some(parts[0].to_string()) };
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

    save_json(&state.data_dir.join("todos.json"), &todos)
        .map_err(|e| e.to_string())?;

    Ok(result)
}
```

**Step 2: 确认编译**

```bash
cargo build --manifest-path src-tauri/Cargo.toml 2>&1 | grep -E "error|warning: unused" | head -20
```
Expected: 无 error。可能有 unused import 警告，正常。

**Step 3: Commit**

```bash
git add src-tauri/src/commands/calendar_sync.rs
git commit -m "feat: add calendar_sync Rust command module"
```

---

## Task 6: 注册 calendar_sync 命令

**Objective:** 在 `commands/mod.rs` 声明新模块，并在 `lib.rs` 的 `invoke_handler` 注册四个命令。

**Files:**
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`

**Step 1: 在 `commands/mod.rs` 末尾新增一行**

在 `pub mod updater;` 之后追加：

```rust
pub mod calendar_sync;
```

**Step 2: 在 `lib.rs` 的 `invoke_handler` 末尾追加四条注册**

在 `commands::image_loop::select_folder_dialog,` 后面（逗号后）追加：

```rust
            commands::calendar_sync::get_calendar_sync_config,
            commands::calendar_sync::save_calendar_sync_config,
            commands::calendar_sync::list_calendars,
            commands::calendar_sync::sync_calendar,
```

**Step 3: 确认编译**

```bash
cargo build --manifest-path src-tauri/Cargo.toml 2>&1 | grep "^error" | head -10
```
Expected: 无输出（无 error）。

**Step 4: Commit**

```bash
git add src-tauri/src/commands/mod.rs src-tauri/src/lib.rs
git commit -m "feat: register calendar_sync Tauri commands"
```

---

## Task 7: 在 tauri.ts 添加 TypeScript 类型和调用函数

**Objective:** 在 `src/lib/tauri.ts` 末尾添加日历同步相关的类型定义和 `invoke` 封装函数。

**Files:**
- Modify: `src/lib/tauri.ts`（在文件末尾追加）

**Step 1: 追加以下代码到 `src/lib/tauri.ts` 末尾**

```typescript
// ─── Calendar Sync ───────────────────────────────────────────────────────────

export type SyncDirection = "ToCalendar" | "FromCalendar" | "Bidirectional";
export type CalendarTargetType = "CalendarEvent" | "Reminder";

export type CalendarSyncConfig = {
  enabled: boolean;
  direction: SyncDirection;
  targetType: CalendarTargetType;
  targetCalendarId: string | null;
};

export type CalendarInfo = {
  id: string;
  title: string;
  color: string | null;
};

export type SyncResult = {
  created: number;
  updated: number;
  skipped: number;
  deleted: number;
};

export async function getCalendarSyncConfig(): Promise<CalendarSyncConfig> {
  return invoke<CalendarSyncConfig>("get_calendar_sync_config");
}

export async function saveCalendarSyncConfig(config: CalendarSyncConfig): Promise<void> {
  return invoke("save_calendar_sync_config", { config });
}

export async function listCalendars(targetType: CalendarTargetType): Promise<CalendarInfo[]> {
  return invoke<CalendarInfo[]>("list_calendars", { targetType });
}

export async function syncCalendar(): Promise<SyncResult> {
  return invoke<SyncResult>("sync_calendar");
}
```

**Step 2: 确认 TypeScript 编译通过**

```bash
pnpm build 2>&1 | tail -10
```
Expected: `✓ built in ...` 或 `vite build` 成功，无 TS error。

**Step 3: Commit**

```bash
git add src/lib/tauri.ts
git commit -m "feat: add calendar sync types and invoke functions to tauri.ts"
```

---

## Task 8: 编写 CalendarSyncPanel 组件（测试先行）

**Objective:** 用 TDD 方式创建自包含的 `CalendarSyncPanel` 组件，含启用开关、目标类型、日历本下拉、同步方向、立即同步按钮。

**Files:**
- Create: `src/features/settings/calendar-sync-panel.test.tsx`
- Create: `src/features/settings/calendar-sync-panel.tsx`

**Step 1: 先写测试文件**

新建 `src/features/settings/calendar-sync-panel.test.tsx`：

```tsx
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { CalendarSyncPanel } from "./calendar-sync-panel";

const {
  getCalendarSyncConfig,
  saveCalendarSyncConfig,
  listCalendars,
  syncCalendar,
} = vi.hoisted(() => ({
  getCalendarSyncConfig: vi.fn().mockResolvedValue({
    enabled: false,
    direction: "ToCalendar",
    targetType: "Reminder",
    targetCalendarId: null,
  }),
  saveCalendarSyncConfig: vi.fn().mockResolvedValue(undefined),
  listCalendars: vi.fn().mockResolvedValue([
    { id: "cal-1", title: "提醒事项", color: "#ff0000" },
  ]),
  syncCalendar: vi.fn().mockResolvedValue({ created: 2, updated: 1, skipped: 0, deleted: 0 }),
}));

vi.mock("../../lib/tauri", () => ({
  getCalendarSyncConfig,
  saveCalendarSyncConfig,
  listCalendars,
  syncCalendar,
}));

test("renders calendar sync section heading", async () => {
  render(<CalendarSyncPanel />);
  expect(await screen.findByText("日历同步")).toBeInTheDocument();
});

test("enable toggle calls saveCalendarSyncConfig", async () => {
  const user = userEvent.setup();
  render(<CalendarSyncPanel />);
  await screen.findByText("日历同步");
  const toggle = screen.getByRole("checkbox", { name: /启用日历同步/ });
  await user.click(toggle);
  expect(saveCalendarSyncConfig).toHaveBeenCalledWith(
    expect.objectContaining({ enabled: true })
  );
});

test("shows calendar options when enabled", async () => {
  getCalendarSyncConfig.mockResolvedValueOnce({
    enabled: true,
    direction: "ToCalendar",
    targetType: "Reminder",
    targetCalendarId: null,
  });
  render(<CalendarSyncPanel />);
  await screen.findByText("目标类型");
  expect(screen.getByText("同步方向")).toBeInTheDocument();
  expect(screen.getByRole("button", { name: "立即同步" })).toBeInTheDocument();
});

test("sync button shows result summary", async () => {
  getCalendarSyncConfig.mockResolvedValueOnce({
    enabled: true,
    direction: "ToCalendar",
    targetType: "Reminder",
    targetCalendarId: "cal-1",
  });
  const user = userEvent.setup();
  render(<CalendarSyncPanel />);
  const syncBtn = await screen.findByRole("button", { name: "立即同步" });
  await user.click(syncBtn);
  await waitFor(() => {
    expect(screen.getByText(/新增 2 条/)).toBeInTheDocument();
  });
});
```

**Step 2: 运行测试，确认失败（RED）**

```bash
pnpm vitest run src/features/settings/calendar-sync-panel.test.tsx 2>&1 | tail -15
```
Expected: 报 `Cannot find module` 或 `CalendarSyncPanel is not defined`，4 tests FAIL。

**Step 3: 创建实现文件**

新建 `src/features/settings/calendar-sync-panel.tsx`：

```tsx
import { useEffect, useState, useCallback } from "react";
import {
  type CalendarSyncConfig,
  type CalendarInfo,
  type SyncResult,
  getCalendarSyncConfig,
  saveCalendarSyncConfig,
  listCalendars,
  syncCalendar,
} from "../../lib/tauri";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "../../components/ui/select";

const DEFAULT_CONFIG: CalendarSyncConfig = {
  enabled: false,
  direction: "ToCalendar",
  targetType: "Reminder",
  targetCalendarId: null,
};

export function CalendarSyncPanel() {
  const [config, setConfig] = useState<CalendarSyncConfig>(DEFAULT_CONFIG);
  const [calendars, setCalendars] = useState<CalendarInfo[]>([]);
  const [syncing, setSyncing] = useState(false);
  const [syncResult, setSyncResult] = useState<SyncResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    getCalendarSyncConfig()
      .then(setConfig)
      .catch(() => {})
      .finally(() => setLoading(false));
  }, []);

  useEffect(() => {
    if (!config.enabled) return;
    listCalendars(config.targetType)
      .then(setCalendars)
      .catch(() => setCalendars([]));
  }, [config.enabled, config.targetType]);

  const updateConfig = useCallback(
    async (patch: Partial<CalendarSyncConfig>) => {
      const next = { ...config, ...patch };
      setConfig(next);
      setSyncResult(null);
      try {
        await saveCalendarSyncConfig(next);
      } catch (e) {
        setError(String(e));
      }
    },
    [config]
  );

  async function handleSync() {
    setSyncing(true);
    setError(null);
    setSyncResult(null);
    try {
      const result = await syncCalendar();
      setSyncResult(result);
    } catch (e) {
      setError(String(e));
    } finally {
      setSyncing(false);
    }
  }

  if (loading) return null;

  return (
    <div className="space-y-4">
      <div className="flex items-center gap-3">
        <label className="flex items-center gap-2 cursor-pointer select-none">
          <input
            type="checkbox"
            aria-label="启用日历同步"
            checked={config.enabled}
            onChange={(e) => updateConfig({ enabled: e.target.checked })}
            className="w-4 h-4 accent-slate-600"
          />
          <span className="text-sm font-medium text-gray-700">启用日历同步</span>
        </label>
      </div>

      {config.enabled && (
        <div className="space-y-4 pl-6 border-l-2 border-slate-200">
          {/* 目标类型 */}
          <div className="space-y-2">
            <label className="block text-sm font-medium text-gray-700">目标类型</label>
            <div className="flex gap-4">
              {(["Reminder", "CalendarEvent"] as const).map((t) => (
                <label key={t} className="flex items-center gap-1.5 cursor-pointer text-sm text-gray-700">
                  <input
                    type="radio"
                    name="targetType"
                    value={t}
                    checked={config.targetType === t}
                    onChange={() => updateConfig({ targetType: t, targetCalendarId: null })}
                    className="accent-slate-600"
                  />
                  {t === "Reminder" ? "提醒事项" : "日历事件"}
                </label>
              ))}
            </div>
          </div>

          {/* 目标日历本 */}
          <div className="space-y-2">
            <label className="block text-sm font-medium text-gray-700">目标日历本</label>
            <Select
              value={config.targetCalendarId ?? ""}
              onValueChange={(v) => updateConfig({ targetCalendarId: v || null })}
            >
              <SelectTrigger className="w-56">
                <SelectValue placeholder="请选择日历本" />
              </SelectTrigger>
              <SelectContent>
                {calendars.map((cal) => (
                  <SelectItem key={cal.id} value={cal.id}>
                    {cal.title}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {/* 同步方向 */}
          <div className="space-y-2">
            <label className="block text-sm font-medium text-gray-700">同步方向</label>
            <div className="flex flex-col gap-1.5">
              {(
                [
                  ["ToCalendar", "仅推送到日历"],
                  ["FromCalendar", "仅从日历导入"],
                  ["Bidirectional", "双向同步"],
                ] as const
              ).map(([val, label]) => (
                <label key={val} className="flex items-center gap-1.5 cursor-pointer text-sm text-gray-700">
                  <input
                    type="radio"
                    name="direction"
                    value={val}
                    checked={config.direction === val}
                    onChange={() => updateConfig({ direction: val })}
                    className="accent-slate-600"
                  />
                  {label}
                </label>
              ))}
            </div>
          </div>

          {/* 立即同步 */}
          <div className="space-y-2">
            <button
              type="button"
              onClick={handleSync}
              disabled={syncing || !config.targetCalendarId}
              className="px-4 py-2 bg-slate-600 text-white rounded-md hover:bg-slate-700 disabled:opacity-50 disabled:cursor-not-allowed transition text-sm focus:outline-none focus:ring-2 focus:ring-slate-500"
            >
              {syncing ? "同步中…" : "立即同步"}
            </button>

            {syncResult && (
              <p className="text-sm text-green-700">
                新增 {syncResult.created} 条，更新 {syncResult.updated} 条，删除 {syncResult.deleted} 条，跳过 {syncResult.skipped} 条。
              </p>
            )}
            {error && (
              <p role="alert" className="text-sm text-red-600">
                {error}
              </p>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
```

**Step 4: 运行测试，确认通过（GREEN）**

```bash
pnpm vitest run src/features/settings/calendar-sync-panel.test.tsx 2>&1 | tail -15
```
Expected: `4 passed`，0 failed。

**Step 5: Commit**

```bash
git add src/features/settings/calendar-sync-panel.tsx src/features/settings/calendar-sync-panel.test.tsx
git commit -m "feat: add CalendarSyncPanel component with tests"
```

---

## Task 9: 将 CalendarSyncPanel 嵌入设置页面

**Objective:** 在 `SettingsPage` 末尾（"关于"区块之前或之后）新增"日历同步"区块，引用 `CalendarSyncPanel`。

**Files:**
- Modify: `src/features/settings/settings-page.tsx`

**Step 1: 在文件顶部 import 区追加一行**

找到 `settings-page.tsx` 的最后一个 `import` 语句之后，追加：

```tsx
import { CalendarSyncPanel } from "./calendar-sync-panel";
```

**Step 2: 在 JSX 中添加日历同步区块**

在 `settings-page.tsx` 的 JSX 中，找到关于（版本检查）区块 `<div>` 的 **闭合 `</div>` 之后**，在 `<div className="flex-1 space-y-8">` 的 **最后一个子元素之后**，添加：

```tsx
        <div>
          <div className="mb-4">
            <h3 className="text-lg font-semibold text-gray-900">日历同步</h3>
            <p className="text-sm text-gray-500">将待办同步到 macOS 日历事件或提醒事项。</p>
          </div>
          <CalendarSyncPanel />
        </div>
```

**Step 3: 确认 TypeScript 编译无误**

```bash
pnpm build 2>&1 | grep -E "error TS|Error" | head -10
```
Expected: 无输出。

**Step 4: 运行全量测试，确认无回归**

```bash
pnpm vitest run 2>&1 | tail -10
```
Expected: 所有已有测试仍 PASS。

**Step 5: Commit**

```bash
git add src/features/settings/settings-page.tsx
git commit -m "feat: add CalendarSyncPanel to settings page"
```

---

## Task 10: 补充 Rust 单元测试

**Objective:** 为配置读写逻辑写 Rust 单元测试，确保 `CalendarSyncConfig` 默认值和持久化正确。

**Files:**
- Modify: `src-tauri/src/commands/calendar_sync.rs`（在文件末尾追加 `#[cfg(test)]` 模块）

**Step 1: 在 `calendar_sync.rs` 末尾追加**

```rust
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
```

**Step 2: 运行 Rust 测试**

```bash
cargo test --manifest-path src-tauri/Cargo.toml calendar_sync 2>&1 | tail -15
```
Expected：
```
test commands::calendar_sync::tests::config_defaults_when_file_missing ... ok
test commands::calendar_sync::tests::config_roundtrip ... ok
test commands::calendar_sync::tests::todo_record_calendar_fields_default_none ... ok

test result: ok. 3 passed; 0 failed
```

**Step 3: Commit**

```bash
git add src-tauri/src/commands/calendar_sync.rs
git commit -m "test: add Rust unit tests for calendar_sync config and TodoRecord fields"
```

---

## Task 11: 更新 Info.plist 权限声明（macOS TCC）

**Objective:** 在 Tauri 应用的 Info.plist 中声明日历和提醒事项权限，使 macOS 能弹出 TCC 授权弹窗。

**Files:**
- 检查 `src-tauri/` 中是否存在自定义 `Info.plist`

**Step 1: 确认是否存在自定义 Info.plist**

```bash
find src-tauri -name "Info.plist" 2>/dev/null
```

**情况 A — 不存在（大多数情况）：**

在 `tauri.conf.json` 的 `"app"` 对象中添加 `"macOSPrivateApi"` 以外的方式，Tauri 2 通过 `"bundle"` → `"macOS"` → `"infoPlist"` 注入键值：

```json
"bundle": {
  ...现有内容...,
  "macOS": {
    "infoPlist": {
      "NSCalendarsUsageDescription": "极趣 Note 需要访问您的日历以同步待办事项。",
      "NSRemindersUsageDescription": "极趣 Note 需要访问您的提醒事项以同步待办事项。"
    }
  }
}
```

**情况 B — 已存在 `Info.plist`：**

在 `<dict>` 内添加两对键值：

```xml
<key>NSCalendarsUsageDescription</key>
<string>极趣 Note 需要访问您的日历以同步待办事项。</string>
<key>NSRemindersUsageDescription</key>
<string>极趣 Note 需要访问您的提醒事项以同步待办事项。</string>
```

**Step 2: 验证 tauri.conf.json 格式合法**

```bash
node -e "JSON.parse(require('fs').readFileSync('src-tauri/tauri.conf.json','utf8')); console.log('valid')"
```
Expected: `valid`

**Step 3: Commit**

```bash
git add src-tauri/tauri.conf.json
git commit -m "feat: add NSCalendarsUsageDescription and NSRemindersUsageDescription to Info.plist"
```

---

## Task 12: 最终验证

**Objective:** 运行全量测试套件，确保所有测试通过，整个功能完整可编译。

**Step 1: 前端全量测试**

```bash
pnpm vitest run 2>&1 | tail -15
```
Expected: 所有测试 PASS，无 FAIL。

**Step 2: TypeScript 类型检查**

```bash
pnpm build 2>&1 | grep -E "error TS" | head -5
```
Expected: 无输出。

**Step 3: Rust 全量测试**

```bash
cargo test --manifest-path src-tauri/Cargo.toml 2>&1 | tail -10
```
Expected: `test result: ok.`，无 FAIL。

**Step 4: 开发模式启动验证（可选，需 macOS）**

```bash
pnpm tauri dev
```
Expected: 应用启动，设置页面底部出现"日历同步"区块，启用开关可操作。

**Step 5: 最终 Commit（若 Step 1-3 均通过）**

```bash
git tag -a v-calendar-sync-wip -m "calendar sync feature complete"
```

---

## 快速索引

| 文件 | 变更类型 | 关键内容 |
|---|---|---|
| `src-tauri/src/models.rs` | 修改 | `TodoRecord` 新增两字段；新增 `CalendarSyncConfig` 等 5 个类型 |
| `src-tauri/src/swift/CalendarBridge.swift` | 新建 | Swift EventKit CLI，5 个子命令 |
| `src-tauri/build.rs` | 修改 | macOS-only `swiftc` 编译步骤 |
| `src-tauri/tauri.conf.json` | 修改 | `resources` + macOS `infoPlist` |
| `src-tauri/src/commands/calendar_sync.rs` | 新建 | 4 个 Tauri 命令 + Rust 单元测试 |
| `src-tauri/src/commands/mod.rs` | 修改 | `pub mod calendar_sync` |
| `src-tauri/src/lib.rs` | 修改 | 注册 4 条命令 |
| `src/lib/tauri.ts` | 修改 | 类型 + 4 个 invoke 函数 |
| `src/features/settings/calendar-sync-panel.tsx` | 新建 | React 配置面板组件 |
| `src/features/settings/calendar-sync-panel.test.tsx` | 新建 | 4 个组件测试 |
| `src/features/settings/settings-page.tsx` | 修改 | 引入并渲染 `CalendarSyncPanel` |
