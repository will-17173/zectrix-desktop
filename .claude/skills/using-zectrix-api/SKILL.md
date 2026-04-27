---
name: using-zectrix-api
description: >
  使用极趣云平台（Zectrix）开放 API 的指南。当用户需要调用 Zectrix API
  管理设备、待办事项、推送内容到墨水屏设备，或编写与 Zectrix 云平台交互的脚本时，
  使用此 skill。触发场景包括：用户提到 "Zectrix API"、"极趣云"、"墨水屏推送"、
  "设备推送"、API 集成、curl 命令调用 Zectrix 接口、或需要在代码中调用 Zectrix
  设备/待办/显示相关接口。
---

# Zectrix Cloud API 使用指南

## 概述

极趣云平台提供 RESTful API，用于管理设备、待办事项和向墨水屏设备推送内容。

## 基础信息

| 项目 | 值 |
|------|-----|
| Base URL | `https://cloud.zectrix.com/open/v1` |
| 认证方式 | Header `X-API-Key: <your_api_key>` |
| API Key 格式 | `zt_<random_string>` |
| 响应格式 | JSON，统一包装为 `{ code: 0, data: ..., msg: "..." }` |
| 成功判定 | `code === 0` |

## 错误处理

所有接口都返回统一格式 `{ code: number, data: T, msg?: string }`：
- `code === 0`：成功
- `code !== 0`：失败，错误信息在 `msg` 字段
- HTTP 401：API Key 无效
- HTTP 404：资源不存在

## API 端点

完整的 API 参考文档位于 `references/api-reference.md`。以下是各端点的简要索引：

### 设备管理
| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/devices` | 获取设备列表 |

### 待办事项
| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/todos` | 获取待办列表（支持 `?status=0/1` 和 `?deviceId=xxx` 过滤） |
| POST | `/todos` | 创建待办 |
| PUT | `/todos/{id}` | 更新待办 |
| PUT | `/todos/{id}/complete` | 切换完成状态 |
| DELETE | `/todos/{id}` | 删除待办 |

### 显示推送
| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/devices/{deviceId}/display/image` | 推送图片（multipart，最多5张，单张≤2MB） |
| POST | `/devices/{deviceId}/display/text` | 推送纯文本（最多5000字，支持 fontSize 12-48） |
| POST | `/devices/{deviceId}/display/structured-text` | 推送标题+正文 |
| DELETE | `/devices/{deviceId}/display/pages/{pageId}` | 删除指定页面（不传 pageId 则清空全部） |

## 使用模式

### 快速示例：用 curl 获取设备列表

```bash
curl "https://cloud.zectrix.com/open/v1/devices" \
  -H "X-API-Key: zt_your_key_here"
```

### 快速示例：推送文本到设备

```bash
curl "https://cloud.zectrix.com/open/v1/devices/AA:BB:CC:DD:EE:FF/display/text" \
  -H "X-API-Key: zt_your_key_here" \
  -H "Content-Type: application/json" \
  -d '{"text":"Hello 墨水屏","fontSize":20,"pageId":"1"}'
```

### 快速示例：推送图片到设备

```bash
curl "https://cloud.zectrix.com/open/v1/devices/AA:BB:CC:DD:EE:FF/display/image" \
  -H "X-API-Key: zt_your_key_here" \
  -F "images=@photo.png" \
  -F "pageId=1"
```

## 工作流程

当用户要求使用 Zectrix API 时，按以下步骤操作：

1. **确认 API Key**：用户需要提供一个有效的 API Key。如果用户不知道，引导其去 cloud.zectrix.com 获取。
2. **确认设备**：推送类操作需要 `deviceId`（MAC 地址格式如 `AA:BB:CC:DD:EE:FF`），可先调用 `GET /devices` 获取可用设备列表。
3. **构造请求**：参考 `references/api-reference.md` 中的完整接口文档，使用正确的 HTTP 方法、路径和参数。
4. **处理响应**：检查 `code === 0` 判断成功，失败时读取 `msg` 获取错误详情。

## 编程语言集成

当用户需要在特定语言中调用 API 时，根据语言给出惯用代码：

- **Rust**：使用 `reqwest` crate（参考项目 `src-tauri/src/api/client.rs` 中的现有实现）
- **Python**：使用 `requests` 库
- **JavaScript/TypeScript**：使用 `fetch` API
- **Shell**：使用 `curl`

## 备注

- 本项目的 `src-tauri/src/api/client.rs` 包含了所有 API 端点的 Rust 实现，可作为参考
- 前端通过 `src/lib/tauri.ts` 中的 Tauri invoke 命令间接调用这些 API，不直接发起 HTTP 请求
- API Key 使用 `keyring` crate 安全存储，不应在代码中硬编码
