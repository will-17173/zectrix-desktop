# macOS 日历同步功能设计

## 概述

为待办功能增加 macOS 日历同步能力，支持与 macOS 日历事件（Calendar.app）和提醒事项（Reminders.app）双向同步。用户可在应用内配置同步方向、目标类型和目标日历本，通过手动触发执行同步。

## 整体架构

### Swift 原生层

新建 `src-tauri/src/swift/CalendarBridge.swift`，编译为独立 CLI 工具 `calendar-bridge`，打包进 Tauri 应用 `resources/` 目录。使用 EventKit 框架，负责：

- 请求日历/提醒事项 TCC 权限
- 列出可用日历本和提醒事项列表
- 对日历事件和提醒事项执行 CRUD
- 以 JSON 格式通过 stdout 返回结果，错误写入 stderr

CLI 接口：

```
calendar-bridge request-permission
calendar-bridge list-calendars --type [calendar|reminder]
calendar-bridge list-items --calendar-id <id>
calendar-bridge create-item --data '<json>'
calendar-bridge update-item --external-id <id> --data '<json>'
calendar-bridge delete-item --external-id <id>
```

### Rust 命令层

新建 `src-tauri/src/commands/calendar_sync.rs`，通过 `std::process::Command` 调用 `calendar-bridge`。负责：

- 暴露 Tauri 命令给前端
- 读写 `CalendarSyncConfig`（持久化到 `~/.zectrix-note/calendar_sync.json`）
- 执行同步逻辑，比较时间戳决定冲突方向

### 前端配置层

在设置页面新增"日历同步"标签页，提供配置面板和手动触发入口。

## 数据模型

### CalendarSyncConfig（`~/.zectrix-note/calendar_sync.json`）

```rust
struct CalendarSyncConfig {
    enabled: bool,
    direction: SyncDirection,        // ToCalendar | FromCalendar | Bidirectional
    target_type: CalendarTargetType, // CalendarEvent | Reminder
    target_calendar_id: Option<String>, // EventKit calendar/list identifier
}
```

### TodoRecord 新增字段（现有 `todos.json`）

```rust
calendar_external_id: Option<String>, // EventKit 的 calendarItemIdentifier
calendar_synced_at: Option<String>,   // 上次同步时间，RFC3339
```

### Swift 层数据结构

```swift
struct CalendarItem: Codable {
    var externalId: String
    var title: String
    var dueDate: String?    // ISO8601
    var isCompleted: Bool
    var lastModified: String // ISO8601
}
```

### 字段映射

| TodoRecord | CalendarItem |
|---|---|
| `title` | `title` |
| `due_date` + `due_time` | `dueDate` |
| `status == 1` | `isCompleted` |
| `calendar_external_id` | `externalId` |

## 同步流程

用户点击"立即同步"按钮，触发 `invoke("sync_calendar")`：

1. Rust 读取 `CalendarSyncConfig`
2. 调用 `calendar-bridge list-items` 获取目标日历全部条目
3. 按同步方向执行：

**ToCalendar（待办 → 日历）：**
- 有 `calendar_external_id`：比较 `updatedAt` vs `lastModified`，app 更新则调用 `update-item`
- 无 `calendar_external_id`：调用 `create-item`，回写 `external_id` 到 TodoRecord

**FromCalendar（日历 → 待办）：**
- 日历条目有对应 TodoRecord：比较时间，日历更新则更新 TodoRecord
- 无对应 TodoRecord：创建新 TodoRecord，写入 `calendar_external_id`

**Bidirectional（双向）：**
- 以上两者合并，冲突时取 `lastModified` 更新的一方为准

**删除处理：**
- app 内 `deleted=true` 且有 `calendar_external_id` 的待办，同步时调用 `delete-item` 清除日历端

4. 保存更新后的 `todos.json`
5. 返回同步摘要：新增/更新/跳过/删除数量

## 权限配置

`Info.plist` 新增：
- `NSCalendarsUsageDescription`：说明访问日历的用途
- `NSRemindersUsageDescription`：说明访问提醒事项的用途

首次点击同步时，调用 `request-permission` 触发 TCC 授权弹窗，授权后永久保存。

## 前端 UI

设置页面新增"日历同步"标签页，从上到下布局：

1. **启用开关** — 关闭时隐藏以下选项
2. **目标类型** — 单选：日历事件 / 提醒事项
3. **目标日历本** — shadcn/ui Select 下拉，调用 `list-calendars` 动态加载
4. **同步方向** — 单选：仅推送到日历 / 仅从日历导入 / 双向同步
5. **立即同步按钮** — 点击后显示加载状态，完成后展示结果摘要（如"新增 2 条，更新 1 条"）

新增 Tauri 命令：
- `get_calendar_sync_config` → `CalendarSyncConfig`
- `save_calendar_sync_config(config)` → `void`
- `list_calendars(target_type)` → `Vec<CalendarInfo>`
- `sync_calendar()` → `SyncResult`

## 关键约束

- 目标日历本选择使用现有 `src/components/ui/select.tsx` 组件，不使用原生 `<select>`
- Swift CLI 编译产物须通过 Tauri 的 `tauri.conf.json` `resources` 字段打包
- `TodoRecord` 新增字段需向后兼容：旧 `todos.json` 反序列化时缺省为 `None`
