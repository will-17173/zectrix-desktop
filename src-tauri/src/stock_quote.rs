use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StockQuote {
    pub code: String,
    pub name: String,
    pub price: f64,
    pub change: f64,
    pub change_percent: f64,
    pub valid: bool,
}

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

pub fn normalize_stock_code(code: &str) -> anyhow::Result<String> {
    let normalized = code.trim();
    if normalized.len() != 6 || !normalized.chars().all(|c| c.is_ascii_digit()) {
        anyhow::bail!("股票代码必须是 6 位数字");
    }

    Ok(normalized.to_string())
}

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
        return format!("更新时间：{}\n暂无有效股票数据", now.format("%Y-%m-%d %H:%M:%S"));
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

pub fn parse_eastmoney_quotes(
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
        let is_valid = !name.is_empty()
            && item.f2.as_str() != Some("-")
            && !item.f2.is_null();

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
            name: if name.is_empty() { "未知".to_string() } else { name.to_string() },
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

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .connect_timeout(std::time::Duration::from_secs(5))
        .user_agent("Mozilla/5.0")
        .build()?;

    let response = client
        .get(&url)
        .header("Referer", "https://quote.eastmoney.com/")
        .send()
        .await?;

    if !response.status().is_success() {
        anyhow::bail!("东方财富 API 返回错误: {}", response.status());
    }

    let body = response.text().await?;
    parse_eastmoney_quotes(&body, codes)
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
            parse_eastmoney_quotes(body, &["000001".to_string(), "600519".to_string()])
                .unwrap();

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

        let quotes = parse_eastmoney_quotes(
            body,
            &["000001".to_string(), " 600519 ".to_string()],
        )
        .unwrap();

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

        let quotes = parse_eastmoney_quotes(body, &["600519".to_string(), "000001".to_string()])
            .unwrap();

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

        let quotes = parse_eastmoney_quotes(body, &["600001".to_string()])
            .unwrap();

        assert_eq!(quotes[0].code, "600001");
        assert_eq!(quotes[0].name, "邯郸钢铁");
        assert!(!quotes[0].valid);
        assert_eq!(quotes[0].price, 0.0);
        assert_eq!(quotes[0].change, 0.0);
        assert_eq!(quotes[0].change_percent, 0.0);
    }
}