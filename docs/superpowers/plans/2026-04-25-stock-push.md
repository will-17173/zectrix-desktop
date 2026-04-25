# 股票推送 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 新增“股票推送”菜单，维护本地 A 股代码列表，实时获取东方财富行情，并通过自由排版文本接口推送到指定页面。

**Architecture:** 后端新增股票列表存储和东方财富行情客户端，`push_stock_quotes` 复用现有 `AppState::push_text` 路径完成推送和页面缓存。前端新增 `StockPushPage`，接入启动状态、Tauri 命令封装、路由和侧边栏，并同步修正 `FreeLayoutPage` 的原生 `<select>`。

**Tech Stack:** Tauri 2, Rust, reqwest, serde_json, chrono, React 19, TypeScript, Vitest, Testing Library, shadcn/ui Select.

---

## File Structure

- Create: `src-tauri/src/commands/stocks.rs`
  - Tauri 命令入口，薄封装 `AppState` 股票方法。
- Create: `src-tauri/src/stock_quote.rs`
  - 代码校验、东方财富 `secid` 映射、行情请求、行情 JSON 解析、推送文本格式化。
- Modify: `src-tauri/src/models.rs`
  - 增加 `StockWatchRecord` 和 `BootstrapState.stock_watchlist`。
- Modify: `src-tauri/src/state.rs`
  - 增加股票列表 JSON 读写、添加、删除、推送聚合方法，以及 Rust 单元测试。
- Modify: `src-tauri/src/commands/mod.rs`
  - 导出 `stocks` 命令模块。
- Modify: `src-tauri/src/lib.rs`
  - 注册股票 Tauri 命令。
- Modify: `src/lib/tauri.ts`
  - 增加 `StockWatchRecord`、`BootstrapState.stockWatchlist` 和股票命令封装。
- Create: `src/features/stocks/stock-push-page.tsx`
  - 股票推送页面 UI 与交互。
- Create: `src/features/stocks/stock-push-page.test.tsx`
  - 股票页面行为测试。
- Modify: `src/features/free-layout/free-layout-page.tsx`
  - 将原生 `<select>` 替换为 shadcn/ui `Select`。
- Create: `src/features/free-layout/free-layout-page.test.tsx`
  - 锁定自由排版选择字号和页码后的推送行为。
- Modify: `src/app/App.tsx`
  - 接入 `/stock-push` 路由、标题、状态更新回调。
- Modify: `src/app/App.test.tsx`
  - 覆盖股票菜单和路由。
- Modify: `src/components/layout/app-sidebar.tsx`
  - 增加“股票推送”菜单项。

---

### Task 1: 后端模型和启动状态

**Files:**
- Modify: `src-tauri/src/models.rs`
- Modify: `src-tauri/src/state.rs`

- [ ] **Step 1: Write the failing Rust test for bootstrap state containing stock watchlist**

在 `src-tauri/src/state.rs` 的 `#[cfg(test)] mod tests` 中添加测试：

```rust
#[test]
fn load_bootstrap_state_includes_stock_watchlist() {
    let temp = tempfile::tempdir().unwrap();
    let state = test_state(&temp);

    save_json(
        &temp.path().join("stock_watchlist.json"),
        &vec![StockWatchRecord {
            code: "600519".to_string(),
            created_at: "2026-04-25T10:30:00Z".to_string(),
        }],
    )
    .unwrap();

    let bootstrap = state.load_bootstrap_state().unwrap();

    assert_eq!(bootstrap.stock_watchlist.len(), 1);
    assert_eq!(bootstrap.stock_watchlist[0].code, "600519");
}
```

- [ ] **Step 2: Run the Rust test to verify it fails**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml load_bootstrap_state_includes_stock_watchlist
```

Expected: FAIL because `StockWatchRecord` and `BootstrapState.stock_watchlist` do not exist.

- [ ] **Step 3: Add `StockWatchRecord` and `stock_watchlist` to models**

In `src-tauri/src/models.rs`, add after `PageCacheRecord`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StockWatchRecord {
    pub code: String,
    #[serde(alias = "created_at", rename = "createdAt")]
    pub created_at: String,
}
```

Update `BootstrapState`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapState {
    pub api_keys: Vec<ApiKeyRecord>,
    pub devices: Vec<DeviceRecord>,
    pub todos: Vec<TodoRecord>,
    pub text_templates: Vec<TextTemplateRecord>,
    pub image_templates: Vec<ImageTemplateRecord>,
    pub last_sync_time: Option<String>,
    pub page_cache: Vec<PageCacheRecord>,
    #[serde(default)]
    pub image_loop_tasks: Vec<ImageLoopTaskRecord>,
    #[serde(default)]
    pub stock_watchlist: Vec<StockWatchRecord>,
}
```

- [ ] **Step 4: Load stock watchlist in bootstrap state**

In `src-tauri/src/state.rs`, update the model import to include `StockWatchRecord`.

Add helper near page-cache helpers:

```rust
fn load_stock_watchlist(&self) -> anyhow::Result<Vec<StockWatchRecord>> {
    load_json(&self.data_dir.join("stock_watchlist.json"))
}
```

In `load_bootstrap_state`, add:

```rust
let stock_watchlist = self.load_stock_watchlist()?;
```

And set the returned field:

```rust
stock_watchlist,
```

- [ ] **Step 5: Run the targeted Rust test**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml load_bootstrap_state_includes_stock_watchlist
```

