use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct StockQuote {
    pub code: String,
    pub name: String,
    pub price: f64,
    pub change: f64,
    pub change_percent: f64,
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

    let first = normalized.chars().next().unwrap();
    if first != '0' && first != '3' && first != '6' {
        anyhow::bail!("仅支持 0、3、6 开头的 A 股代码");
    }

    Ok(normalized.to_string())
}

pub fn stock_code_to_secid(code: &str) -> anyhow::Result<String> {
    let normalized = normalize_stock_code(code)?;
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

fn number_field(value: &serde_json::Value, field: &str, code: &str) -> anyhow::Result<f64> {
    if let Some(number) = value.as_f64() {
        return Ok(number);
    }

    if let Some(text) = value.as_str() {
        return text
            .parse::<f64>()
            .map_err(|_| anyhow::anyhow!("股票 {code} 的 {field} 字段无法解析"));
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
        .map(|code| normalize_stock_code(code))
        .collect::<anyhow::Result<Vec<_>>>()?
        .into_iter()
        .map(|code| {
            by_code
                .remove(&code)
                .ok_or_else(|| anyhow::anyhow!("行情接口未返回股票 {code}"))
        })
        .collect()
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
        assert!(stock_code_to_secid("688")
            .unwrap_err()
            .to_string()
            .contains("6 位数字"));
        assert!(stock_code_to_secid("830000")
            .unwrap_err()
            .to_string()
            .contains("仅支持"));
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
        assert_eq!(quotes[1].code, "600519");
        assert_eq!(quotes[1].name, "贵州茅台");
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
}