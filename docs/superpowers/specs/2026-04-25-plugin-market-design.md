# 插件市场功能设计文档

**日期**: 2026-04-25
**作者**: Codex

## 功能概述

新增“插件市场”功能。插件的本质是执行一段逻辑，得到符合平台规范的输出，然后由系统统一推送到设备页面。

第一版重点实现插件运行框架、自定义 JS 插件、一次推送和循环推送。内置插件也纳入同一套模型，但暂不在本设计中确定具体接入哪些公共 API。后续可以从 `docs/public-apis-catalog.md` 中挑选 API，按相同输出规范封装为内置插件。

## 已确认需求

- 新增侧边栏菜单“插件市场”。
- 插件输出最终通过自由排版文本接口或图片推送接口推送到设备。
- 自定义插件使用 JS 代码。
- 自定义插件不做参数表单、API Key 配置或授权管理。
- 如果用户插件需要 API Key，用户自己直接写在 JS 代码里。
- 平台只要求用户 JS 最终返回符合插件输出规范的对象。
- 自定义插件在 Rust 后端嵌入式 JS 引擎中执行。
- 插件运行时需要提供 HTTP helper，底层走 Rust `reqwest`，不受浏览器 CORS 限制。
- 内置插件和自定义插件都要支持一次推送和循环推送。
- 循环推送每次触发都重新执行插件代码，获取最新数据后再推送。
- 本设计暂不选择第一批内置公共 API。

## 非目标

- 第一版不做插件远程下载、评分、更新或真正的在线市场分发。
- 第一版不为自定义插件提供输入 schema 或可视化参数表单。
- 第一版不为自定义插件提供 API Key 管理。
- 第一版不支持 Node.js 模块系统、`require`、`import` 或本地文件读写。
- 第一版不做富文本排版。
- 第一版不处理自定义 JS 的安全隔离策略，用户自行负责代码来源和代码内容。

## 总体架构

插件系统分为四层：

1. `PluginMarketPage`：前端插件市场页面，展示内置插件、自定义插件和插件循环任务。用户可以创建、编辑、删除、测试运行、推送一次和创建循环任务。
2. `plugin_runtime`：Rust 后端 JS 运行模块，嵌入 JS 引擎执行用户代码，并注入 HTTP helper。
3. `plugin_output`：插件输出规范和校验模块，把 JS 返回值解析为可推送的统一结果。
4. `push_pipeline`：插件推送流程。`text` 走现有自由排版文本接口；`image` 直接走图片推送。

插件数据和任务数据保存到应用数据目录：

- `custom_plugins.json`
- `plugin_loop_tasks.json`

## 插件输出规范

用户 JS 最终必须返回一个对象。第一版支持两种输出类型。

### 文本输出

```js
return {
  type: "text",
  text: "要推送的纯文本",
  fontSize: 20
};
```

`type: "text"` 走现有自由排版文本接口。`fontSize` 可选，默认值为 `20`。

### 图片输出

```js
return {
  type: "image",
  imageDataUrl: "data:image/png;base64,..."
};
```

`type: "image"` 直接解码 base64 图片，统一处理成设备需要的 `400x300 PNG` 后推送。

### 通用可选字段

```js
return {
  type: "text",
  title: "天气",
  text: "...",
  metadata: { source: "open-meteo" }
};
```

`title` 用于运行结果、任务显示和页面缓存 metadata。`metadata` 保存为 JSON 字符串，不参与推送。

### 输出校验

以下情况视为无效输出，运行失败且不推送：

- 缺少 `type`。
- `type` 不是 `text` 或 `image`。
- `text` 缺少非空 `text`。
- `image` 缺少合法 `imageDataUrl`。
- `fontSize` 超出允许范围。
- 图片无法解码。
- 返回结果超过大小限制。

## JS 运行环境

自定义插件在 Rust 后端嵌入式 JS 引擎中执行。用户代码按异步函数体处理，平台包装为：

```js
async function main() {
  // 用户写的代码
}
return await main();
```

运行环境注入两个 HTTP helper：

```js
const data = await fetchJson("https://example.com/api", {
  method: "GET",
  headers: { "Authorization": "Bearer xxx" },
  body: undefined
});

const text = await fetchText("https://example.com/text");
```

`fetchJson` 和 `fetchText` 底层使用 Rust `reqwest`。如果用户需要 API Key，直接写在 JS 代码里：

```js
const apiKey = "自己的 key";
const data = await fetchJson(`https://api.example.com/data?key=${apiKey}`);

