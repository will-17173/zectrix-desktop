# 图片循环推送功能设计文档

## 概述

在图片推送菜单（本地图库）中新增「循环相册」功能，允许用户选择本地文件夹，配置循环间隔、目标设备和页面，定时轮播显示文件夹中的所有图片。

## 应用场景

电子相册展示 - 设备循环显示文件夹里的图片，类似电子相册轮播效果。

## 数据模型

### ImageLoopTaskRecord

```rust
// src-tauri/src/models.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageLoopTaskRecord {
    pub id: i64,
    pub name: String,                      // 任务名称（用户自定义）
    pub folder_path: String,               // 图片文件夹路径
    pub device_id: String,                 // 目标设备 ID
    pub page_id: u32,                      // 目标页面
    pub interval_seconds: u32,             // 循环间隔（秒）
    pub duration_type: String,             // "none" | "until_time" | "for_duration"
    pub end_time: Option<String>,          // 结束时间点（当天时间，格式 "HH:MM"，如 "18:00"）
    pub duration_minutes: Option<u32>,     // 运行时长（分钟）
    
    // 运行状态
    pub status: String,                    // "idle" | "running" | "completed" | "error"
    pub current_index: u32,                // 当前播放图片序号（0-based）
    pub total_images: u32,                 // 文件夹内图片总数
    pub started_at: Option<String>,        // 启动时间（ISO 格式）
    pub last_push_at: Option<String>,      // 最后推送时间
    pub error_message: Option<String>,     // 错误信息
    pub created_at: String,                // 创建时间
    pub updated_at: String,                // 更新时间
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
    pub image_files: Vec<String>,          // 图片文件名列表
    pub warning: Option<String>,           // 无图片时的警告信息
}
```

### 存储位置

任务配置存储在 `~/.zectrix-note/image_loop_tasks.json`

## UI 设计

### 页面布局

在现有 `ImageTemplatesPage` 页面底部新增「循环相册」区域：

```
┌─────────────────────────────────────────────────────────────┐
│  本地图库                                                    │
│  [导入图片]                                                  │
│  ┌─────┐ ┌─────┐ ┌─────┐                                    │
│  │图片1│ │图片2│ │图片3│ ...                                │
│  └─────┘ └─────┘ └─────┘                                    │
├─────────────────────────────────────────────────────────────┤
│  循环相册                                                    │
│  [+ 新建任务]                                                │
│                                                              │
│  ┌─────────────────────────────────────────────────────┐    │
│  │ 🟢 任务名称: 周末旅行相册                    [▶] [⏹] │    │
│  │ 当前: 第 3/12 张                                    │    │
│  │ ▶ 展开                                               │    │
│  └─────────────────────────────────────────────────────┘    │
│                                                              │
│  ┌─────────────────────────────────────────────────────┐    │
│  │ ⚪ 任务名称: 产品展示                        [▶] [⏹] │    │
│  │ 未启动                                              │    │
│  │ ▶ 展开                                               │    │
│  └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

### 任务卡片展开详情

```
┌─────────────────────────────────────────────────────┐
│ 🟢 任务名称: 周末旅行相册                    [▶] [⏹] │
│ 当前: 第 3/12 张                                    │
│ ▼ 收起                                               │
├─────────────────────────────────────────────────────┤
│ 文件夹: /Users/xxx/Pictures/travel                   │
│ 目标: 我的设备 (AA:BB:CC) / 第 2 页                   │
│ 间隔: 30 秒                                          │
│ 持续: 运行至 18:00                                   │
│ 已运行: 15 分钟，播放 3 轮                            │
│ 最后推送: 10:45:30                                   │
│ ─────────────────────────────────────────            │
│ [编辑] [删除]                                        │
└─────────────────────────────────────────────────────┘
```

### 新建/编辑任务对话框

```
┌─────────────────────────────────────────────────┐
│  新建循环相册任务                          [✕]  │
├─────────────────────────────────────────────────┤
│                                                  │
│  任务名称                                        │
│  ┌─────────────────────────────────────────────┐│
│  │ 周末旅行相册                                ││
│  └─────────────────────────────────────────────┘│
│                                                  │
│  图片文件夹                              [选择] │
│  ┌─────────────────────────────────────────────┐│
│  │ /Users/xxx/Pictures/travel                  ││
│  └─────────────────────────────────────────────┘│
│  ⚠️ 已检测到 12 张图片 (jpg/png)                │
│                                                  │
│  目标设备                                        │
│  ┌─────────────────────────────────────────────┐│
│  │ 我的设备 (AA:BB:CC)                    [▼]  ││
│  └─────────────────────────────────────────────┘│
│                                                  │
│  目标页面                                        │
│  ┌─────────────────────────────────────────────┐│
│  │ 第 2 页                                [▼]  ││
│  └─────────────────────────────────────────────┘│
│                                                  │
│  循环间隔                                        │
│  ┌─────────────────────────────────────────────┐│
│  │ 30 秒                                  [▼]  ││
│  └─────────────────────────────────────────────┘│
│  选项: 10秒、30秒、1分钟、5分钟、10分钟          │
│                                                  │
│  持续时间                                        │
│  ○ 无限制                                        │
│  ● 运行至指定时间    ┌─────────────┐            │
│  │                   │ 18:00       │            │
│  │                   └─────────────┘            │
│  ○ 运行指定时长      ┌─────────────┐ 分钟       │
│  │                   │ 60          │            │
│  │                   └─────────────┘            │
│                                                  │
│                    [取消]  [保存任务]            │
└─────────────────────────────────────────────────┘
```

### 状态指示器

- 🟢 绿色圆点：运行中
- ⚪ 白色圆点：未启动 (idle)
- 🟡 黄色圆点：已完成
- 🔴 红色圆点：错误状态

## 核心工作流程

### 创建任务流程

```
用户点击"新建任务"
    ↓
