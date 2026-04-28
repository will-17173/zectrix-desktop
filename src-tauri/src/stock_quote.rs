use std::collections::HashMap;

use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StockQuote {
    pub code: String,
    pub name: String,
    pub price: f64,
    pub change: f64,
    pub change_percent: f64,
    pub valid: bool,
}

#[cfg(test)]
#[derive(Debug, Deserialize)]
struct EastmoneyResponse {
    data: Option<EastmoneyData>,
}

#[cfg(test)]
#[derive(Debug, Deserialize)]
struct EastmoneyData {
    diff: Vec<EastmoneyQuote>,
}

#[cfg(test)]
#[derive(Debug, Deserialize)]
struct EastmoneyQuote {
    f2: serde_json::Value,
    f3: serde_json::Value,
    f4: serde_json::Value,
    f12: String,
    f14: String,
}

#[cfg(test)]
#[derive(Debug, Deserialize)]
struct EastmoneyStockResponse {
    data: Option<EastmoneyStockData>,
}

#[cfg(test)]
#[derive(Debug, Deserialize)]
struct EastmoneyStockData {
    f43: serde_json::Value,
    f57: String,
    f58: String,
    f169: serde_json::Value,
    f170: serde_json::Value,
}

pub fn normalize_stock_code(code: &str) -> anyhow::Result<String> {
    let normalized = code.trim();
    if normalized.len() != 6 || !normalized.chars().all(|c| c.is_ascii_digit()) {
        anyhow::bail!("股票代码必须是 6 位数字");
    }

    Ok(normalized.to_string())
}

#[cfg(test)]
pub fn stock_code_to_secid(code: &str) -> anyhow::Result<String> {
    let normalized = normalize_stock_code(code)?;
    // 6 开头为上海证券交易所 (前缀1)，其他为深圳证券交易所 (前缀0)
    let prefix = if normalized.starts_with('6') {
        "1"
    } else {
        "0"
    };
    Ok(format!("{prefix}.{normalized}"))
}

fn signed_amount(value: f64) -> String {
    if value > 0.0 {
        format!("+{value:.2}")
    } else {
        format!("{value:.2}")
    }
}