return {
  type: "text",
  text: data.message
};
```

运行约束：

- 单次 JS 执行默认超时 20 秒。
- 单次 HTTP 请求默认超时 15 秒。
- 文本型返回结果默认最大 256 KB。
- 图片可以允许更大的 base64，但解码后统一压到 `400x300 PNG`。
- 运行失败时返回错误给前端，不推送。

## 前端页面设计

新增页面：

- `src/features/plugins/plugin-market-page.tsx`
- `src/features/plugins/plugin-market-page.test.tsx`

修改：

- `src/app/App.tsx`
- `src/components/layout/app-sidebar.tsx`
- `src/lib/tauri.ts`

侧边栏新增“插件市场”，路径 `/plugins`。

页面分区：

- 内置插件：第一版显示预留列表结构，可为空或显示暂无内置插件。
- 自定义插件：创建、编辑、删除、测试运行、推送一次、创建循环任务。
- 插件循环任务：展示、启动、停止、编辑、删除任务。

自定义插件第一版使用 `<textarea>` 编辑 JS 代码，暂不引入 Monaco 或 CodeMirror。

测试运行只执行插件并展示规范化结果，不推送。预览方式：

- `text`：显示文本内容。
- `image`：显示图片缩略预览。

推送一次会重新执行插件代码，不复用测试运行结果。

设备选择第一版沿用现有行为，默认使用第一个设备。页码支持 1 到 5，并使用 `src/components/ui/select.tsx` 中的 shadcn/ui `Select` 组件。

## 数据模型

```ts
type CustomPluginRecord = {
  id: number;
  name: string;
  description: string;
  code: string;
  createdAt: string;
  updatedAt: string;
};
```

```ts
type PluginLoopTask = {
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
```

`pluginId` 对内置插件使用 slug，对自定义插件使用自定义插件 id 的字符串形式。

## 后端模块与命令

新增模块：

- `src-tauri/src/commands/plugins.rs`
- `src-tauri/src/plugin_runtime.rs`
- `src-tauri/src/plugin_output.rs`
- `src-tauri/src/plugin_tasks.rs`

新增命令：

```rust
list_custom_plugins() -> Vec<CustomPluginRecord>
save_custom_plugin(input) -> CustomPluginRecord
delete_custom_plugin(plugin_id) -> ()

run_plugin_once(plugin_kind, plugin_id) -> PluginRunResult
push_plugin_once(plugin_kind, plugin_id, device_id, page_id) -> ()

list_plugin_loop_tasks() -> Vec<PluginLoopTask>
create_plugin_loop_task(input) -> PluginLoopTask
update_plugin_loop_task(task_id, input) -> PluginLoopTask
delete_plugin_loop_task(task_id) -> ()
start_plugin_loop_task(task_id) -> PluginLoopTask
stop_plugin_loop_task(task_id) -> PluginLoopTask
```

`BootstrapState` 增加：

- `customPlugins`
- `pluginLoopTasks`

## 推送数据流

测试运行：

1. 前端调用 `run_plugin_once`。
2. 后端读取插件代码。
3. `plugin_runtime` 执行 JS。
4. `plugin_output` 校验并归一化输出。
5. 前端展示结果，不推送。

推送一次：

1. 前端调用 `push_plugin_once`。
2. 后端重新执行插件代码。
3. 后端校验输出。
4. `text` 调用 `api::client::push_text`。
5. `image` 解码并处理成 `400x300 PNG`，再调用 `api::client::push_image`。
6. 推送成功后写入页面缓存。

循环推送：

1. 用户创建插件循环任务。
2. 用户启动任务。
3. 后端后台任务按 `intervalSeconds` 触发。
4. 每次触发都重新读取插件代码并执行。
5. 输出校验成功后推送。
6. 到达持续条件后任务进入 `completed`。
7. 某次执行或推送失败时，任务进入 `error`，记录错误信息并停止继续循环。

用户修改插件代码后，下一次循环执行使用最新代码。

## 页面缓存

插件推送成功后写入页面缓存。

建议 content type：

- `plugin_text`
- `plugin_image`

metadata 保存 JSON 字符串，包含：

- 插件名称。
- 插件类型：`builtin` 或 `custom`。
- 插件 id。
- 输出类型：`text` 或 `image`。
- 运行时间。
- 插件输出中的 `title` 和 `metadata`。

缩略图：

- `text` 保存文本前 100 个字符。
- `image` 保存图片缩略图。

## 错误处理

以下错误通过前端 toast 展示：

- 没有可用设备。
- 插件不存在。
- JS 执行超时。
- HTTP helper 请求失败。
- JS 抛出异常。
- 输出格式不符合规范。
- 图片解码失败。
- 推送接口失败。
- 循环任务启动失败。

测试运行失败不改变插件和任务状态。循环任务运行中失败时，任务状态改为 `error`，保存 `errorMessage`。

## 测试设计

前端 Vitest 覆盖：

- 插件市场菜单和 `/plugins` 页面渲染。
- 自定义插件列表展示。
- 新建、编辑、删除插件调用正确回调。
- 测试运行显示 `text` 结果。
- 推送一次传递插件类型、插件 id、设备 id 和页码。
- 创建循环任务传递间隔和持续条件。
- 启动、停止循环任务更新 UI 状态。

Rust 测试覆盖：

- 合法 `text` 输出能解析为统一结果。
- 合法 `image` data URL 能解码。
- 缺少字段、空文本、未知 type、非法图片会失败。
- `fetchJson` 和 `fetchText` 的错误会转换为用户可读错误。
- 自定义插件保存、更新、删除持久化正确。
- `push_plugin_once` 根据输出类型调用正确推送路径。
- 循环任务启动后状态变为 `running`。
- 循环任务停止后状态变为 `idle`。
- 循环任务执行失败后状态变为 `error` 并记录错误信息。

## 实施顺序建议

1. 增加数据模型、Tauri 命令和本地持久化。
2. 实现插件输出规范解析与测试。
3. 引入 JS 引擎并实现基础运行。
4. 注入 `fetchJson` 和 `fetchText`。
5. 实现一次测试运行和一次推送。
6. 实现插件市场前端页面。
7. 实现插件循环任务。
8. 补齐页面缓存和错误展示。
