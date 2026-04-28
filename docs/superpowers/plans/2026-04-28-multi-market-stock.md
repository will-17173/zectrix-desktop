# 股票多市场支持（A股/港股/美股）Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 在现有股票推送功能基础上新增港股和美股支持，用户输入股票代码时自动识别市场，推送文本中展示市场标签。

**Architecture:** `stock_quote.rs` 新增 `StockCode` 结构体和 `parse_stock_input()` 替换原有的 `normalize_stock_code()`，腾讯行情接口加 `hk`/`us` 前缀支持三市场。`StockWatchRecord` 新增 `market` 字段，向后兼容旧数据（默认 `"a"`）。前端输入框移除 6 位数字限制，添加市场 badge。

**Tech Stack:** Rust（`stock_quote.rs`、`models.rs`、`state.rs`），TypeScript/React（`tauri.ts`、`stock-push-page.tsx`）。

---

## 文件变更总览

| 操作 | 路径 |
|------|------|
| 修改 | `src-tauri/src/stock_quote.rs` |
| 修改 | `src-tauri/src/models.rs` |
| 修改 | `src-tauri/src/state.rs` |
| 修改 | `src/lib/tauri.ts` |
| 修改 | `src/features/stocks/stock-push-page.tsx` |

---

### Task 1: 扩展 stock_quote.rs — 新增 StockCode 和市场识别

**Files:**
- Modify: `src-tauri/src/stock_quote.rs`

`StockCode` 结构体和 `parse_stock_input()` 替换 `normalize_stock_code()`，同时扩展 `stock_code_to_tencent_symbol()` 支持三市场。

- [ ] **Step 1: 写新的识别逻辑测试**

在 `src-tauri/src/stock_quote.rs` 的 `#[cfg(test)]` 模块末尾追加：

```rust
#[test]
fn parse_stock_input_identifies_a_share() {
    let code = parse_stock_input("600519").unwrap();
    assert_eq!(code.code, "600519");
    assert_eq!(code.market, "a");
}

#[test]
fn parse_stock_input_identifies_a_share_with_whitespace() {
    let code = parse_stock_input(" 000001 ").unwrap();
    assert_eq!(code.code, "000001");
    assert_eq!(code.market, "a");
}

#[test]
fn parse_stock_input_identifies_hk_stock_pads_to_5_digits() {
    let code = parse_stock_input("700").unwrap();
    assert_eq!(code.code, "00700");
    assert_eq!(code.market, "hk");

    let code2 = parse_stock_input("9988").unwrap();
    assert_eq!(code2.code, "09988");
    assert_eq!(code2.market, "hk");

    let code3 = parse_stock_input("00700").unwrap();
    assert_eq!(code3.code, "00700");
    assert_eq!(code3.market, "hk");
}

#[test]
fn parse_stock_input_identifies_us_stock() {
    let code = parse_stock_input("AAPL").unwrap();
    assert_eq!(code.code, "AAPL");
    assert_eq!(code.market, "us");

    let code2 = parse_stock_input("brk.b").unwrap();
    assert_eq!(code2.code, "BRK.B");
    assert_eq!(code2.market, "us");

    let code3 = parse_stock_input("BABA").unwrap();
    assert_eq!(code3.code, "BABA");
    assert_eq!(code3.market, "us");
}

#[test]
fn parse_stock_input_rejects_invalid_codes() {
    assert!(parse_stock_input("").is_err());
    assert!(parse_stock_input("1234567").is_err()); // 7位纯数字
}

#[test]
fn stock_code_to_tencent_symbol_supports_hk_and_us() {
    let sc_hk = StockCode { code: "00700".to_string(), market: "hk".to_string() };
    assert_eq!(stock_code_to_tencent_symbol_v2(&sc_hk).unwrap(), "hk00700");

    let sc_us = StockCode { code: "AAPL".to_string(), market: "us".to_string() };
    assert_eq!(stock_code_to_tencent_symbol_v2(&sc_us).unwrap(), "usAAPL");

    let sc_a_sh = StockCode { code: "600519".to_string(), market: "a".to_string() };
    assert_eq!(stock_code_to_tencent_symbol_v2(&sc_a_sh).unwrap(), "sh600519");

    let sc_a_sz = StockCode { code: "000001".to_string(), market: "a".to_string() };
    assert_eq!(stock_code_to_tencent_symbol_v2(&sc_a_sz).unwrap(), "sz000001");
}
```