Expected: PASS.

- [ ] **Step 6: Commit**

```powershell
git add src-tauri/src/models.rs src-tauri/src/state.rs
git commit -m "feat: add stock watchlist to bootstrap state"
```

---

### Task 2: 股票代码校验和行情文本格式化

**Files:**
- Create: `src-tauri/src/stock_quote.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write failing unit tests for stock quote helpers**

Create `src-tauri/src/stock_quote.rs`:

```rust
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq)]
pub struct StockQuote {
    pub code: String,
    pub name: String,
    pub price: f64,
    pub change: f64,
    pub change_percent: f64,
}

pub fn normalize_stock_code(_code: &str) -> anyhow::Result<String> {
    unimplemented!("implemented in Step 3")
}

pub fn stock_code_to_secid(_code: &str) -> anyhow::Result<String> {
    unimplemented!("implemented in Step 3")
}

pub fn format_stock_push_text(_quotes: &[StockQuote], _now: chrono::DateTime<chrono::Local>) -> String {
    unimplemented!("implemented in Step 3")
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn maps_a_share_codes_to_eastmoney_secids() {
        assert_eq!(stock_code_to_secid("600519").unwrap(), "1.600519");
        assert_eq!(stock_code_to_secid("000001").unwrap(), "0.000001");
        assert_eq!(stock_code_to_secid("300750").unwrap(), "0.300750");
    }

    #[test]
    fn rejects_invalid_stock_codes() {
        assert!(stock_code_to_secid("688").unwrap_err().to_string().contains("6 位数字"));
        assert!(stock_code_to_secid("830000").unwrap_err().to_string().contains("仅支持"));
        assert!(stock_code_to_secid("abc001").unwrap_err().to_string().contains("6 位数字"));
    }

    #[test]
    fn formats_stock_push_text_with_one_timestamp_and_names() {
        let now = chrono::Local.with_ymd_and_hms(2026, 4, 25, 10, 30, 12).unwrap();
        let text = format_stock_push_text(
            &[
                StockQuote {
                    code: "600519".to_string(),
                    name: "贵州茅台".to_string(),
                    price: 1458.49,
                    change: 39.49,
                    change_percent: 2.78,
                },
                StockQuote {
                    code: "600000".to_string(),
                    name: "浦发银行".to_string(),
                    price: 9.45,
                    change: -0.09,
                    change_percent: -0.94,
                },
            ],
            now,
        );

        assert_eq!(
            text,
            "更新时间：2026-04-25 10:30:12\n\n600519 贵州茅台 1458.49 +39.49 +2.78%\n600000 浦发银行 9.45 -0.09 -0.94%"
        );
    }
}
```

In `src-tauri/src/lib.rs`, add:

```rust
mod stock_quote;
```

- [ ] **Step 2: Run stock quote tests to verify they fail**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml stock_quote
```

Expected: FAIL because helper functions are not implemented.

- [ ] **Step 3: Implement validation, secid mapping, and formatting**

Replace helper implementations in `src-tauri/src/stock_quote.rs`:

```rust
pub fn normalize_stock_code(code: &str) -> anyhow::Result<String> {
    let normalized = code.trim();
    if normalized.len() != 6 || !normalized.chars().all(|c| c.is_ascii_digit()) {
        anyhow::bail!("股票代码必须是 6 位数字");
    }

    let first = normalized.chars().next().unwrap();
    if first != '0' && first != '3' && first != '6' {
        anyhow::bail!("仅支持 0、3、6 开头的 A 股代码");
    }

    Ok(normalized.to_string())
}

pub fn stock_code_to_secid(code: &str) -> anyhow::Result<String> {
    let normalized = normalize_stock_code(code)?;
    let prefix = if normalized.starts_with('6') { "1" } else { "0" };
    Ok(format!("{prefix}.{normalized}"))
}

fn signed_amount(value: f64) -> String {
    if value > 0.0 {
        format!("+{value:.2}")
    } else {
        format!("{value:.2}")
    }
}

pub fn format_stock_push_text(quotes: &[StockQuote], now: chrono::DateTime<chrono::Local>) -> String {
    let mut lines = vec![
        format!("更新时间：{}", now.format("%Y-%m-%d %H:%M:%S")),
        String::new(),
    ];

    lines.extend(quotes.iter().map(|quote| {
        format!(
            "{} {} {:.2} {} {}%",
            quote.code,
            quote.name,
            quote.price,
            signed_amount(quote.change),
            signed_amount(quote.change_percent)
        )
    }));

    lines.join("\n")
}
```