打开表单对话框
    ↓
用户点击"选择文件夹"
    ↓
调用 Tauri 文件对话框 API (select_folder_dialog)
    ↓
用户选择文件夹后，调用 scan_image_folder
    ↓
后端扫描文件夹，筛选图片文件（jpg/png/jpeg/gif/bmp/webp）
    ↓
返回图片数量和文件列表
    ↓
表单显示检测结果（图片数量或警告）
    ↓
用户填写其他配置并保存
    ↓
调用 create_image_loop_task 保存任务
    ↓
任务列表刷新，显示新任务（状态: idle）
```

### 启动任务流程

```
用户点击"▶ 启动"
    ↓
调用 start_image_loop_task
    ↓
后端更新状态为 running，设置 startedAt，重置 currentIndex 为 0
    ↓
前端定时器开始运行
    ↓
定时器每隔 intervalSeconds 触发
    ↓
调用 push_folder_image(taskId)
    ↓
后端读取文件夹图片列表，获取当前 index 的图片
    ↓
加载图片，处理为 400x300（复用现有 process_image 逻辑）
    ↓
调用现有 push_image API 推送到设备
    ↓
更新 currentIndex++，记录 lastPushAt
    ↓
若到达文件夹末尾，currentIndex 重置为 0（开始新一轮）
    ↓
检查持续时间条件：
  - duration_type="until_time": 当前时间 >= end_time
  - duration_type="for_duration": 已运行时间 >= duration_minutes
    ↓
若达到结束条件，自动更新状态为 completed
    ↓
前端检测到状态变化，停止定时器
```

### 停止任务流程

```
用户点击"⏹ 停止"
    ↓
调用 stop_image_loop_task
    ↓
后端更新状态为 idle，清除 startedAt
    ↓
前端停止定时器
```

### 错误处理

推送失败时：
- 更新任务状态为 error
- 记录 error_message
- 前端停止定时器
- 任务卡片显示红色状态和错误信息
- 用户可点击"▶"重新启动（从当前 index 继续）

空文件夹处理：
- 允许创建任务，但显示警告："该文件夹未检测到图片"
- 运行时跳过，状态保持 idle 或立即 completed

## 后端 Tauri 命令

### 新增命令列表

```rust
// src-tauri/src/commands/image_loop.rs