- [ ] **Step 2: 运行测试确认失败**

```bash
cargo test --manifest-path src-tauri/Cargo.toml parse_stock_input 2>&1
cargo test --manifest-path src-tauri/Cargo.toml stock_code_to_tencent_symbol_supports 2>&1
```

期望：编译报错（`parse_stock_input` 和 `stock_code_to_tencent_symbol_v2` 未定义）。

- [ ] **Step 3: 在 stock_quote.rs 中新增 StockCode 结构体和 parse_stock_input()**

在文件顶部 `use` 语句之后，`StockQuote` 定义之前插入：

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct StockCode {
    pub code: String,
    pub market: String, // "a" | "hk" | "us"
}

pub fn parse_stock_input(input: &str) -> anyhow::Result<StockCode> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        anyhow::bail!("股票代码不能为空");
    }

    let all_digits = trimmed.chars().all(|c| c.is_ascii_digit());
    let has_letter = trimmed.chars().any(|c| c.is_ascii_alphabetic());

    if all_digits {
        match trimmed.len() {
            6 => Ok(StockCode { code: trimmed.to_string(), market: "a".to_string() }),
            1..=5 => {
                let padded = format!("{:0>5}", trimmed);
                Ok(StockCode { code: padded, market: "hk".to_string() })
            }
            _ => anyhow::bail!("无法识别的股票代码格式：{trimmed}"),
        }
    } else if has_letter {
        let upper = trimmed.to_uppercase();
        Ok(StockCode { code: upper, market: "us".to_string() })
    } else {
        anyhow::bail!("无法识别的股票代码格式：{trimmed}")
    }
}
```

- [ ] **Step 4: 新增 stock_code_to_tencent_symbol_v2()**

在现有 `stock_code_to_tencent_symbol()` 函数之后插入：

```rust
pub fn stock_code_to_tencent_symbol_v2(sc: &StockCode) -> anyhow::Result<String> {
    match sc.market.as_str() {
        "hk" => Ok(format!("hk{}", sc.code)),
        "us" => Ok(format!("us{}", sc.code)),
        _ => {
            // A 股复用原有逻辑
            let code = &sc.code;
            let market = if code.starts_with('6') {
                "sh"
            } else if code.starts_with('0') || code.starts_with('3') {
                "sz"
            } else if code.starts_with('4') || code.starts_with('8') {
                "bj"
            } else {
                "sz"
            };
            Ok(format!("{market}{code}"))
        }
    }
}
```

- [ ] **Step 5: 运行测试确认通过**

```bash
cargo test --manifest-path src-tauri/Cargo.toml parse_stock_input 2>&1
cargo test --manifest-path src-tauri/Cargo.toml stock_code_to_tencent_symbol_supports 2>&1
```

期望：所有新测试 PASS。

- [ ] **Step 6: 提交**

```bash
git add src-tauri/src/stock_quote.rs
git commit -m "feat: add StockCode struct and parse_stock_input for multi-market support"
```

---

### Task 2: 扩展 fetch_tencent_quotes 支持多市场

**Files:**
- Modify: `src-tauri/src/stock_quote.rs`

`fetch_tencent_quotes` 改为接收 `&[StockCode]`，`parse_tencent_quotes` 改为按 `StockCode` 匹配。腾讯行情返回的 code 字段（fields[2]）对于港股和美股格式不同，需要用 symbol → StockCode 的映射来还原。

- [ ] **Step 1: 写多市场 fetch 的测试**

在 `#[cfg(test)]` 模块追加：

