# 股票推送功能设计文档

**日期**: 2026-04-25
**作者**: Codex

## 功能概述

新增“股票推送”菜单。用户可以在本地维护若干个 A 股股票代码，选择目标页面后，一键实时获取行情并通过现有自由排版文本接口推送到墨水屏。

推送内容包含整屏唯一的行情获取时间，以及每只股票的代码、名称、股价、涨跌值、涨跌百分比。

## 已确认需求

- 新增侧边栏菜单“股票推送”。
- 用户只输入 6 位 A 股代码，例如 `600519`、`000001`。
- 程序自动判断沪深市场。
- 股票列表需要持久保存到本地，关闭应用后仍保留。
- 行情数据由后端直接请求公开行情接口，不要求用户配置 API Key。
- 推送复用“自由排版”的文本推送接口。
- 推送时需要选择第几页。
- 目标设备与当前“自由排版”一致，默认使用第一个设备。
- 推送文本每行包含股票代码、股票名称、股价、涨跌值、涨跌百分比。

## 用户界面设计

新增 `StockPushPage` 页面，路径为 `/stock-push`，侧边栏名称为“股票推送”。菜单位置放在“自由排版”附近，因为该功能本质上是一个自动生成自由排版文本的入口。

页面保持现有工具页布局：

- 顶部标题“股票推送”和简短说明。
- 股票代码输入框，只接受 6 位数字。
- “添加”按钮，添加前校验格式和重复项。
- 已添加股票列表，展示股票代码，并提供删除按钮。
- 页码选择器，支持第 1 到第 5 页，必须使用 `src/components/ui/select.tsx` 中的 shadcn/ui `Select` 组件。
- “推送”按钮，点击后拉取实时行情、生成文本并推送。

推送文本格式固定为：

```text
更新时间：2026-04-25 10:30:12

600519 贵州茅台 1690.00 +12.30 +0.73%
000001 平安银行 10.20 -0.05 -0.49%
```

整屏只有一个更新时间。股票名称由实时行情接口返回，不需要用户手动录入。

## 前端接入设计

新增文件：

- `src/features/stocks/stock-push-page.tsx`
- `src/features/stocks/stock-push-page.test.tsx`

修改文件：

- `src/app/App.tsx`
- `src/components/layout/app-sidebar.tsx`
- `src/lib/tauri.ts`
- `src/features/free-layout/free-layout-page.tsx`

`App.tsx` 增加 `/stock-push` 的页面标题和渲染逻辑。`AppSidebar` 增加“股票推送”菜单项，图标使用 lucide 中的行情相关图标，例如 `TrendingUp` 或 `ChartCandlestick`。

`src/lib/tauri.ts` 增加类型和命令封装：

```typescript
export type StockWatchRecord = {
  code: string;
  createdAt: string;
};

export async function addStockWatch(code: string): Promise<StockWatchRecord>;
export async function removeStockWatch(code: string): Promise<void>;
export async function listStockWatchlist(): Promise<StockWatchRecord[]>;
export async function pushStockQuotes(deviceId: string, pageId: number): Promise<void>;
```

为了页面初次打开即可显示股票列表，`BootstrapState` 增加 `stockWatchlist` 字段。添加和删除股票后，前端可以局部更新列表，不必强制重载整个应用状态。

`FreeLayoutPage` 当前使用原生 `<select>`，与仓库规则冲突。实现本功能时同步将它改为 shadcn/ui `Select`，并让股票页面复用相同的页码选择风格。

## 后端设计

新增股票相关命令模块，建议文件为：

- `src-tauri/src/commands/stocks.rs`
- `src-tauri/src/stock_quote.rs`