#[tauri::command]
pub fn list_image_loop_tasks(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<ImageLoopTaskRecord>, String>

#[tauri::command]
pub fn create_image_loop_task(
    state: tauri::State<'_, AppState>,
    input: ImageLoopTaskInput,
) -> Result<ImageLoopTaskRecord, String>

#[tauri::command]
pub fn update_image_loop_task(
    state: tauri::State<'_, AppState>,
    task_id: i64,
    input: ImageLoopTaskInput,
) -> Result<ImageLoopTaskRecord, String>

#[tauri::command]
pub fn delete_image_loop_task(
    state: tauri::State<'_, AppState>,
    task_id: i64,
) -> Result<(), String>

#[tauri::command]
pub fn start_image_loop_task(
    state: tauri::State<'_, AppState>,
    task_id: i64,
) -> Result<ImageLoopTaskRecord, String>

#[tauri::command]
pub fn stop_image_loop_task(
    state: tauri::State<'_, AppState>,
    task_id: i64,
) -> Result<ImageLoopTaskRecord, String>

#[tauri::command]
pub async fn push_folder_image(
    state: tauri::State<'_, AppState>,
    task_id: i64,
) -> Result<ImageLoopTaskRecord, String>

#[tauri::command]
pub fn scan_image_folder(
    state: tauri::State<'_, AppState>,
    folder_path: String,
) -> Result<ImageFolderScanResult, String>

#[tauri::command]
pub fn select_folder_dialog() -> Result<Option<String>, String>
```

### 命令实现要点

**scan_image_folder**：
- 使用 `std::fs::read_dir` 遍历文件夹
- 筛选扩展名为 jpg/png/jpeg/gif/bmp/webp 的文件
- 返回总数和文件名列表

**push_folder_image**：
- 从任务记录获取 folder_path 和 current_index
- 扫描文件夹获取图片列表
- 加载 current_index 对应的图片
- 复用现有 `process_image` 逻辑处理为 400x300
- 调用 `crate::api::client::push_image` 推送
- 更新 current_index（循环到 0 如果到达末尾）
- 检查持续时间条件，自动停止

**select_folder_dialog**：
- 使用 `tauri::api::dialog::blocking::FileDialogBuilder`
- 设置默认路径为用户主目录
- 只允许选择文件夹

## 前端组件结构

### 文件结构

```
src/features/images/
├── image-templates-page.tsx        # 现有页面，底部新增循环相册区域
├── image-editor-dialog.tsx         # 现有组件
├── image-loop-task-list.tsx        # 新增：循环任务列表组件
├── image-loop-task-card.tsx        # 新增：单个任务卡片组件
├── image-loop-task-dialog.tsx      # 新增：新建/编辑任务对话框
└── use-image-loop-runner.ts        # 新增：定时器管理 Hook
```

### useImageLoopRunner Hook

```typescript
// src/features/images/use-image-loop-runner.ts

export function useImageLoopRunner(
  tasks: ImageLoopTask[],
  pushFolderImage: (taskId: number) => Promise<ImageLoopTask>,
) {
  const timersRef = useRef<Map<number, NodeJS.Timeout>>(new Map());
  
  useEffect(() => {
    const runningTasks = tasks.filter(t => t.status === 'running');
    const runningIds = new Set(runningTasks.map(t => t.id));
    
    // 清理已停止任务的定时器
    for (const [taskId, timer] of timersRef.current.entries()) {
      if (!runningIds.has(taskId)) {
        clearInterval(timer);
        timersRef.current.delete(taskId);
      }
    }
    
    // 为运行中的任务创建定时器（如果尚未创建）
    for (const task of runningTasks) {
      if (!timersRef.current.has(task.id)) {
        const timer = setInterval(async () => {
          try {
            const updated = await pushFolderImage(task.id);
            // 更新任务状态（通过父组件的 refetch 或局部状态更新）
            if (updated.status !== 'running') {
              clearInterval(timer);
              timersRef.current.delete(task.id);
            }
          } catch (e) {
            // 错误已由后端记录到 task.status/error_message
            clearInterval(timer);
            timersRef.current.delete(task.id);
          }
        }, task.intervalSeconds * 1000);
        
        timersRef.current.set(task.id, timer);
      }
    }
    
    return () => {
      // 组件卸载时清理所有定时器
      for (const timer of timersRef.current.values()) {
        clearInterval(timer);
      }
    };
  }, [tasks, pushFolderImage]);
}
```

### 前端 Tauri 命令封装

```typescript
// src/lib/tauri.ts 新增

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

## 循环间隔预设选项

```typescript
const INTERVAL_OPTIONS = [
  { value: 10, label: "10 秒" },
  { value: 30, label: "30 秒" },
  { value: 60, label: "1 分钟" },
  { value: 300, label: "5 分钟" },
  { value: 600, label: "10 分钟" },
];
```

## BootstrapState 扩展

将循环任务列表加入启动加载：

```rust
// src-tauri/src/models.rs BootstrapState 新增字段

pub struct BootstrapState {
    // ... 现有字段
    pub image_loop_tasks: Vec<ImageLoopTaskRecord>,
}
```

前端在应用启动时通过 `loadBootstrapState` 获取所有循环任务。

## 测试要点

### 后端测试

- 创建/更新/删除任务的 CRUD 操作
- scan_image_folder 正确识别图片文件
- push_folder_image 图片处理和推送逻辑
- 持续时间条件判断（结束时间/时长）

### 前端测试

- 任务列表渲染和状态显示
- 表单验证（名称必填、文件夹已选择）
- 定时器正确启动和停止
- 错误状态的显示和处理