```rust
#[test]
fn parses_tencent_hk_quote() {
    // 腾讯港股行情格式：fields[2] 是去掉前缀的代码（如 "00700"），fields[1] 是名称
    let body = r#"v_hk00700="1~腾讯控股~00700~380.00~382.00~380.00~123456~50000~73456~379.80~1000~379.60~2000~379.40~3000~379.20~4000~379.00~5000~380.20~800~380.40~1200~380.60~1600~380.80~2000~381.00~2500~~20260428160000~-2.00~-0.52~385.00~378.00~380.00/123456/4689480000~123456~468948~0.20~~";"#;
    let sc = StockCode { code: "00700".to_string(), market: "hk".to_string() };
    let quotes = parse_tencent_quotes_v2(body, &[sc]).unwrap();
    assert_eq!(quotes[0].code, "00700");
    assert_eq!(quotes[0].name, "腾讯控股");
    assert_eq!(quotes[0].price, 380.00);
    assert!(quotes[0].valid);
}

#[test]
fn parses_tencent_us_quote() {
    let body = r#"v_usAAPL="1~苹果公司~AAPL~189.30~188.10~189.30~4567890~2000000~2567890~189.20~5000~189.10~8000~189.00~10000~188.90~15000~188.80~20000~189.40~3000~189.50~5000~189.60~8000~189.70~10000~189.80~12000~~20260428160000~1.20~0.64~190.50~187.80~189.30/4567890/864045270~4567890~86404~0.20~~";"#;
    let sc = StockCode { code: "AAPL".to_string(), market: "us".to_string() };
    let quotes = parse_tencent_quotes_v2(body, &[sc]).unwrap();
    assert_eq!(quotes[0].code, "AAPL");
    assert_eq!(quotes[0].name, "苹果公司");
    assert_eq!(quotes[0].price, 189.30);
    assert!(quotes[0].valid);
}
```

- [ ] **Step 2: 运行测试确认失败**

```bash
cargo test --manifest-path src-tauri/Cargo.toml parses_tencent_hk_quote 2>&1
cargo test --manifest-path src-tauri/Cargo.toml parses_tencent_us_quote 2>&1
```

期望：编译报错（`parse_tencent_quotes_v2` 未定义）。

- [ ] **Step 3: 实现 parse_tencent_quotes_v2**

在现有 `parse_tencent_quotes` 之后新增：

```rust
pub fn parse_tencent_quotes_v2(
    body: &str,
    requested: &[StockCode],
) -> anyhow::Result<Vec<StockQuote>> {
    // 建立 symbol → StockCode 的映射，用于解析后还原
    let mut symbol_map: HashMap<String, &StockCode> = HashMap::new();
    for sc in requested {
        if let Ok(sym) = stock_code_to_tencent_symbol_v2(sc) {
            symbol_map.insert(sym, sc);
        }
    }

    // 同时建立 code → StockQuote 的映射（key 是 StockCode.code）
    let mut by_code: HashMap<String, StockQuote> = HashMap::new();

    for line in body.lines() {
        let Some((name_part, value_part)) = line.split_once("=\"") else {
            continue;
        };
        let raw_value = value_part.trim().trim_end_matches(';').trim_end_matches('"');
        let fields: Vec<&str> = raw_value.split('~').collect();

        // 从变量名（name_part）匹配 symbol，如 v_hk00700 → hk00700
        let symbol = name_part.trim_start_matches("v_");
        let Some(sc) = symbol_map.get(symbol) else {
            continue;
        };

        if fields.len() < 33 {
            by_code.insert(sc.code.clone(), unavailable_stock_quote(&sc.code, None));
            continue;
        }

        let name = fields[1].trim();
        let price = fields[3].trim().parse::<f64>().unwrap_or(0.0);
        let change = fields[31].trim().parse::<f64>().unwrap_or(0.0);
        let change_percent = fields[32].trim().parse::<f64>().unwrap_or(0.0);
        let status = fields.get(40).copied().unwrap_or_default();
        let is_valid = !name.is_empty() && price > 0.0 && status != "D";

        by_code.insert(
            sc.code.clone(),
            StockQuote {
                code: sc.code.clone(),
                name: if name.is_empty() { "未知".to_string() } else { name.to_string() },
                price: if is_valid { price } else { 0.0 },
                change: if is_valid { change } else { 0.0 },
                change_percent: if is_valid { change_percent } else { 0.0 },
                valid: is_valid,
            },
        );
    }

    Ok(requested
        .iter()
        .map(|sc| {
            by_code
                .remove(&sc.code)
                .unwrap_or_else(|| unavailable_stock_quote(&sc.code, None))
        })
        .collect())
}
```