新增模型：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StockWatchRecord {
    pub code: String,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct StockQuote {
    pub code: String,
    pub name: String,
    pub price: String,
    pub change: String,
    pub change_percent: String,
}
```

本地股票列表保存到应用数据目录下的 `stock_watchlist.json`。保存内容只包含用户维护的股票代码和创建时间，不保存实时行情。

新增 Tauri 命令：

- `list_stock_watchlist() -> Vec<StockWatchRecord>`
- `add_stock_watch(code: String) -> StockWatchRecord`
- `remove_stock_watch(code: String) -> ()`
- `push_stock_quotes(device_id: String, page_id: u32) -> ()`

`BootstrapState` 增加 `stock_watchlist` 字段，`load_bootstrap_state` 读取该 JSON 文件并返回。

## 行情获取与格式化

后端根据 6 位代码自动推断市场：

- `6` 开头：沪市，接口前缀 `sh`
- `0` 或 `3` 开头：深市，接口前缀 `sz`
- 其他开头：不支持，添加时提示“仅支持 0、3、6 开头的 A 股代码”

行情客户端使用新浪兼容的实时行情接口：

```text
https://hq.sinajs.cn/list=sh600519,sz000001
```

请求时后端设置常见浏览器 `User-Agent` 和新浪财经 `Referer`，并按接口返回的 GBK 文本解析。普通 A 股返回字段中使用名称、当前价、昨收、当前日期和当前时间；涨跌值由当前价减昨收计算，涨跌百分比由涨跌值除以昨收计算。公开接口不可用、返回格式异常、缺少关键字段、昨收为 0 或价格无法解析时，本次推送失败并返回明确错误。

格式化逻辑集中在后端，生成最终自由排版文本：

1. 第一行是本次行情获取时间，优先使用接口返回的行情日期和时间；如果所有返回项都缺少行情日期时间，则使用本地时间格式 `YYYY-MM-DD HH:mm:ss`。
2. 第二行为空行。
3. 后续每只股票一行，顺序与本地股票列表一致。
4. 涨跌值和涨跌百分比统一由后端计算并格式化为两位小数，上涨补 `+`，下跌保留 `-`。

## 推送与页面缓存

`push_stock_quotes` 读取本地股票列表，获取行情后调用现有 `AppState::push_text` 路径，字号使用自由排版默认值 `20`，并传入用户选择的 `page_id`。

由于走现有 `push_text` 路径，推送成功后会写入页面缓存，`contentType` 为 `text`。缓存缩略图为生成文本的前 100 个字符，页面管理可以正常显示该页的文本预览。

## 错误处理

以下场景中断操作并在前端显示 toast：

- 没有可用设备。
- 股票列表为空。
- 输入不是 6 位数字。
- 输入不是 `0`、`3`、`6` 开头。
- 添加重复股票代码。
- 行情接口请求失败。
- 行情接口返回缺少名称、价格、涨跌值或涨跌百分比。
- 自由排版推送接口失败。

如果任意一只股票行情获取失败，整次推送失败，不推送不完整内容。

## 测试设计

前端 Vitest 覆盖：

- `StockPushPage` 渲染输入框、股票列表、页码选择和推送按钮。
- 非法股票代码不会调用添加接口，并显示错误。
- 合法股票代码会调用 `addStockWatch`。
- 删除按钮会调用 `removeStockWatch`。
- 推送按钮会把默认第一个设备和选中的页码传给 `pushStockQuotes`。
- `App` 显示“股票推送”菜单并能路由到新页面。
- `FreeLayoutPage` 使用共享 `Select` 后仍能选择字号和页码并推送。

后端 Rust 测试覆盖：

- 股票代码市场推断：`600519 -> sh600519`，`000001 -> sz000001`，`300750 -> sz300750`。
- 非法代码被拒绝。
- 推送文本格式包含唯一更新时间和股票名称。
- 股票列表 JSON 添加、去重、删除逻辑正确。

## 非目标

- 不做云端同步股票列表。
- 不支持港股、美股、基金、ETF 或指数。
- 不提供行情刷新定时任务。
- 不做多设备选择，保持与当前“自由排版”一致，使用第一个设备。
- 不在添加股票时请求网络校验名称，名称只在推送时从行情接口读取。
