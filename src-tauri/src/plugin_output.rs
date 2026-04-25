use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextImageStyle {
    #[serde(default = "default_font_size")]
    pub font_size: u32,
    #[serde(default = "default_line_height")]
    pub line_height: f32,
    #[serde(default = "default_padding")]
    pub padding: u32,
    #[serde(default = "default_align")]
    pub align: String,
    #[serde(default = "default_vertical_align")]
    pub vertical_align: String,
}

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
pub struct TextImagePluginOutput {
    pub text: String,
    #[serde(default)]
    pub style: TextImageStyle,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImagePluginOutput {
    pub image_data_url: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum PluginOutput {
    Text(TextPluginOutput),
    TextImage(TextImagePluginOutput),
    Image(ImagePluginOutput),
}

impl Default for TextImageStyle {
    fn default() -> Self {
        Self {
            font_size: default_font_size(),
            line_height: default_line_height(),
            padding: default_padding(),
            align: default_align(),
            vertical_align: default_vertical_align(),
        }
    }
}

fn default_font_size() -> u32 {
    20
}

fn default_line_height() -> f32 {
    1.25
}

fn default_padding() -> u32 {
    16
}

fn default_align() -> String {
    "left".into()
}

fn default_vertical_align() -> String {
    "top".into()
}

fn validate_font_size(font_size: u32) -> anyhow::Result<()> {
    if !(8..=72).contains(&font_size) {
        anyhow::bail!("fontSize 必须在 8 到 72 之间");
    }
    Ok(())
}

fn validate_style(style: &TextImageStyle) -> anyhow::Result<()> {
    validate_font_size(style.font_size)?;
    if !(0.8..=3.0).contains(&style.line_height) {
        anyhow::bail!("lineHeight 必须在 0.8 到 3.0 之间");
    }
    if style.padding > 120 {
        anyhow::bail!("padding 不能超过 120");
    }
    if !matches!(style.align.as_str(), "left" | "center" | "right") {
        anyhow::bail!("align 只能是 left、center 或 right");
    }
    if !matches!(style.vertical_align.as_str(), "top" | "middle") {
        anyhow::bail!("verticalAlign 只能是 top 或 middle");
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
        "textImage" => {
            let output: TextImagePluginOutput = serde_json::from_value(raw)?;
            require_text(&output.text)?;
            validate_style(&output.style)?;
            Ok(PluginOutput::TextImage(output))
        }
        "image" => {
            let output: ImagePluginOutput = serde_json::from_value(raw)?;
            if !output.image_data_url.starts_with("data:image/") {
                anyhow::bail!("imageDataUrl 必须是 data:image/...;base64 格式");
            }
            if !output.image_data_url.contains(";base64,") {
                anyhow::bail!("imageDataUrl 必须包含 base64 图片数据");
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
    fn parses_text_image_output_with_default_style() {
        let raw = serde_json::json!({ "type": "textImage", "text": "第一行\n第二行" });

        let parsed = parse_plugin_output(raw).unwrap();

        match parsed {
            PluginOutput::TextImage(output) => {
                assert_eq!(output.text, "第一行\n第二行");
                assert_eq!(output.style.font_size, 20);
                assert_eq!(output.style.align, "left");
                assert_eq!(output.style.vertical_align, "top");
            }
            _ => panic!("expected textImage output"),
        }
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
    fn rejects_invalid_alignment() {
        let raw = serde_json::json!({
            "type": "textImage",
            "text": "hello",
            "style": { "align": "justify" }
        });

        let err = parse_plugin_output(raw).unwrap_err().to_string();

        assert!(err.contains("align"));
    }
}