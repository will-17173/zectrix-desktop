use crate::builtin_plugins::BuiltinPlugin;

pub fn plugin() -> BuiltinPlugin {
    BuiltinPlugin {
        id: "poetry-random".to_string(),
        name: "随机古诗词".to_string(),
        description: "随机获取一首古诗词并推送到设备".to_string(),
        category: "文学".to_string(),
        config: vec![],
        code: r#"(async function() {
    const data = await fetchJson("https://v1.jinrishici.com/all.json");
    const content = data.content.replace(/，/g, '，\n').replace(/。/g, '。\n').replace(/！/g, '！\n').replace(/？/g, '？\n').replace(/；/g, '；\n');
    const text = '「' + data.origin + '」\n' + data.author + '\n\n' + content;
    return { type: 'text', text: text, title: '随机古诗词', fontSize: 24 };
})()"#
            .to_string(),
        ..Default::default()
    }
}
