use crate::builtin_plugins::BuiltinPlugin;

pub fn plugin() -> BuiltinPlugin {
    BuiltinPlugin {
        id: "dog-random".to_string(),
        name: "随机显示狗狗".to_string(),
        description: "随机获取一张狗狗图片并推送到设备".to_string(),
        category: "图片".to_string(),
        code: r#"(async function() {
    let data;
    let retries = 0;
    while (retries < 10) {
        data = await fetchJson("https://random.dog/woof.json");
        if (data.url.match(/\.(jpg|jpeg|png|gif)$/i)) {
            break;
        }
        retries++;
    }
    const imageDataUrl = await fetchBase64(data.url);
    return { type: "image", imageDataUrl: imageDataUrl, title: "随机狗狗" };
})()"#
            .to_string(),
        config: vec![],
        ..Default::default()
    }
}
