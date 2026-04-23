# Zectrix Desktop

极趣 Note 4 桌面客户端 — 待办管理、内容推送工具，配合 Zectrix 墨水屏设备使用。

---

## 1. 项目介绍

Zectrix Desktop 是极趣 Note 4 服务的桌面客户端应用，为用户提供便捷的本地待办管理和内容推送功能。

### 核心特性

- **本地优先架构** — 待办数据优先存储在本地，支持离线操作，手动同步到云端
- **多类型内容推送** — 支持文本、图片、涂鸦等多种内容推送到墨水屏设备
- **设备管理** — 通过 MAC 地址绑定设备，本地缓存设备信息
- **安全存储** — API Key 使用操作系统级 Keyring 加密存储
- **跨平台支持** — 支持 Windows、 macOS、Linux

### 技术栈

| 层级 | 技术 |
|------|------|
| 前端 | React 19 + TypeScript + React Router 7 + TanStack Query |
| UI | Tailwind CSS 4 + Radix UI + Lucide Icons |
| 桌面框架 | Tauri 2 |
| 后端 | Rust (reqwest, image, keyring) |
| 测试 | Vitest (前端) + cargo test (Rust) |

---

## 2. 使用指南

### 2.1 初次配置

首次使用需要进行以下配置：

1. **获取 API Key**
   - 访问 [https://cloud.zectrix.com/home/api-keys](https://cloud.zectrix.com/home/api-keys) 创建 API Key
   - 在应用「设置」页面添加 API Key

2. **绑定设备**
   - 在「设置」页面点击「添加设备」
   - 输入设备的 MAC 地址（格式：`XX:XX:XX:XX:XX:XX`）
   - 系统会自动验证设备并获取设备别名

### 2.2 待办事项

待办事项是应用的核心功能，支持完整的 CRUD 操作和云端同步。

**创建待办**
- 点击「添加待办」按钮
- 输入标题、描述、截止日期/时间
- 可选择关联的设备

**管理待办**
- **完成/恢复** — 点击复选框切换状态
- **编辑** — 点击「编辑」修改内容
- **删除** — 点击「删除」移除待办
- **推送** — 点击「推送」将待办内容发送到设备显示

**同步状态标识**
- 黄色「本地」标签 — 数据仅存储在本地，未同步
- 绿色「云端」标签 — 数据已同步到云端

### 2.3 文本推送

「文本推送」功能支持将文本内容直接推送到设备指定页面。

**使用步骤**
1. 在文本内容区域输入要推送的文字（支持换行）
2. 选择字体大小（16px - 64px）
3. 选择目标页面（第 1-5 页）
4. 点击「推送」按钮发送

### 2.4 图片推送

「图片推送」功能支持图片导入、编辑和推送。

**导入图片**
1. 点击「导入图片」按钮
2. 选择本地图片文件
3. 在编辑器中进行处理：
   - **裁切** — 拖拽裁切框调整区域
   - **旋转** — 支持 90°、180°、270° 旋转
   - **翻转** — 支持水平/垂直翻转
4. 图片会自动调整为 400×300 的设备适配尺寸
5. 保存到本地图库

**推送图片**
- 在图库列表中选择图片
- 选择目标页面
- 点击「推送」按钮

### 2.5 涂鸦推送

「涂鸦推送」功能支持在画布上自由绘制并推送。

**绘制工具**
- **画笔** — 选择颜色和大小进行绘制
- **橡皮擦** — 擦除绘制内容
- **清空** — 清除整个画布

**推送涂鸦**
- 完成绘制后选择目标页面
- 点击「推送」按钮

### 2.6 自由排版

「自由排版」功能提供更灵活的文本排版选项。

- 支持 5000 字以内的文本
- 可选字号范围：12px - 48px
- 选择页码后推送

### 2.7 数据同步

点击工具栏的「同步」按钮执行数据同步：

**同步流程**
1. 上传本地修改的待办数据到云端
2. 从云端拉取最新数据
3. 合并本地和云端数据（本地修改优先）

**注意事项**
- 同步需要网络连接
- 首次同步前必须配置 API Key
- 离线状态下同步按钮会提示「当前离线」

---

## 3. 开发指南

### 3.1 开发环境

**前提条件**

| 工具 | 版本要求 |
|------|----------|
| Node.js | 20+ |
| pnpm | 推荐 |
| Rust | stable |
| Tauri CLI | cargo install tauri-cli |

**安装依赖**

```bash
pnpm install
```

### 3.2 本地开发

启动开发服务器：

```bash
pnpm tauri dev
```

这会同时启动：
- Vite 开发服务器（前端热更新）
- Tauri 窗口应用

### 3.3 项目结构

```
zectrix-desktop/
├── src/                          # 前端源码
│   ├── app/                      # 应用入口和路由
│   ├── components/               # UI 组件
│   │   ├── layout/               # 布局组件
│   │   └ ui/                     # Radix UI 封装组件
│   ├── features/                 # 功能模块
│   │   ├── devices/              # 设备管理
│   │   ├── free-layout/          # 自由排版
│   │   ├── images/               # 图片推送
│   │   ├── settings/             # 设置页面
│   │   ├── sketch/               # 涂鸦推送
│   │   ├── templates/            # 文本推送
│   │   └ todos/                  # 待办事项
│   └── lib/                      # 工具库
│       └── tauri.ts              # Tauri 命令封装
├── src-tauri/                    # Rust 后端
│   ├── src/
│   │   ├── api/                  # API 客户端
│   │   ├── commands/             # Tauri 命令
│   │   ├── models.rs             # 数据模型
│   │   ├── state.rs              # 应用状态管理
│   │   ├── storage/              # 本地存储
│   │   └── lib.rs                # 入口
│   └── Cargo.toml
├── package.json
├── vite.config.ts
└── tailwind.config.ts
```

### 3.4 测试

**前端测试**

```bash
pnpm vitest run        # 运行所有测试
pnpm vitest watch      # 监听模式
```

**Rust 测试**

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

### 3.5 构建

构建生产版本：

```bash
pnpm tauri build
```

构建产物位于 `src-tauri/target/release/bundle/` 目录。

### 3.6 数据存储

应用数据存储在用户目录下的 `.zectrix-note` 文件夹：

```
~/.zectrix-note/
├── config.json          # 应用配置（同步时间等）
├── api_keys.json        # API Key 记录
├── devices.json         # 设备缓存
├── todos.json           # 待办数据
├── text_templates.json  # 文本模板
├── image_templates.json # 图片模板索引
└── images/              # 图片文件存储
    ├── 1.png
    ├── 2.png
    └── ...
```

### 3.7 API 参考

应用通过 `https://cloud.zectrix.com/open/v1` API 与云端交互：

| 端点 | 方法 | 功能 |
|------|------|------|
| `/devices` | GET | 获取设备列表 |
| `/todos` | GET/POST | 待办 CRUD |
| `/todos/{id}` | PUT/DELETE | 更新/删除待办 |
| `/todos/{id}/complete` | PUT | 标记完成 |
| `/devices/{id}/display/text` | POST | 推送文本 |
| `/devices/{id}/display/structured-text` | POST | 推送结构化文本 |
| `/devices/{id}/display/image` | POST | 推送图片 |

所有请求需要 `X-API-Key` Header 认证。

---

## 许可证

MIT License