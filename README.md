# Zectrix Desktop

极趣 Note 4桌面客户端 — 本地优先的待办管理、设备管理、文本/图片推送、手动同步。

## 技术栈

- **前端**: React 19 + TypeScript + React Router + TanStack Query
- **桌面框架**: Tauri 2
- **后端**: Rust (reqwest, keyring, image)

## 开发

### 前提条件

- Node.js 20+
- pnpm
- Rust (stable)
- Tauri CLI (`cargo install tauri-cli`)

### 本地开发

```bash
pnpm install
pnpm tauri dev
```

### 测试

```bash
# 前端测试
pnpm vitest run

# Rust 测试
cargo test --manifest-path src-tauri/Cargo.toml
```

### 构建

```bash
pnpm tauri build
```

## 功能模块

- **待办事项**: 本地 CRUD、状态切换、推送到设备
- **文本推送**: 模板管理、结构化文本推送
- **图片推送**: 上传、裁切（4:3）、旋转/翻转、400×300 导出、推送
- **设备管理**: MAC 地址验证、本地缓存
- **设置**: API Key 加密存储（OS Keyring）
- **同步**: 手动同步，上传脏数据后拉取云端最新
