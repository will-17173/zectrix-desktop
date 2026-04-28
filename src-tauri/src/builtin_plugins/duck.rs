use crate::builtin_plugins::BuiltinPlugin;

pub fn plugin() -> BuiltinPlugin {
    BuiltinPlugin {
        id: "duck-random".to_string(),
        name: "随机显示鸭子".to_string(),
        description: "随机获取一张鸭子图片并推送到设备".to_string(),
        category: "图片".to_string(),
        code: r#"(async function() {
    const data = await fetchJson("https://random-d.uk/api/random");
    const imageDataUrl = await fetchBase64(data.url);
    return { type: "image", imageDataUrl: imageDataUrl, title: "随机鸭子" };
})()"#
            .to_string(),
        config: vec![],
        ..Default::default()
    }
}
