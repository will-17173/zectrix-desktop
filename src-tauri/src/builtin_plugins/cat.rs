use crate::builtin_plugins::BuiltinPlugin;

pub fn plugin() -> BuiltinPlugin {
    BuiltinPlugin {
        id: "cat-random".to_string(),
        name: "随机显示猫猫".to_string(),
        description: "随机获取一张猫猫图片并推送到设备".to_string(),
        category: "图片".to_string(),
        code: r#"(async function() {
    const data = await fetchJson("https://cataas.com/cat?width=400&height=300&json=true");
    const imageDataUrl = await fetchBase64(data.url);
    return { type: "image", imageDataUrl: imageDataUrl, title: "随机猫猫" };
})()"#
            .to_string(),
        config: vec![],
        ..Default::default()
    }
}