- [ ] **Step 4: Run stock quote tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml stock_quote
```

Expected: PASS.

- [ ] **Step 5: Commit**

```powershell
git add src-tauri/src/stock_quote.rs src-tauri/src/lib.rs
git commit -m "feat: add stock quote formatting helpers"
```

---

### Task 3: 东方财富行情解析和请求

**Files:**
- Modify: `src-tauri/src/stock_quote.rs`

- [ ] **Step 1: Write failing parser tests**

Append tests inside `src-tauri/src/stock_quote.rs` test module:

```rust
#[test]
fn parses_eastmoney_quote_response_in_requested_order() {
    let body = r#"{
        "rc": 0,
        "data": {
            "total": 2,
            "diff": [
                {"f2":1458.49,"f3":2.78,"f4":39.49,"f12":"600519","f14":"贵州茅台"},
                {"f2":11.0,"f3":0.0,"f4":0.0,"f12":"000001","f14":"平安银行"}
            ]
        }
    }"#;

    let quotes = parse_eastmoney_quotes(body, &["000001".to_string(), "600519".to_string()]).unwrap();

    assert_eq!(quotes[0].code, "000001");
    assert_eq!(quotes[0].name, "平安银行");
    assert_eq!(quotes[1].code, "600519");
    assert_eq!(quotes[1].name, "贵州茅台");
}

#[test]
fn rejects_missing_eastmoney_quote() {
    let body = r#"{
        "rc": 0,
        "data": {
            "total": 1,
            "diff": [
                {"f2":1458.49,"f3":2.78,"f4":39.49,"f12":"600519","f14":"贵州茅台"}
            ]
        }
    }"#;

    let error = parse_eastmoney_quotes(body, &["600519".to_string(), "000001".to_string()])
        .unwrap_err()
        .to_string();

    assert!(error.contains("000001"));
}
```

- [ ] **Step 2: Run parser tests to verify they fail**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml parse_eastmoney
```

Expected: FAIL because `parse_eastmoney_quotes` does not exist.

- [ ] **Step 3: Implement Eastmoney response types and parser**

Add to `src-tauri/src/stock_quote.rs`:

```rust
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
struct EastmoneyResponse {
    data: Option<EastmoneyData>,
}

#[derive(Debug, Deserialize)]
struct EastmoneyData {
    diff: Vec<EastmoneyQuote>,
}

#[derive(Debug, Deserialize)]
struct EastmoneyQuote {
    f2: serde_json::Value,
    f3: serde_json::Value,
    f4: serde_json::Value,
    f12: String,
    f14: String,
}

fn number_field(value: &serde_json::Value, field: &str, code: &str) -> anyhow::Result<f64> {
    if let Some(n) = value.as_f64() {
        return Ok(n);
    }
    if let Some(s) = value.as_str() {
        return s
            .parse::<f64>()
            .map_err(|_| anyhow::anyhow!("股票 {code} 的 {field} 字段无法解析"));
    }
    anyhow::bail!("股票 {code} 缺少有效的 {field} 字段")
}

pub fn parse_eastmoney_quotes(body: &str, requested_codes: &[String]) -> anyhow::Result<Vec<StockQuote>> {
    let response: EastmoneyResponse = serde_json::from_str(body)?;
    let data = response
        .data
        .ok_or_else(|| anyhow::anyhow!("行情接口返回缺少 data 字段"))?;

    let mut by_code = HashMap::new();
    for item in data.diff {
        let code = item.f12;
        if item.f14.trim().is_empty() {
            anyhow::bail!("股票 {code} 缺少名称");
        }
        let quote = StockQuote {
            code: code.clone(),
            name: item.f14,
            price: number_field(&item.f2, "f2", &code)?,
            change_percent: number_field(&item.f3, "f3", &code)?,
            change: number_field(&item.f4, "f4", &code)?,
        };
        by_code.insert(code, quote);
    }

    requested_codes
        .iter()
        .map(|code| {
            by_code
                .remove(code)
                .ok_or_else(|| anyhow::anyhow!("行情接口未返回股票 {code}"))
        })
        .collect()
}
```

- [ ] **Step 4: Add async fetch function**

Add to `src-tauri/src/stock_quote.rs`:

```rust
pub async fn fetch_eastmoney_quotes(codes: &[String]) -> anyhow::Result<Vec<StockQuote>> {
    if codes.is_empty() {
        anyhow::bail!("股票列表为空");
    }

    let secids = codes
        .iter()
        .map(|code| stock_code_to_secid(code))
        .collect::<anyhow::Result<Vec<_>>>()?
        .join(",");

    let url = format!(
        "https://push2.eastmoney.com/api/qt/ulist.np/get?fltt=2&invt=2&fields=f12,f14,f2,f3,f4&secids={secids}"
    );

    let body = reqwest::Client::new()
        .get(url)
        .header("User-Agent", "Mozilla/5.0")
        .header("Referer", "https://quote.eastmoney.com/")
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    parse_eastmoney_quotes(&body, codes)
}
```

