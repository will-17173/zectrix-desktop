# 插件市场实施计划

**日期**: 2026-04-25
**状态**: 已按当前实现更新

## 范围

插件系统支持内置插件、自定义 JS 插件、单次推送和循环推送。插件输出只支持两种类型：

- `text`：推送纯文本，可选 `fontSize`。
- `image`：推送图片，支持 `imageDataUrl` 或 `imageUrl`。

## 后端模块

- `src-tauri/src/builtin_plugins.rs`：内置插件定义。
- `src-tauri/src/commands/plugins.rs`：插件相关 Tauri 命令。
- `src-tauri/src/plugin_output.rs`：插件输出解析和校验。
- `src-tauri/src/plugin_runtime.rs`：JS 运行环境和 helper 注入。
- `src-tauri/src/plugin_tasks.rs`：插件循环任务执行。
- `src-tauri/src/state.rs`：插件持久化、运行、推送和页面缓存。

## 输出规则

文本输出：

```js
return {
  type: "text",
  text: "要推送的内容",
  fontSize: 20,
  title: "可选标题",
  metadata: { source: "optional" },
};
```

图片输出：

```js
return {
  type: "image",
  imageUrl: "https://example.com/card.png",
  title: "可选标题",
};
```

校验规则：

- 输出必须是对象并包含 `type`。
- `type` 必须是 `text` 或 `image`。
- `text` 输出必须包含非空 `text`。
- `image` 输出必须包含合法的 `imageDataUrl` 或非空 `imageUrl`。
- `fontSize` 必须在允许范围内。

## 前端模块

- `src/features/plugins/plugin-market-page.tsx`：插件市场页面、内置插件、自定义插件和循环任务 UI。
- `src/features/plugins/plugin-market-page.test.tsx`：插件页面行为测试。
- `src/lib/tauri.ts`：插件命令的前端类型和调用封装。

自定义插件使用说明应只展示 `text` 和 `image` 输出示例。

## 当前内置插件输出

所有内置插件都应返回 `text` 或 `image`：

- 图片类插件返回 `image`。
- 文本类插件返回 `text`。

新增或修改内置插件时，需要检查返回对象里没有其他输出类型。

## 验证

推荐验证命令：

```bash
pnpm vitest run src/features/plugins/plugin-market-page.test.tsx
cargo test --manifest-path src-tauri/Cargo.toml plugin_output
cargo test --manifest-path src-tauri/Cargo.toml state::tests::run_plugin_once_rejects_text_image_output_for_custom_plugin -- --exact
```