- [ ] **Step 4: 实现 fetch_tencent_quotes_v2**

在 `fetch_tencent_quotes` 之后新增：

```rust
async fn fetch_tencent_quotes_v2(stock_codes: &[StockCode]) -> anyhow::Result<Vec<StockQuote>> {
    if stock_codes.is_empty() {
        anyhow::bail!("股票列表为空");
    }

    let symbols: Vec<String> = stock_codes
        .iter()
        .filter_map(|sc| stock_code_to_tencent_symbol_v2(sc).ok())
        .collect();

    if symbols.is_empty() {
        return Ok(stock_codes
            .iter()
            .map(|sc| unavailable_stock_quote(&sc.code, Some("代码无效")))
            .collect());
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .connect_timeout(std::time::Duration::from_secs(5))
        .user_agent("Mozilla/5.0")
        .build()?;
    let url = format!("https://qt.gtimg.cn/q={}", symbols.join(","));
    let response = client.get(&url).send().await?;
    if !response.status().is_success() {
        anyhow::bail!("腾讯行情接口请求失败: {}", response.status());
    }

    let bytes = response.bytes().await?;
    let (body, _, _) = encoding_rs::GBK.decode(&bytes);
    parse_tencent_quotes_v2(&body, stock_codes)
}
```

- [ ] **Step 5: 更新 fetch_stock_quotes 公开 API 签名**

将现有的：

```rust
pub async fn fetch_stock_quotes(codes: &[String]) -> anyhow::Result<Vec<StockQuote>> {
    fetch_tencent_quotes(codes).await
}
```

替换为：

```rust
pub async fn fetch_stock_quotes(stock_codes: &[StockCode]) -> anyhow::Result<Vec<StockQuote>> {
    fetch_tencent_quotes_v2(stock_codes).await
}
```

- [ ] **Step 6: 运行测试确认通过**

```bash
cargo test --manifest-path src-tauri/Cargo.toml parses_tencent_hk_quote 2>&1
cargo test --manifest-path src-tauri/Cargo.toml parses_tencent_us_quote 2>&1
```

期望：PASS。

- [ ] **Step 7: 提交**

```bash
git add src-tauri/src/stock_quote.rs
git commit -m "feat: add multi-market tencent quote fetching (hk/us/a)"
```

---

### Task 3: 更新 format_stock_push_text 加市场标签

**Files:**
- Modify: `src-tauri/src/stock_quote.rs`

- [ ] **Step 1: 写市场标签格式化测试**

在 `#[cfg(test)]` 模块追加：

```rust
#[test]
fn formats_stock_push_text_with_market_labels() {
    let now = chrono::Local
        .with_ymd_and_hms(2026, 4, 28, 10, 30, 0)
        .unwrap();
    let text = format_stock_push_text(
        &[
            StockQuote {
                code: "600519".to_string(),
                name: "贵州茅台".to_string(),
                price: 1458.49,
                change: 39.49,
                change_percent: 2.78,
                valid: true,
            },
            StockQuote {
                code: "00700".to_string(),
                name: "腾讯控股".to_string(),
                price: 380.00,
                change: -2.00,
                change_percent: -0.52,
                valid: true,
            },
            StockQuote {
                code: "AAPL".to_string(),
                name: "苹果公司".to_string(),
                price: 189.30,
                change: 1.20,
                change_percent: 0.64,
                valid: true,
            },
        ],
        &[
            StockCode { code: "600519".to_string(), market: "a".to_string() },
            StockCode { code: "00700".to_string(), market: "hk".to_string() },
            StockCode { code: "AAPL".to_string(), market: "us".to_string() },
        ],
        now,
    );

    assert!(text.contains("600519 贵州茅台 1458.49 +39.49 +2.78%"));
    assert!(!text.contains("[A]")); // A 股不加标签
    assert!(text.contains("00700 腾讯控股 380.00 -2.00 -0.52% [港]"));
    assert!(text.contains("AAPL 苹果公司 189.30 +1.20 +0.64% [美]"));
}
```

- [ ] **Step 2: 运行测试确认失败**

