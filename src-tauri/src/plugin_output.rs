use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextPluginOutput {
    pub text: String,
    #[serde(default = "default_font_size")]
    pub font_size: u32,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImagePluginOutput {
    #[serde(default)]
    pub image_data_url: String,
    #[serde(default, alias = "url")]
    pub image_url: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum PluginOutput {
    Text(TextPluginOutput),
    Image(ImagePluginOutput),
}

fn default_font_size() -> u32 {
    20
}

fn validate_font_size(font_size: u32) -> anyhow::Result<()> {
    if !(8..=72).contains(&font_size) {
        anyhow::bail!("fontSize 必须在 8 到 72 之间");
    }
    Ok(())
}

fn require_text(text: &str) -> anyhow::Result<()> {
    if text.trim().is_empty() {
        anyhow::bail!("text 不能为空");
    }
    if text.len() > 256 * 1024 {
        anyhow::bail!("text 超过 256KB 限制");
    }
    Ok(())
}

pub fn parse_plugin_output(raw: serde_json::Value) -> anyhow::Result<PluginOutput> {
    let output_type = raw
        .get("type")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("插件输出缺少 type"))?;

    match output_type {
        "text" => {
            let output: TextPluginOutput = serde_json::from_value(raw)?;
            require_text(&output.text)?;
            validate_font_size(output.font_size)?;
            Ok(PluginOutput::Text(output))
        }
        "image" => {
            let output: ImagePluginOutput = serde_json::from_value(raw)?;
            let has_data_url = !output.image_data_url.trim().is_empty();
            let has_image_url = output
                .image_url
                .as_deref()
                .is_some_and(|url| !url.trim().is_empty());

            if !has_data_url && !has_image_url {
                anyhow::bail!("图片输出必须提供 imageDataUrl 或 imageUrl");
            }

            if has_data_url {
                if !output.image_data_url.starts_with("data:image/") {
                    anyhow::bail!("imageDataUrl 必须是 data:image/...;base64 格式");
                }
                if !output.image_data_url.contains(";base64,") {
                    anyhow::bail!("imageDataUrl 必须包含 base64 图片数据");
                }
            }
            Ok(PluginOutput::Image(output))
        }
        other => anyhow::bail!("不支持的插件输出 type: {other}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_text_output_with_default_font_size() {
        let raw = serde_json::json!({ "type": "text", "text": "hello" });

        let parsed = parse_plugin_output(raw).unwrap();

        match parsed {
            PluginOutput::Text(output) => {
                assert_eq!(output.text, "hello");
                assert_eq!(output.font_size, 20);
            }
            _ => panic!("expected text output"),
        }
    }

    #[test]
    fn rejects_text_image_output() {
        let raw = serde_json::json!({ "type": "textImage", "text": "第一行\n第二行" });

        let err = parse_plugin_output(raw).unwrap_err().to_string();

        assert!(err.contains("不支持的插件输出 type: textImage"));
    }

    #[test]
    fn parses_image_data_url() {
        let raw = serde_json::json!({
            "type": "image",
            "imageDataUrl": "data:image/png;base64,iVBORw0KGgo="
        });

        let parsed = parse_plugin_output(raw).unwrap();

        match parsed {
            PluginOutput::Image(output) => {
                assert!(output.image_data_url.starts_with("data:image/png;base64,"));
                assert!(output.image_url.is_none());
            }
            _ => panic!("expected image output"),
        }
    }

    #[test]
    fn parses_image_url() {
        let raw = serde_json::json!({
            "type": "image",
            "imageUrl": "https://example.com/card.png"
        });

        let parsed = parse_plugin_output(raw).unwrap();

        match parsed {
            PluginOutput::Image(output) => {
                assert_eq!(output.image_data_url, "");
                assert_eq!(
                    output.image_url.as_deref(),
                    Some("https://example.com/card.png")
                );
            }
            _ => panic!("expected image output"),
        }
    }

    #[test]
    fn parses_image_url_alias_url() {
        let raw = serde_json::json!({
            "type": "image",
            "url": "https://example.com/card.png"
        });

        let parsed = parse_plugin_output(raw).unwrap();

        match parsed {
            PluginOutput::Image(output) => {
                assert_eq!(
                    output.image_url.as_deref(),
                    Some("https://example.com/card.png")
                );
            }
            _ => panic!("expected image output"),
        }
    }

    #[test]
    fn rejects_missing_type() {
        let raw = serde_json::json!({ "text": "hello" });

        let err = parse_plugin_output(raw).unwrap_err().to_string();

        assert!(err.contains("缺少 type"));
    }

    #[test]
    fn rejects_empty_text() {
        let raw = serde_json::json!({ "type": "text", "text": "   " });

        let err = parse_plugin_output(raw).unwrap_err().to_string();

        assert!(err.contains("text 不能为空"));
    }

    #[test]
    fn rejects_text_image_style_options_as_unsupported_output() {
        let raw = serde_json::json!({
            "type": "textImage",
            "text": "hello",
            "style": { "align": "justify" }
        });

        let err = parse_plugin_output(raw).unwrap_err().to_string();

        assert!(err.contains("不支持的插件输出 type: textImage"));
    }
}
