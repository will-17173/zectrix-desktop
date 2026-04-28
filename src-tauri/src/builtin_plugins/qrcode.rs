use crate::builtin_plugins::{BuiltinPlugin, PluginConfigOption};

pub fn plugin() -> BuiltinPlugin {
    BuiltinPlugin {
        id: "text-to-qrcode".to_string(),
        name: "文本转二维码".to_string(),
        description: "将输入的文本转换为二维码图片".to_string(),
        category: "工具".to_string(),
        config: vec![PluginConfigOption {
            name: "text".to_string(),
            label: "文本内容".to_string(),
            input_type: Some("text".to_string()),
            options: vec![],
            default: "".to_string(),
        }],
        code: r#"(async function() {
    const text = config.text;
    if (!text) throw new Error('请输入文本内容');
    const imageDataUrl = await generateQrCode(text);
    return { type: 'image', imageDataUrl: imageDataUrl, title: '二维码' };
})()"#
            .to_string(),
        ..Default::default()
    }
}