- [ ] **Step 5: Run stock quote tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml stock_quote
```

Expected: PASS.

- [ ] **Step 6: Commit**

```powershell
git add src-tauri/src/stock_quote.rs
git commit -m "feat: parse eastmoney stock quotes"
```

---

### Task 4: 后端股票列表管理和推送命令

**Files:**
- Create: `src-tauri/src/commands/stocks.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/state.rs`

- [ ] **Step 1: Write failing Rust tests for watchlist management**

Add to `src-tauri/src/state.rs` tests:

```rust
#[test]
fn adds_and_removes_stock_watch_records() {
    let temp = tempfile::tempdir().unwrap();
    let state = test_state(&temp);

    let created = state.add_stock_watch("600519").unwrap();
    assert_eq!(created.code, "600519");

    let list = state.list_stock_watchlist().unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].code, "600519");

    let duplicate = state.add_stock_watch("600519").unwrap_err().to_string();
    assert!(duplicate.contains("已存在"));

    state.remove_stock_watch("600519").unwrap();
    assert!(state.list_stock_watchlist().unwrap().is_empty());
}

#[test]
fn rejects_invalid_stock_watch_code() {
    let temp = tempfile::tempdir().unwrap();
    let state = test_state(&temp);

    let error = state.add_stock_watch("830000").unwrap_err().to_string();

    assert!(error.contains("仅支持"));
}
```

- [ ] **Step 2: Run management tests to verify they fail**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml stock_watch
```

Expected: FAIL because `add_stock_watch`, `remove_stock_watch`, and `list_stock_watchlist` do not exist.

- [ ] **Step 3: Implement watchlist persistence methods**

In `src-tauri/src/state.rs`, add methods in `impl AppState`:

```rust
pub fn list_stock_watchlist(&self) -> anyhow::Result<Vec<StockWatchRecord>> {
    self.load_stock_watchlist()
}

fn save_stock_watchlist(&self, records: &[StockWatchRecord]) -> anyhow::Result<()> {
    save_json(&self.data_dir.join("stock_watchlist.json"), records)
}

pub fn add_stock_watch(&self, code: &str) -> anyhow::Result<StockWatchRecord> {
    let normalized = crate::stock_quote::normalize_stock_code(code)?;
    let mut records = self.load_stock_watchlist()?;

    if records.iter().any(|record| record.code == normalized) {
        anyhow::bail!("股票代码 {normalized} 已存在");
    }

    let record = StockWatchRecord {
        code: normalized,
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    records.push(record.clone());
    self.save_stock_watchlist(&records)?;

    Ok(record)
}

pub fn remove_stock_watch(&self, code: &str) -> anyhow::Result<()> {
    let normalized = crate::stock_quote::normalize_stock_code(code)?;
    let mut records = self.load_stock_watchlist()?;
    let before = records.len();
    records.retain(|record| record.code != normalized);

    if records.len() == before {
        anyhow::bail!("股票代码 {normalized} 未找到");
    }

    self.save_stock_watchlist(&records)
}

pub async fn push_stock_quotes(&self, device_id: &str, page_id: u32) -> anyhow::Result<()> {
    let records = self.load_stock_watchlist()?;
    if records.is_empty() {
        anyhow::bail!("股票列表为空");
    }

    let codes = records
        .iter()
        .map(|record| record.code.clone())
        .collect::<Vec<_>>();
    let quotes = crate::stock_quote::fetch_eastmoney_quotes(&codes).await?;
    let text = crate::stock_quote::format_stock_push_text(&quotes, chrono::Local::now());

    self.push_text(&text, Some(20), device_id, Some(page_id)).await
}
```

- [ ] **Step 4: Add Tauri command module**

Create `src-tauri/src/commands/stocks.rs`:

```rust
#[tauri::command]
pub fn list_stock_watchlist(
    state: tauri::State<'_, crate::state::AppState>,
) -> Result<Vec<crate::models::StockWatchRecord>, String> {
    state.list_stock_watchlist().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_stock_watch(
    state: tauri::State<'_, crate::state::AppState>,
    code: String,
) -> Result<crate::models::StockWatchRecord, String> {
    state.add_stock_watch(&code).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_stock_watch(
    state: tauri::State<'_, crate::state::AppState>,
    code: String,
) -> Result<(), String> {
    state.remove_stock_watch(&code).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn push_stock_quotes(
    state: tauri::State<'_, crate::state::AppState>,
    device_id: String,
    page_id: u32,
) -> Result<(), String> {
    state
        .push_stock_quotes(&device_id, page_id)
        .await
        .map_err(|e| e.to_string())
}
```

In `src-tauri/src/commands/mod.rs`, add:

```rust
pub mod stocks;
```

In `src-tauri/src/lib.rs`, add these to `tauri::generate_handler!`:

```rust
commands::stocks::list_stock_watchlist,
commands::stocks::add_stock_watch,
commands::stocks::remove_stock_watch,
commands::stocks::push_stock_quotes,
```

