# 页面管理功能设计文档

**日期**: 2024-04-24
**作者**: Claude Code

## 功能概述

新增"页面管理"菜单，展示每个设备的5个墨水屏页面内容，支持查看缩略图预览和删除页面内容。

## 用户需求

- 在右边显示5个页面卡片
- 其他推送功能（涂鸦推送、图片推送、自由排版、文本推送）推送内容到页面后，页面管理显示对应内容
- 页面管理可以删除页面内容（清空页面，但页面本身保留）
- 页面内容按设备区分，每个设备有独立的5个页面

## 数据结构设计

新增 SQLite 表 `page_cache`：

```sql
CREATE TABLE page_cache (
  id INTEGER PRIMARY KEY,
  device_id TEXT NOT NULL,           -- 设备 MAC 地址
  page_id INTEGER NOT NULL,          -- 页码 1-5
  content_type TEXT NOT NULL,        -- 内容类型: "sketch" | "image" | "text" | "structured_text"
  thumbnail TEXT,                    -- 缩略图路径（图片/涂鸦）或文本摘要
  metadata TEXT,                     -- JSON 格式额外信息
  pushed_at TEXT NOT NULL,           -- 推送时间
  UNIQUE(device_id, page_id)         -- 每个设备的每个页面只有一条记录
);
```

**content_type 来源映射**：
- `sketch` — 涂鸦推送
- `image` — 图片推送
- `text` — 自由排版（纯文本）
- `structured_text` — 文本推送（标题+正文）

**metadata 示例**：
- 文本推送：`{"title": "买牛奶", "bodyPreview": "记得去超市..."}`
- 自由排版：`{"fontSize": 20}`
- 图片/涂鸦：`{"width": 400, "height": 300}`

## 后端 API 设计

### 新增 Tauri 命令

**1. 获取页面列表**
```typescript
getPageCacheList(deviceId: string): Promise<PageCacheRecord[]>
```
返回指定设备的5个页面内容信息，按 pageId 排序。

**2. 删除页面**
```typescript
deletePageCache(deviceId: string, pageId: number): Promise<void>
```
- 先删除本地缓存记录和缩略图文件
- 再调用云端 API `DELETE /devices/{deviceId}/display/pages/{pageId}`

### 修改现有命令

以下命令推送成功后自动写入 page_cache 表：
- `push_sketch`
- `push_image_template`
- `push_text`（自由排版）
- `push_structured_text`（文本推送）

### 类型定义

```typescript
export type PageCacheRecord = {
  deviceId: string;
  pageId: number;          // 1-5
  contentType: "sketch" | "image" | "text" | "structured_text";
  thumbnail: string | null; // 缩略图路径或文本摘要
  metadata: Record<string, unknown>;
  pushedAt: string;
};
```

## 前端组件设计

### 路由与导航

- 新增路由 `/page-manager`
- 侧边栏添加"页面管理"菜单项（使用 `Layers` 图标）

### PageManagerPage 组件结构

```
- 设备选择器（下拉框，选择当前操作的设备）
- 5个页面卡片（横向排列或网格布局）
  - 每个卡片显示：
    - 页码标签："第 1 页"
    - 内容类型图标（涂鸦/图片/文本图标）
    - 缩略图区域：图片显示缩略图，文本显示摘要前3行
    - 删除按钮（仅当有内容时显示）
    - 空状态提示："暂无内容"
```

### 状态管理

组件状态：
- `selectedDeviceId` — 当前选中的设备
- `pageList` — 该设备的5个页面数据列表
- `loading` — 加载状态
- `deletingPageId` — 正在删除的页码（用于显示加载状态）

### 交互流程

- 进入页面时：默认选中第一个设备，加载其页面列表
- 切换设备时：重新加载对应设备的页面列表
- 点击删除：立即调用 `deletePageCache`，成功后更新列表（无需确认框）

## 数据流设计

**刷新策略**：统一在 App.tsx 刷新

- `loadBootstrapState` 返回值新增 `pageCache` 字段
- App.tsx 传递 `pageCache` 数据给 PageManagerPage
- 推送成功后调用 `reload()` 重新加载全部状态

这与现有架构一致（todos/templates 推送后都是 reload）。

## 实现范围

### 后端修改
1. 新增 `page_cache` 表和 CRUD 操作
2. 新增 `get_page_cache_list` 命令
3. 新增 `delete_page_cache` 命令
4. 修改 `push_sketch` 写入缓存
5. 修改 `push_image_template` 写入缓存
6. 修改 `push_text` 写入缓存
7. 修改 `push_structured_text` 写入缓存
8. 修改 `load_bootstrap_state` 返回 pageCache

### 前端修改
1. 新增 `src/features/page-manager/page-manager-page.tsx`
2. 修改 `src/components/layout/app-sidebar.tsx` 添加菜单项
3. 修改 `src/app/App.tsx` 添加路由和传递数据
4. 修改 `src/lib/tauri.ts` 添加类型定义和调用函数
5. 修改路由配置添加 `/page-manager` 路径

## 云端 API 参考

删除页面接口：
```
DELETE /devices/{deviceId}/display/pages/{pageId}
```
返回：`{ "code": 0, "msg": "success" }`