```bash
cargo test --manifest-path src-tauri/Cargo.toml formats_stock_push_text_with_market_labels 2>&1
```

期望：编译报错（`format_stock_push_text` 签名不匹配）。

- [ ] **Step 3: 修改 format_stock_push_text 签名和实现**

将现有函数：

```rust
pub fn format_stock_push_text(
    quotes: &[StockQuote],
    now: chrono::DateTime<chrono::Local>,
) -> String {
```

替换为（新增 `stock_codes` 参数用于查市场）：

```rust
pub fn format_stock_push_text(
    quotes: &[StockQuote],
    stock_codes: &[StockCode],
    now: chrono::DateTime<chrono::Local>,
) -> String {
    let code_market: HashMap<String, String> = stock_codes
        .iter()
        .map(|sc| (sc.code.clone(), sc.market.clone()))
        .collect();

    let valid_quotes: Vec<&StockQuote> = quotes.iter().filter(|q| q.valid).collect();

    if valid_quotes.is_empty() {
        return format!(
            "更新时间：{}\n暂无有效股票数据",
            now.format("%Y-%m-%d %H:%M:%S")
        );
    }

    let mut lines = vec![
        format!("更新时间：{}", now.format("%Y-%m-%d %H:%M:%S")),
        String::new(),
    ];

    lines.extend(valid_quotes.iter().map(|quote| {
        let market_label = match code_market.get(&quote.code).map(String::as_str) {
            Some("hk") => " [港]",
            Some("us") => " [美]",
            _ => "",
        };
        format!(
            "{} {} {:.2} {} {}%{}",
            quote.code,
            quote.name,
            quote.price,
            signed_amount(quote.change),
            signed_amount(quote.change_percent),
            market_label
        )
    }));

    lines.join("\n")
}
```

- [ ] **Step 4: 修复旧测试（签名变更导致编译报错）**

旧测试 `formats_stock_push_text_with_one_timestamp_and_names` 和 `formats_stock_push_text_filters_invalid_quotes` 需要追加第二个参数。找到这两个测试，在 `format_stock_push_text(` 调用中加入空的 `stock_codes` 参数：

```rust
// 原来：
format_stock_push_text(&[...], now)

// 改为：
format_stock_push_text(&[...], &[], now)
```

- [ ] **Step 5: 运行所有测试**

```bash
cargo test --manifest-path src-tauri/Cargo.toml 2>&1 | tail -20
```

期望：test result: ok，0 failed。

- [ ] **Step 6: 提交**

```bash
git add src-tauri/src/stock_quote.rs
git commit -m "feat: add market labels ([港]/[美]) to stock push text"
```

---

### Task 4: 更新 models.rs — StockWatchRecord 加 market 字段

**Files:**
- Modify: `src-tauri/src/models.rs`

- [ ] **Step 1: 修改 StockWatchRecord**

找到 `src-tauri/src/models.rs` 中的 `StockWatchRecord`，将：

```rust
pub struct StockWatchRecord {
    pub code: String,
    #[serde(alias = "created_at", rename = "createdAt")]
    pub created_at: String,
}
```

替换为：

```rust
pub struct StockWatchRecord {
    pub code: String,
    #[serde(default = "default_market")]
    pub market: String,
    #[serde(alias = "created_at", rename = "createdAt")]
    pub created_at: String,
}

fn default_market() -> String {
    "a".to_string()
}
```

- [ ] **Step 2: 编译确认**

```bash
cargo build --manifest-path src-tauri/Cargo.toml 2>&1 | grep "^error" | head -20
```

此步会有编译错误，因为 `state.rs` 中 `add_stock_watch` 构造 `StockWatchRecord` 时没有 `market` 字段——下一步修复。

- [ ] **Step 3: 提交**

```bash
git add src-tauri/src/models.rs
git commit -m "feat: add market field to StockWatchRecord with backward-compatible default"
```

---

### Task 5: 更新 state.rs — 使用 StockCode，修复编译错误

**Files:**
- Modify: `src-tauri/src/state.rs`

需要修改 `add_stock_watch`、`remove_stock_watch`、`push_stock_quotes`、`fetch_stock_quotes`，以及自由函数 `execute_stock_push`。

- [ ] **Step 1: 修改 add_stock_watch**