- [ ] **Step 5: Run targeted Rust tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml stock_watch
```

Expected: PASS.

- [ ] **Step 6: Commit**

```powershell
git add src-tauri/src/state.rs src-tauri/src/commands/stocks.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs
git commit -m "feat: add stock watchlist commands"
```

---

### Task 5: TypeScript Tauri bindings

**Files:**
- Modify: `src/lib/tauri.ts`

- [ ] **Step 1: Add TypeScript stock types and bootstrap field**

In `src/lib/tauri.ts`, add:

```typescript
export type StockWatchRecord = {
  code: string;
  createdAt: string;
};
```

Update `BootstrapState`:

```typescript
export type BootstrapState = {
  apiKeys: ApiKeyRecord[];
  devices: DeviceRecord[];
  todos: Array<TodoRecord>;
  textTemplates: Array<TextTemplateRecord>;
  imageTemplates: Array<ImageTemplateRecord>;
  lastSyncTime: string | null;
  pageCache: Array<PageCacheRecord>;
  imageLoopTasks: Array<ImageLoopTask>;
  stockWatchlist: Array<StockWatchRecord>;
};
```

- [ ] **Step 2: Add stock command wrappers**

In `src/lib/tauri.ts`, add near other command wrappers:

```typescript
export async function listStockWatchlist(): Promise<StockWatchRecord[]> {
  return invoke<StockWatchRecord[]>("list_stock_watchlist");
}

export async function addStockWatch(code: string): Promise<StockWatchRecord> {
  return invoke<StockWatchRecord>("add_stock_watch", { code });
}

export async function removeStockWatch(code: string): Promise<void> {
  return invoke("remove_stock_watch", { code });
}

export async function pushStockQuotes(deviceId: string, pageId: number): Promise<void> {
  return invoke("push_stock_quotes", { deviceId, pageId });
}
```

- [ ] **Step 3: Run frontend type check**

Run:

```powershell
pnpm build
```

Expected: FAIL because `emptyState` and test fixtures do not include `stockWatchlist`. The specific acceptable failure is missing `stockWatchlist` on objects typed as `BootstrapState`.

- [ ] **Step 4: Hold commit until Task 7 compiles**

Do not commit this task alone if `pnpm build` fails. Commit it together with Task 7 once app state callers are updated.

---

### Task 6: StockPushPage UI

**Files:**
- Create: `src/features/stocks/stock-push-page.tsx`
- Create: `src/features/stocks/stock-push-page.test.tsx`

- [ ] **Step 1: Write failing StockPushPage tests**

Create `src/features/stocks/stock-push-page.test.tsx`:

```tsx
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { Toaster } from "../../components/ui/toast";
import { StockPushPage } from "./stock-push-page";

const devices = [{ deviceId: "AA:BB", alias: "桌面屏", board: "note" }];
const watchlist = [{ code: "600519", createdAt: "2026-04-25T10:30:00Z" }];

test("renders stock controls and existing watchlist", () => {
  render(
    <StockPushPage
      devices={devices}
      watchlist={watchlist}
      onAddStock={vi.fn()}
      onRemoveStock={vi.fn()}
      onPushStocks={vi.fn()}
    />
  );

  expect(screen.getByRole("heading", { name: "股票推送" })).toBeInTheDocument();
  expect(screen.getByLabelText("股票代码")).toBeInTheDocument();
  expect(screen.getByText("600519")).toBeInTheDocument();
  expect(screen.getByRole("button", { name: "推送" })).toBeInTheDocument();
});

test("rejects invalid stock code before calling add", async () => {
  const user = userEvent.setup();
  const onAddStock = vi.fn();
  render(
    <>
      <Toaster position="top-center" />
      <StockPushPage
        devices={devices}
        watchlist={[]}
        onAddStock={onAddStock}
        onRemoveStock={vi.fn()}
        onPushStocks={vi.fn()}
      />
    </>,
  );

  await user.type(screen.getByLabelText("股票代码"), "830000");
  await user.click(screen.getByRole("button", { name: "添加" }));

  expect(onAddStock).not.toHaveBeenCalled();
  expect(await screen.findByText("仅支持 0、3、6 开头的 A 股代码")).toBeInTheDocument();
});

test("adds and removes stocks", async () => {
  const user = userEvent.setup();
  const onAddStock = vi.fn().mockResolvedValue({ code: "000001", createdAt: "2026-04-25T10:31:00Z" });
  const onRemoveStock = vi.fn().mockResolvedValue(undefined);
  render(
    <StockPushPage
      devices={devices}
      watchlist={watchlist}
      onAddStock={onAddStock}
      onRemoveStock={onRemoveStock}
      onPushStocks={vi.fn()}
    />
  );

  await user.type(screen.getByLabelText("股票代码"), "000001");
  await user.click(screen.getByRole("button", { name: "添加" }));
  await user.click(screen.getByRole("button", { name: "删除 600519" }));

  expect(onAddStock).toHaveBeenCalledWith("000001");
  expect(onRemoveStock).toHaveBeenCalledWith("600519");
});