pub fn format_stock_push_text(
    quotes: &[StockQuote],
    now: chrono::DateTime<chrono::Local>,
) -> String {
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

pub fn ensure_has_valid_stock_quote(quotes: &[StockQuote]) -> anyhow::Result<()> {
    if quotes.iter().any(|quote| quote.valid) {
        return Ok(());
    }

    anyhow::bail!("无有效股票数据，请检查股票代码或稍后重试")
}

#[cfg(test)]
fn number_field(value: &serde_json::Value, field: &str, code: &str) -> anyhow::Result<f64> {
    // 停牌或退市股票返回 "-"，视为无数据
    if let Some(text) = value.as_str() {
        if text == "-" {
            return Ok(0.0);
        }
        return text
            .parse::<f64>()
            .map_err(|_| anyhow::anyhow!("股票 {code} 的 {field} 字段无法解析"));
    }

    if let Some(number) = value.as_f64() {
        return Ok(number);
    }

    // null 或缺失也视为无数据
    if value.is_null() {
        return Ok(0.0);
    }

    anyhow::bail!("股票 {code} 缺少有效的 {field} 字段")
}

fn unavailable_stock_quote(code: &str, reason: Option<&str>) -> StockQuote {
    StockQuote {
        code: code.to_string(),
        name: reason.unwrap_or("未知").to_string(),
        price: 0.0,
        change: 0.0,
        change_percent: 0.0,
        valid: false,
    }
}

fn stock_code_to_tencent_symbol(code: &str) -> anyhow::Result<String> {
    let normalized = normalize_stock_code(code)?;
    let market = if normalized.starts_with('6') {
        "sh"
    } else if normalized.starts_with('0') || normalized.starts_with('3') {
        "sz"
    } else if normalized.starts_with('4') || normalized.starts_with('8') {
        "bj"
    } else {
        "sz"
    };
    Ok(format!("{market}{normalized}"))
}

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

fn parse_tencent_quotes(body: &str, requested_codes: &[String]) -> anyhow::Result<Vec<StockQuote>> {
    let mut by_code = HashMap::new();

    for line in body.lines() {
        let Some((name_part, value_part)) = line.split_once("=\"") else {
            continue;
        };
        let raw_value = value_part
            .trim()
            .trim_end_matches(';')
            .trim_end_matches('"');
        let fields = raw_value.split('~').collect::<Vec<_>>();
        let Some(code) = fields.get(2).map(|field| field.trim().to_string()) else {
            continue;
        };
        if normalize_stock_code(&code).is_err() || !name_part.contains(&code) {
            continue;
        }

        if fields.len() < 33 {
            by_code.insert(code.clone(), unavailable_stock_quote(&code, None));
            continue;
        }

        let name = fields[1].trim();
        let price = fields[3].trim().parse::<f64>().unwrap_or(0.0);
        let change = fields[31].trim().parse::<f64>().unwrap_or(0.0);
        let change_percent = fields[32].trim().parse::<f64>().unwrap_or(0.0);
        let status = fields.get(40).copied().unwrap_or_default();
        let is_valid = !name.is_empty() && price > 0.0 && status != "D";

        by_code.insert(
            code.clone(),
            StockQuote {
                code,
                name: if name.is_empty() {
                    "未知".to_string()
                } else {
                    name.to_string()
                },
                price: if is_valid { price } else { 0.0 },
                change: if is_valid { change } else { 0.0 },
                change_percent: if is_valid { change_percent } else { 0.0 },
                valid: is_valid,
            },
        );
    }

    Ok(requested_codes
        .iter()
        .map(|code| normalize_stock_code(code))
        .collect::<anyhow::Result<Vec<_>>>()?
        .into_iter()
        .map(|code| {
            by_code
                .remove(&code)
                .unwrap_or_else(|| unavailable_stock_quote(&code, None))
        })
        .collect())
}

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

async fn fetch_tencent_quotes(codes: &[String]) -> anyhow::Result<Vec<StockQuote>> {
    if codes.is_empty() {
        anyhow::bail!("股票列表为空");
    }

    let symbols = codes
        .iter()
        .filter_map(|code| stock_code_to_tencent_symbol(code).ok())
        .collect::<Vec<_>>();
    if symbols.is_empty() {
        return Ok(codes
            .iter()
            .map(|code| unavailable_stock_quote(code.trim(), Some("代码无效")))
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
    parse_tencent_quotes(&body, codes)
}

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

#[cfg(test)]
fn parse_eastmoney_quotes(
    body: &str,
    requested_codes: &[String],
) -> anyhow::Result<Vec<StockQuote>> {
    let response: EastmoneyResponse = serde_json::from_str(body)?;
    let data = response
        .data
        .ok_or_else(|| anyhow::anyhow!("行情接口返回缺少 data 字段"))?;

    let mut by_code = HashMap::new();
    for item in data.diff {
        let code = item.f12;
        let name = item.f14.trim();

        // 名称为空或价格为 "-" 标记为无效
        let is_valid = !name.is_empty() && item.f2.as_str() != Some("-") && !item.f2.is_null();

        let price = if is_valid {
            number_field(&item.f2, "f2", &code)?
        } else {
            0.0
        };
        let change_percent = if is_valid {
            number_field(&item.f3, "f3", &code)?
        } else {
            0.0
        };
        let change = if is_valid {
            number_field(&item.f4, "f4", &code)?
        } else {
            0.0
        };

        let quote = StockQuote {
            code: code.clone(),
            name: if name.is_empty() {
                "未知".to_string()
            } else {
                name.to_string()
            },
            price,
            change_percent,
            change,
            valid: is_valid,
        };
        by_code.insert(code, quote);
    }

    // 处理请求的代码，API 未返回的标记为无效
    Ok(requested_codes
        .iter()
        .map(|code| normalize_stock_code(code))
        .collect::<anyhow::Result<Vec<_>>>()?
        .into_iter()
        .map(|code| {
            by_code
                .remove(&code)
                .map(|mut quote| {
                    quote.code = code.clone();
                    quote
                })
                .unwrap_or_else(|| StockQuote {
                    code: code.clone(),
                    name: "未知".to_string(),
                    price: 0.0,
                    change: 0.0,
                    change_percent: 0.0,
                    valid: false,
                })
        })
        .collect())
}

#[cfg(test)]
pub fn parse_eastmoney_stock_quote(body: &str, requested_code: &str) -> anyhow::Result<StockQuote> {
    let code = normalize_stock_code(requested_code)?;
    let response: EastmoneyStockResponse = serde_json::from_str(body)?;
    let Some(data) = response.data else {
        return Ok(unavailable_stock_quote(&code, None));
    };

    let name = data.f58.trim();
    let is_valid = data.f57 == code
        && !name.is_empty()
        && data.f43.as_str() != Some("-")
        && !data.f43.is_null();

    let price = if is_valid {
        number_field(&data.f43, "f43", &code)?
    } else {
        0.0
    };
    let change = if is_valid {
        number_field(&data.f169, "f169", &code)?
    } else {
        0.0
    };
    let change_percent = if is_valid {
        number_field(&data.f170, "f170", &code)?
    } else {
        0.0
    };

    Ok(StockQuote {
        code,
        name: if name.is_empty() {
            "未知".to_string()
        } else {
            name.to_string()
        },
        price,
        change,
        change_percent,
        valid: is_valid,
    })
}

pub async fn fetch_stock_quotes(stock_codes: &[StockCode]) -> anyhow::Result<Vec<StockQuote>> {
    fetch_tencent_quotes_v2(stock_codes).await
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
        // 任意 6 位数字都可以
        assert_eq!(stock_code_to_secid("830000").unwrap(), "0.830000");
    }

    #[test]
    fn rejects_invalid_stock_codes() {
        assert!(stock_code_to_secid("688")
            .unwrap_err()
            .to_string()
            .contains("6 位数字"));
        assert!(stock_code_to_secid("abc001")
            .unwrap_err()
            .to_string()
            .contains("6 位数字"));
    }

    #[test]
    fn formats_stock_push_text_with_one_timestamp_and_names() {
        let now = chrono::Local
            .with_ymd_and_hms(2026, 4, 25, 10, 30, 12)
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
                    code: "600000".to_string(),
                    name: "浦发银行".to_string(),
                    price: 9.45,
                    change: -0.09,
                    change_percent: -0.94,
                    valid: true,
                },
            ],
            now,
        );

        assert_eq!(
            text,
            "更新时间：2026-04-25 10:30:12\n\n600519 贵州茅台 1458.49 +39.49 +2.78%\n600000 浦发银行 9.45 -0.09 -0.94%"
        );
    }

    #[test]
    fn formats_stock_push_text_filters_invalid_quotes() {
        let now = chrono::Local
            .with_ymd_and_hms(2026, 4, 25, 10, 30, 12)
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
                    code: "999999".to_string(),
                    name: "未知".to_string(),
                    price: 0.0,
                    change: 0.0,
                    change_percent: 0.0,
                    valid: false,
                },
            ],
            now,
        );

        assert_eq!(
            text,
            "更新时间：2026-04-25 10:30:12\n\n600519 贵州茅台 1458.49 +39.49 +2.78%"
        );
    }

    #[test]
    fn rejects_pushes_when_every_quote_is_invalid() {
        let result = ensure_has_valid_stock_quote(&[
            StockQuote {
                code: "999999".to_string(),
                name: "未知".to_string(),
                price: 0.0,
                change: 0.0,
                change_percent: 0.0,
                valid: false,
            },
            StockQuote {
                code: "000000".to_string(),
                name: "未知".to_string(),
                price: 0.0,
                change: 0.0,
                change_percent: 0.0,
                valid: false,
            },
        ]);

        assert!(result.unwrap_err().to_string().contains("无有效股票数据"));
    }

    #[test]
    fn allows_pushes_when_any_quote_is_valid() {
        let result = ensure_has_valid_stock_quote(&[
            StockQuote {
                code: "600519".to_string(),
                name: "贵州茅台".to_string(),
                price: 1458.49,
                change: 39.49,
                change_percent: 2.78,
                valid: true,
            },
            StockQuote {
                code: "999999".to_string(),
                name: "未知".to_string(),
                price: 0.0,
                change: 0.0,
                change_percent: 0.0,
                valid: false,
            },
        ]);

        assert!(result.is_ok());
    }

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

        let quotes =
            parse_eastmoney_quotes(body, &["000001".to_string(), "600519".to_string()]).unwrap();

        assert_eq!(quotes[0].code, "000001");
        assert_eq!(quotes[0].name, "平安银行");
        assert!(quotes[0].valid);
        assert_eq!(quotes[1].code, "600519");
        assert_eq!(quotes[1].name, "贵州茅台");
        assert!(quotes[1].valid);
    }

    #[test]
    fn parses_eastmoney_quotes_with_trimmed_requested_codes_in_requested_order() {
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

        let quotes =
            parse_eastmoney_quotes(body, &["000001".to_string(), " 600519 ".to_string()]).unwrap();

        assert_eq!(quotes[0].code, "000001");
        assert_eq!(quotes[0].name, "平安银行");
        assert_eq!(quotes[1].code, "600519");
        assert_eq!(quotes[1].name, "贵州茅台");
    }

    #[test]
    fn marks_missing_stock_as_invalid() {
        let body = r#"{
            "rc": 0,
            "data": {
                "total": 1,
                "diff": [
                    {"f2":1458.49,"f3":2.78,"f4":39.49,"f12":"600519","f14":"贵州茅台"}
                ]
            }
        }"#;

        let quotes =
            parse_eastmoney_quotes(body, &["600519".to_string(), "000001".to_string()]).unwrap();

        assert_eq!(quotes[0].code, "600519");
        assert!(quotes[0].valid);
        assert_eq!(quotes[1].code, "000001");
        assert!(!quotes[1].valid);
        assert_eq!(quotes[1].name, "未知");
    }

    #[test]
    fn handles_suspended_stock_with_dash_values() {
        let body = r#"{
            "rc": 0,
            "data": {
                "total": 1,
                "diff": [
                    {"f2":"-","f3":"-","f4":"-","f12":"600001","f14":"邯郸钢铁"}
                ]
            }
        }"#;

        let quotes = parse_eastmoney_quotes(body, &["600001".to_string()]).unwrap();

        assert_eq!(quotes[0].code, "600001");
        assert_eq!(quotes[0].name, "邯郸钢铁");
        assert!(!quotes[0].valid);
        assert_eq!(quotes[0].price, 0.0);
        assert_eq!(quotes[0].change, 0.0);
        assert_eq!(quotes[0].change_percent, 0.0);
    }

    #[test]
    fn parses_eastmoney_single_stock_response() {
        let body = r#"{
            "rc": 0,
            "data": {
                "f43": 1458.49,
                "f57": "600519",
                "f58": "贵州茅台",
                "f169": 39.49,
                "f170": 2.78
            }
        }"#;

        let quote = parse_eastmoney_stock_quote(body, "600519").unwrap();

        assert_eq!(quote.code, "600519");
        assert_eq!(quote.name, "贵州茅台");
        assert_eq!(quote.price, 1458.49);
        assert_eq!(quote.change, 39.49);
        assert_eq!(quote.change_percent, 2.78);
        assert!(quote.valid);
    }

    #[test]
    fn parses_eastmoney_single_stock_dash_values_as_invalid() {
        let body = r#"{
            "rc": 0,
            "data": {
                "f43": "-",
                "f57": "600001",
                "f58": "邯郸钢铁",
                "f169": "-",
                "f170": "-"
            }
        }"#;

        let quote = parse_eastmoney_stock_quote(body, "600001").unwrap();

        assert_eq!(quote.code, "600001");
        assert_eq!(quote.name, "邯郸钢铁");
        assert_eq!(quote.price, 0.0);
        assert_eq!(quote.change, 0.0);
        assert_eq!(quote.change_percent, 0.0);
        assert!(!quote.valid);
    }

    #[test]
    fn creates_invalid_quote_for_unavailable_stock_data() {
        let quote = unavailable_stock_quote("600001", Some("连接失败"));

        assert_eq!(quote.code, "600001");
        assert_eq!(quote.name, "连接失败");
        assert_eq!(quote.price, 0.0);
        assert_eq!(quote.change, 0.0);
        assert_eq!(quote.change_percent, 0.0);
        assert!(!quote.valid);
    }

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

    #[test]
    fn parses_tencent_quotes_in_requested_order_and_marks_delisted_stock_invalid() {
        let body = r#"v_sh600000="1~浦发银行~600000~9.45~9.54~9.53~848590~275116~573474~9.44~2856~9.43~17850~9.42~5989~9.41~5122~9.40~8231~9.45~351~9.46~836~9.47~1191~9.48~956~9.49~561~~20260424161422~-0.09~-0.94~9.62~9.43~9.45/848590/806720096~848590~80672~0.25~6.29~~9.62~9.43~1.99~3147.40~3147.40~0.43~10.49~8.59~1.22~36153~9.51~6.29~6.29~~~0.32~80672.0096~0.0000~0~ ~GP-A~-24.04~-4.16~3.94~6.12~0.50~14.39~9.43~-4.45~-6.06~-12.01~33305838300~33305838300~82.27~-27.70~33305838300~~~-8.52~0.00~~CNY~0~___D__F__N~9.38~3012~";
v_sh600019="1~宝钢股份~600019~6.35~6.38~6.39~588359~320458~267901~6.35~2841~6.34~7928~6.33~9935~6.32~5417~6.31~11474~6.36~4603~6.37~8643~6.38~6578~6.39~6620~6.40~8505~~20260424161403~-0.03~-0.47~6.39~6.31~6.35/588359/373815250~588359~37382~0.27~14.65~~6.39~6.31~1.25~1383.16~1383.16~0.68~7.02~5.74~0.57~";
v_sh600001="1~邯郸钢铁~600001~5.29~5.29~0.00~0~0~0~0.00~0~0.00~0~0.00~0~0.00~0~0.00~0~0.00~0~0.00~0~0.00~0~0.00~0~0.00~0~~20260424090000~0.00~0.00~0.00~0.00~5.29/0/0~0~0~0.00~-673.65~D~0.00~0.00~0.00~148.99~148.99~1.20~-1~-1~0.00~0~0.00~55.62~24.86~~~~0.0000~0.0000~0~ ~GP-A~0.00~0.00~0.00~-0.18~-0.08~~~0.00~0.00~0.00~2816456569~2816456569~~0.00~2816456569~~~0.00~0.00~~CNY~0~~0.00~0~";"#;

        let quotes = parse_tencent_quotes(
            body,
            &[
                "600001".to_string(),
                "600000".to_string(),
                "600019".to_string(),
            ],
        )
        .unwrap();

        assert_eq!(quotes[0].code, "600001");
        assert!(!quotes[0].valid);
        assert_eq!(quotes[1].name, "浦发银行");
        assert_eq!(quotes[1].price, 9.45);
        assert_eq!(quotes[1].change, -0.09);
        assert_eq!(quotes[1].change_percent, -0.94);
        assert!(quotes[1].valid);
        assert_eq!(quotes[2].name, "宝钢股份");
        assert!(quotes[2].valid);
    }

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
}