找到 `pub fn add_stock_watch(&self, code: &str)` 方法，将其替换为：

```rust
pub fn add_stock_watch(&self, code: &str) -> anyhow::Result<StockWatchRecord> {
    let sc = crate::stock_quote::parse_stock_input(code)?;
    let mut records = self.load_stock_watchlist()?;

    if records.iter().any(|record| record.code == sc.code && record.market == sc.market) {
        anyhow::bail!("股票代码 {} 已存在", sc.code);
    }

    let record = StockWatchRecord {
        code: sc.code,
        market: sc.market,
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    records.push(record.clone());
    self.save_stock_watchlist(&records)?;

    Ok(record)
}
```

- [ ] **Step 2: 修改 remove_stock_watch**

找到 `pub fn remove_stock_watch(&self, code: &str)` 方法，将其替换为：

```rust
pub fn remove_stock_watch(&self, code: &str) -> anyhow::Result<()> {
    let sc = crate::stock_quote::parse_stock_input(code)?;
    let mut records = self.load_stock_watchlist()?;
    let before = records.len();
    records.retain(|record| !(record.code == sc.code && record.market == sc.market));

    if records.len() == before {
        anyhow::bail!("股票代码 {} 未找到", sc.code);
    }

    self.save_stock_watchlist(&records)
}
```

- [ ] **Step 3: 修改 push_stock_quotes**

找到 `pub async fn push_stock_quotes` 方法，将 codes 提取逻辑替换为 StockCode：

```rust
pub async fn push_stock_quotes(&self, device_id: &str, page_id: u32) -> anyhow::Result<()> {
    let records = self.load_stock_watchlist()?;
    if records.is_empty() {
        anyhow::bail!("股票列表为空");
    }

    let stock_codes: Vec<crate::stock_quote::StockCode> = records
        .iter()
        .map(|r| crate::stock_quote::StockCode {
            code: r.code.clone(),
            market: r.market.clone(),
        })
        .collect();
    let quotes = crate::stock_quote::fetch_stock_quotes(&stock_codes).await?;
    crate::stock_quote::ensure_has_valid_stock_quote(&quotes)?;
    let text = crate::stock_quote::format_stock_push_text(&quotes, &stock_codes, chrono::Local::now());

    self.push_text(&text, Some(20), device_id, Some(page_id))
        .await
}
```

- [ ] **Step 4: 修改 fetch_stock_quotes（state 方法）**

找到 `pub async fn fetch_stock_quotes(&self)` 方法，替换为：

```rust
pub async fn fetch_stock_quotes(&self) -> anyhow::Result<Vec<crate::stock_quote::StockQuote>> {
    let records = self.load_stock_watchlist()?;
    if records.is_empty() {
        return Ok(Vec::new());
    }

    let stock_codes: Vec<crate::stock_quote::StockCode> = records
        .iter()
        .map(|r| crate::stock_quote::StockCode {
            code: r.code.clone(),
            market: r.market.clone(),
        })
        .collect();
    crate::stock_quote::fetch_stock_quotes(&stock_codes).await
}
```

- [ ] **Step 5: 修改 execute_stock_push（自由函数）**

找到文件顶部附近的 `async fn execute_stock_push` 函数，将 codes 提取逻辑替换为：

```rust
let stock_codes: Vec<crate::stock_quote::StockCode> = watchlist
    .iter()
    .map(|r| crate::stock_quote::StockCode {
        code: r.code.clone(),
        market: r.market.clone(),
    })
    .collect();
let quotes = crate::stock_quote::fetch_stock_quotes(&stock_codes).await?;
crate::stock_quote::ensure_has_valid_stock_quote(&quotes)?;
let text = crate::stock_quote::format_stock_push_text(&quotes, &stock_codes, chrono::Local::now());
```

（删除原来的 `let codes = watchlist.iter().map(|r| r.code.clone()).collect::<Vec<_>>();` 这行以及下面旧的 fetch/format 调用。）

- [ ] **Step 6: 编译验证**

```bash
cargo build --manifest-path src-tauri/Cargo.toml 2>&1 | grep "^error" | head -30
```

期望：0 个 error。

- [ ] **Step 7: 运行所有 Rust 测试**