test("pushes stocks to first device and selected page", async () => {
  const user = userEvent.setup();
  const onPushStocks = vi.fn().mockResolvedValue(undefined);
  render(
    <StockPushPage
      devices={devices}
      watchlist={watchlist}
      onAddStock={vi.fn()}
      onRemoveStock={vi.fn()}
      onPushStocks={onPushStocks}
    />
  );

  await user.click(screen.getByRole("combobox"));
  await user.click(await screen.findByRole("option", { name: "第 3 页" }));
  await user.click(screen.getByRole("button", { name: "推送" }));

  expect(onPushStocks).toHaveBeenCalledWith("AA:BB", 3);
});
```

- [ ] **Step 2: Run StockPushPage tests to verify they fail**

Run:

```powershell
pnpm vitest run src/features/stocks/stock-push-page.test.tsx
```

Expected: FAIL because `stock-push-page.tsx` does not exist.

- [ ] **Step 3: Implement StockPushPage**

Create `src/features/stocks/stock-push-page.tsx`:

```tsx
import { useEffect, useState } from "react";
import { toast } from "../../components/ui/toast";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "../../components/ui/select";
import type { StockWatchRecord } from "../../lib/tauri";

type Device = { deviceId: string; alias: string; board: string };

const PAGE_OPTIONS = [
  { value: 1, label: "第 1 页" },
  { value: 2, label: "第 2 页" },
  { value: 3, label: "第 3 页" },
  { value: 4, label: "第 4 页" },
  { value: 5, label: "第 5 页" },
];

type Props = {
  devices: Device[];
  watchlist: StockWatchRecord[];
  onAddStock: (code: string) => Promise<StockWatchRecord>;
  onRemoveStock: (code: string) => Promise<void>;
  onPushStocks: (deviceId: string, pageId: number) => Promise<void>;
};

function validateCode(code: string): string | null {
  if (!/^\d{6}$/.test(code)) return "股票代码必须是 6 位数字";
  if (!/^[036]/.test(code)) return "仅支持 0、3、6 开头的 A 股代码";
  return null;
}

