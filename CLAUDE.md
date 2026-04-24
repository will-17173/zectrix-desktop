# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

极趣 Note 4 桌面客户端 — Tauri 2 应用，用于待办管理和内容推送到墨水屏设备。

## 常用命令

```bash
# 安装依赖
pnpm install

# 开发模式（同时启动 Vite 和 Tauri 窗口）
pnpm tauri dev

# 前端测试
pnpm vitest run              # 运行所有测试
pnpm vitest watch            # 监听模式
pnpm vitest run src/features/todos/todo-list-page.test.tsx  # 运行单个测试

# Rust 测试
cargo test --manifest-path src-tauri/Cargo.toml

# 构建生产版本
pnpm tauri build

# 构建产物位于 src-tauri/target/release/bundle/
```

## 架构

### 前端-后端通信

- **前端入口**: `src/lib/tauri.ts` — 封装所有 Tauri invoke 命令，定义 TypeScript 类型
- **后端入口**: `src-tauri/src/lib.rs` — 注册所有 Tauri 命令到 invoke_handler
- **状态管理**: `src-tauri/src/state.rs` — AppState 结构，管理所有本地数据和同步逻辑

### 数据流

1. 应用启动时调用 `loadBootstrapState()` 加载所有本地数据（API Keys、设备、待办、模板、页面缓存）
2. 前端通过 `src/lib/tauri.ts` 的函数调用 Tauri 命令
3. Rust 后端通过 `commands/` 目录下的模块处理命令，操作 `AppState`
4. 本地数据存储在 `~/.zectrix-note/` 目录（JSON 文件 + 图片文件夹）

### 目录结构要点

- `src/features/` — 功能模块，每个模块包含页面组件和相关逻辑
- `src/components/ui/` — Radix UI 封装的可复用 UI 组件
- `src/components/layout/` — 布局组件（侧边栏、工具栏）
- `src-tauri/src/api/client.rs` — API 客户端，与 https://cloud.zectrix.com/open/v1 交互
- `src-tauri/src/storage/` — 本地 JSON 文件存储逻辑

### API 端点

所有请求需要 `X-API-Key` Header 认证：
- `/devices` — 设备管理
- `/todos` — 待办 CRUD
- `/devices/{id}/display/text` — 推送文本
- `/devices/{id}/display/image` — 推送图片

### 本地优先架构

待办数据优先存储本地，支持离线操作：
- 新建待办生成 `localId`（格式：`todo-{timestamp}-{counter}`）
- `dirty` 字段标记本地修改，同步时上传到云端
- `deleted` 字段标记已删除，同步时从云端删除
- 同步流程：上传本地修改 → 拉取云端数据 → 合并（本地修改优先）

## 开发注意事项

### 前端

- 使用 TanStack Query 的 `useQuery`/`useMutation` 时，数据来源是 Tauri 命令而非 HTTP 请求
- 路由在 `src/app/App.tsx` 中直接定义（未使用 React Router 的路由配置）
- UI 组件使用 Radix UI + Tailwind CSS 4，遵循现有组件的样式模式

### Rust

- AppState 使用 `tauri::Manager` 的 `.manage()` 注册，命令中通过 `tauri::State` 获取
- API Key 使用 `keyring` crate 存储，不要在代码中硬编码或明文存储
- 图片处理使用 `image` crate，支持裁剪、旋转、翻转等操作