```bash
cargo test --manifest-path src-tauri/Cargo.toml 2>&1 | tail -20
```

期望：test result: ok，0 failed。

- [ ] **Step 8: 提交**

```bash
git add src-tauri/src/state.rs
git commit -m "feat: update state.rs to use StockCode for multi-market watchlist"
```

---

### Task 6: 更新前端 tauri.ts 和 stock-push-page.tsx

**Files:**
- Modify: `src/lib/tauri.ts`
- Modify: `src/features/stocks/stock-push-page.tsx`

- [ ] **Step 1: 更新 tauri.ts 的 StockWatchRecord 类型**

找到 `src/lib/tauri.ts` 中：

```typescript
export type StockWatchRecord = {
  code: string;
  createdAt: string;
};
```

替换为：

```typescript
export type StockWatchRecord = {
  code: string;
  market: string; // "a" | "hk" | "us"
  createdAt: string;
};
```

- [ ] **Step 2: 更新 stock-push-page.tsx — 移除 validateCode 旧逻辑**

找到 `validateCode` 函数：

```typescript
function validateCode(code: string): string | null {
  if (!/^\d{6}$/.test(code)) {
    return "股票代码必须是 6 位数字";
  }
  return null;
}
```

替换为：

```typescript
function validateCode(code: string): string | null {
  const trimmed = code.trim();
  if (!trimmed) return "请输入股票代码";
  const allDigits = /^\d+$/.test(trimmed);
  const hasLetter = /[a-zA-Z]/.test(trimmed);
  if (allDigits && trimmed.length > 6) return "纯数字代码不能超过 6 位";
  if (!allDigits && !hasLetter) return "无法识别的股票代码格式";
  return null;
}
```

- [ ] **Step 3: 更新输入框 placeholder 和 inputMode**

找到输入框：

```tsx
placeholder="例如 600519"
inputMode="numeric"
maxLength={6}
```

替换为：

```tsx
placeholder="A股: 600519 | 港股: 00700 | 美股: AAPL"
maxLength={10}
```

（删除 `inputMode="numeric"` 这行，因为美股代码含字母。）

- [ ] **Step 4: 更新股票列表展示，加市场 badge**

找到股票列表中显示 `stock.code` 的部分：

```tsx
<span className="font-mono text-sm">
  {stock.code}
  {stockName ? <span className="ml-2 text-xs text-gray-500">({stockName})</span> : null}
</span>
```

替换为：

```tsx
<span className="font-mono text-sm flex items-center gap-1.5">
  {stock.code}
  {stock.market === "hk" && (
    <span className="rounded px-1 py-0.5 text-xs bg-blue-100 text-blue-700">港</span>
  )}
  {stock.market === "us" && (
    <span className="rounded px-1 py-0.5 text-xs bg-green-100 text-green-700">美</span>
  )}
  {stockName ? <span className="text-xs text-gray-500">({stockName})</span> : null}
</span>
```

- [ ] **Step 5: 更新页面 header 描述文字**

找到：

```tsx
<p className="text-sm text-gray-500">维护 A 股代码列表，实时获取行情后推送到设备的指定页面。</p>
```

替换为：

```tsx
<p className="text-sm text-gray-500">维护股票代码列表（A股/港股/美股），实时获取行情后推送到设备的指定页面。</p>
```

- [ ] **Step 6: 运行前端测试**

```bash
pnpm vitest run 2>&1 | tail -20
```

期望：所有测试通过。

- [ ] **Step 7: 提交**

```bash
git add src/lib/tauri.ts src/features/stocks/stock-push-page.tsx
git commit -m "feat: update frontend to support multi-market stock codes"
```

---

### Task 7: 验收测试

- [ ] **Step 1: 运行全部 Rust 测试**

```bash
cargo test --manifest-path src-tauri/Cargo.toml 2>&1 | tail -10
```

期望：test result: ok，0 failed。

- [ ] **Step 2: 运行全部前端测试**

```bash
pnpm vitest run 2>&1 | tail -10
```

期望：所有测试通过。

- [ ] **Step 3: 最终提交（如有遗漏文件）**

```bash
git status
# 确认干净后无需额外提交
```