export function StockPushPage({ devices, watchlist, onAddStock, onRemoveStock, onPushStocks }: Props) {
  const [stocks, setStocks] = useState(watchlist);
  const [code, setCode] = useState("");
  const [pageId, setPageId] = useState(1);
  const [adding, setAdding] = useState(false);
  const [removingCode, setRemovingCode] = useState<string | null>(null);
  const [pushing, setPushing] = useState(false);

  useEffect(() => {
    setStocks(watchlist);
  }, [watchlist]);

  async function handleAdd(e: React.FormEvent) {
    e.preventDefault();
    const normalized = code.trim();
    const error = validateCode(normalized);
    if (error) {
      toast.error(error);
      return;
    }
    if (stocks.some((stock) => stock.code === normalized)) {
      toast.error(`股票代码 ${normalized} 已存在`);
      return;
    }

    setAdding(true);
    try {
      const record = await onAddStock(normalized);
      setStocks((prev) => [...prev, record]);
      setCode("");
      toast.success("股票已添加");
    } catch (e) {
      toast.error(`添加失败: ${e instanceof Error ? e.message : String(e)}`);
    } finally {
      setAdding(false);
    }
  }

  async function handleRemove(stockCode: string) {
    setRemovingCode(stockCode);
    try {
      await onRemoveStock(stockCode);
      setStocks((prev) => prev.filter((stock) => stock.code !== stockCode));
      toast.success("股票已删除");
    } catch (e) {
      toast.error(`删除失败: ${e instanceof Error ? e.message : String(e)}`);
    } finally {
      setRemovingCode(null);
    }
  }

  async function handlePush() {
    const deviceId = devices[0]?.deviceId;
    if (!deviceId) {
      toast.error("没有可用设备");
      return;
    }
    if (stocks.length === 0) {
      toast.error("请先添加股票代码");
      return;
    }

    setPushing(true);
    try {
      await onPushStocks(deviceId, pageId);
      toast.success(`推送成功，已发送到第 ${pageId} 页`);
    } catch (e) {
      toast.error(`推送失败: ${e instanceof Error ? e.message : String(e)}`);
    } finally {
      setPushing(false);
    }
  }

  return (
    <section className="space-y-6">
      <div className="flex items-center justify-between gap-3">
        <div>
          <h2 className="text-lg font-semibold">股票推送</h2>
          <p className="text-sm text-gray-500">维护 A 股代码，实时获取行情后推送到设备页面。</p>
        </div>
      </div>

      <form onSubmit={handleAdd} className="space-y-4 max-w-md p-4 rounded-xl border border-gray-200 bg-white/85 shadow-sm">
        <div className="space-y-2">
          <label htmlFor="stock-code" className="block text-sm font-medium">股票代码</label>
          <div className="flex gap-2">
            <input
              id="stock-code"
              value={code}
              onChange={(e) => setCode(e.target.value)}
              placeholder="例如 600519"
              className="min-w-0 flex-1 px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700"
            />
            <button
              type="submit"
              disabled={adding || !code.trim()}
              className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:cursor-not-allowed disabled:bg-blue-300"
            >
              {adding ? "添加中..." : "添加"}
            </button>
          </div>
        </div>

        <div className="space-y-2">
          <div className="text-sm font-medium">已添加股票</div>
          {stocks.length === 0 ? (
            <p className="text-sm text-gray-500">暂无股票代码</p>
          ) : (
            <ul className="divide-y divide-gray-100 rounded-md border border-gray-200 bg-white">
              {stocks.map((stock) => (
                <li key={stock.code} className="flex items-center justify-between gap-3 px-3 py-2">
                  <span className="font-mono text-sm">{stock.code}</span>
                  <button
                    type="button"
                    aria-label={`删除 ${stock.code}`}
                    onClick={() => handleRemove(stock.code)}
                    disabled={removingCode === stock.code}
                    className="px-2 py-1 text-sm text-red-600 hover:text-red-700 disabled:text-gray-400"
                  >
                    {removingCode === stock.code ? "删除中..." : "删除"}
                  </button>
                </li>
              ))}
            </ul>
          )}
        </div>

        <div className="space-y-2">
          <label htmlFor="stock-page-id" className="block text-sm font-medium">目标页面</label>
          <Select value={String(pageId)} onValueChange={(v) => setPageId(Number(v))}>
            <SelectTrigger id="stock-page-id">
              <SelectValue placeholder="选择页面" />
            </SelectTrigger>
            <SelectContent>
              {PAGE_OPTIONS.map((opt) => (
                <SelectItem key={opt.value} value={String(opt.value)}>
                  {opt.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        <button
          type="button"
          onClick={handlePush}
          disabled={pushing || stocks.length === 0}
          className="w-full px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:cursor-not-allowed disabled:bg-blue-300"
        >
          {pushing ? "推送中..." : "推送"}
        </button>
      </form>

      {devices.length > 0 && (
        <div className="text-sm text-gray-500">
          推送到设备: {devices[0].alias || devices[0].deviceId}
        </div>
      )}
    </section>
  );
}
```

- [ ] **Step 4: Run StockPushPage tests**

Run:

```powershell
pnpm vitest run src/features/stocks/stock-push-page.test.tsx
```

Expected: PASS.

- [ ] **Step 5: Commit**

```powershell
git add src/features/stocks/stock-push-page.tsx src/features/stocks/stock-push-page.test.tsx
git commit -m "feat: add stock push page"
```

---

### Task 7: App route, sidebar, and bootstrap wiring

**Files:**
- Modify: `src/app/App.tsx`
- Modify: `src/app/App.test.tsx`
- Modify: `src/components/layout/app-sidebar.tsx`
- Modify: `src/lib/tauri.ts`

- [ ] **Step 1: Update App tests for stock route**

In `src/app/App.test.tsx`, update every `BootstrapState` fixture to include:

```typescript
stockWatchlist: [],
```

Update the `vi.mock("../lib/tauri", ...)` object to include the new command mocks:

```typescript
addStockWatch: vi.fn(),
removeStockWatch: vi.fn(),
pushStockQuotes: vi.fn(),
```

Add a route test:

```tsx
test("renders the stock push route from the sidebar", async () => {
  render(
    <MemoryRouter>
      <App />
    </MemoryRouter>,
  );

  await screen.findByText("股票推送");
  await userEvent.click(screen.getByRole("link", { name: "股票推送" }));

  expect(screen.getByRole("heading", { name: "股票推送" })).toBeInTheDocument();
});
```

- [ ] **Step 2: Run App tests to verify they fail**

Run:

```powershell
pnpm vitest run src/app/App.test.tsx
```

Expected: FAIL because route and sidebar item do not exist.

- [ ] **Step 3: Wire stock page in App**

In `src/app/App.tsx`, import:

```typescript
import { StockPushPage } from "../features/stocks/stock-push-page";
```

Import Tauri wrappers:

```typescript
addStockWatch,
removeStockWatch,
pushStockQuotes,
```

Update `emptyState`:

```typescript
stockWatchlist: [],
```

Update `sectionTitles`:

```typescript
"/stock-push": "股票推送",
```

Add route branch before `/free-layout` or near it:

```tsx
if (path === "/stock-push") {
  return (
    <StockPushPage
      devices={state.devices}
      watchlist={state.stockWatchlist}
      onAddStock={async (code) => {
        const record = await addStockWatch(code);
        setState((prev) => ({
          ...prev,
          stockWatchlist: [...prev.stockWatchlist, record],
        }));
        return record;
      }}
      onRemoveStock={async (code) => {
        await removeStockWatch(code);
        setState((prev) => ({
          ...prev,
          stockWatchlist: prev.stockWatchlist.filter((stock) => stock.code !== code),
        }));
      }}
      onPushStocks={pushStockQuotes}
    />
  );
}
```

- [ ] **Step 4: Add sidebar item**

In `src/components/layout/app-sidebar.tsx`, import `TrendingUp`:

```typescript
import { CheckSquare, FileText, Image, Settings, PenTool, LayoutGrid, Layers, TrendingUp } from "lucide-react";
```

Add menu item near “自由排版”:

```typescript
{ label: "股票推送", icon: TrendingUp, href: "/stock-push" },
```

- [ ] **Step 5: Run App tests and build**

Run:

```powershell
pnpm vitest run src/app/App.test.tsx
pnpm build
```

Expected: PASS.

- [ ] **Step 6: Commit**

```powershell
git add src/app/App.tsx src/app/App.test.tsx src/components/layout/app-sidebar.tsx src/lib/tauri.ts
git commit -m "feat: wire stock push route"
```

---

### Task 8: FreeLayoutPage shadcn Select cleanup

**Files:**
- Modify: `src/features/free-layout/free-layout-page.tsx`
- Create: `src/features/free-layout/free-layout-page.test.tsx`

- [ ] **Step 1: Write FreeLayoutPage behavior test**

Create `src/features/free-layout/free-layout-page.test.tsx`:

```tsx
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { FreeLayoutPage } from "./free-layout-page";

test("pushes free layout text with selected font size and page", async () => {
  const user = userEvent.setup();
  const onPushText = vi.fn().mockResolvedValue(undefined);
  render(
    <FreeLayoutPage
      devices={[{ deviceId: "AA:BB", alias: "桌面屏", board: "note" }]}
      onPushText={onPushText}
    />
  );

  await user.type(screen.getByLabelText("文本内容"), "行情内容");

  const selects = screen.getAllByRole("combobox");
  await user.click(selects[0]);
  await user.click(await screen.findByRole("option", { name: "24px" }));
  await user.click(selects[1]);
  await user.click(await screen.findByRole("option", { name: "第 4 页" }));

  await user.click(screen.getByRole("button", { name: "推送" }));

  expect(onPushText).toHaveBeenCalledWith("行情内容", 24, "AA:BB", 4);
});
```

- [ ] **Step 2: Run the FreeLayoutPage test**

Run:

```powershell
pnpm vitest run src/features/free-layout/free-layout-page.test.tsx
```

Expected: It may pass with native `<select>`, but the source still violates the repository rule. Continue to Step 3.

- [ ] **Step 3: Replace native selects with shadcn Select**

In `src/features/free-layout/free-layout-page.tsx`, import:

```typescript
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "../../components/ui/select";
```

Replace the font-size `<select>` block with:

```tsx
<Select value={String(fontSize)} onValueChange={(v) => setFontSize(Number(v))}>
  <SelectTrigger id="font-size">
    <SelectValue placeholder="选择字号" />
  </SelectTrigger>
  <SelectContent>
    {FONT_SIZE_OPTIONS.map((opt) => (
      <SelectItem key={opt.value} value={String(opt.value)}>
        {opt.label}
      </SelectItem>
    ))}
  </SelectContent>
</Select>
```

Replace the page `<select>` block with:

```tsx
<Select value={String(pageId)} onValueChange={(v) => setPageId(Number(v))}>
  <SelectTrigger id="page-id">
    <SelectValue placeholder="选择页面" />
  </SelectTrigger>
  <SelectContent>
    {PAGE_OPTIONS.map((opt) => (
      <SelectItem key={opt.value} value={String(opt.value)}>
        {opt.label}
      </SelectItem>
    ))}
  </SelectContent>
</Select>
```

- [ ] **Step 4: Verify no native select remains in FreeLayoutPage**

Run:

```powershell
rg -n "<select|</select|<option" src/features/free-layout/free-layout-page.tsx
```

Expected: no output.

- [ ] **Step 5: Run FreeLayoutPage test**

Run:

```powershell
pnpm vitest run src/features/free-layout/free-layout-page.test.tsx
```

Expected: PASS.

- [ ] **Step 6: Commit**

```powershell
git add src/features/free-layout/free-layout-page.tsx src/features/free-layout/free-layout-page.test.tsx
git commit -m "fix: use shared select in free layout"
```

---

### Task 9: End-to-end verification

**Files:**
- No source edits expected unless verification exposes a defect.

- [ ] **Step 1: Run frontend tests**

Run:

```powershell
pnpm vitest run
```

Expected: PASS.

- [ ] **Step 2: Run frontend build**

Run:

```powershell
pnpm build
```

Expected: PASS.

- [ ] **Step 3: Run Rust tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected: PASS.

- [ ] **Step 4: Run a live Eastmoney smoke check**

Run:

```powershell
Invoke-WebRequest -UseBasicParsing -Uri "https://push2.eastmoney.com/api/qt/ulist.np/get?fltt=2&invt=2&fields=f12,f14,f2,f3,f4&secids=1.600519,0.000001" -Headers @{"User-Agent"="Mozilla/5.0";"Referer"="https://quote.eastmoney.com/"} -TimeoutSec 10
```

Expected: response content contains `"f12":"600519"` and `"f14":"贵州茅台"`.

- [ ] **Step 5: Inspect git status**

Run:

```powershell
git status --short --branch
```

Expected: branch is ahead by implementation commits and worktree is clean.
