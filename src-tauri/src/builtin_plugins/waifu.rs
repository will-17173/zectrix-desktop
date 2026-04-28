use crate::builtin_plugins::{BuiltinPlugin, PluginConfigOption, PluginConfigOptionItem};

pub fn plugin() -> BuiltinPlugin {
    BuiltinPlugin {
        id: "waifu-random".to_string(),
        name: "随机显示 Waifu".to_string(),
        description: "随机获取一张动漫图片并推送到设备".to_string(),
        category: "图片".to_string(),
        config: vec![PluginConfigOption {
            name: "type".to_string(),
            label: "分类".to_string(),
            input_type: None,
            options: vec![
                PluginConfigOptionItem {
                    value: "sfw".to_string(),
                    label: "SFW".to_string(),
                },
                PluginConfigOptionItem {
                    value: "nsfw".to_string(),
                    label: "NSFW".to_string(),
                },
            ],
            default: "sfw".to_string(),
        }],
        code: r#"(async function() {
    const type = config.type || 'sfw';
    const sfwCategories = ['waifu', 'neko', 'shinobu', 'megumin', 'bully', 'cuddle', 'cry', 'hug', 'awoo', 'kiss', 'lick', 'pat', 'smug', 'bonk', 'yeet', 'blush', 'smile', 'wave', 'highfive', 'handhold', 'nom', 'bite', 'glomp', 'slap', 'kill', 'kick', 'happy', 'wink', 'poke', 'dance', 'cringe'];
    const nsfwCategories = ['waifu', 'neko', 'trap', 'blowjob'];
    const categories = type === 'sfw' ? sfwCategories : nsfwCategories;
    const category = categories[Math.floor(Math.random() * categories.length)];
    const data = await fetchJson('https://api.waifu.pics/' + type + '/' + category);
    const url = data.url;
    if (!url) throw new Error('未获取到图片');
    const imageDataUrl = await fetchBase64(url);
    return { type: 'image', imageDataUrl: imageDataUrl, title: '随机 Waifu' };
})()"#
            .to_string(),
        ..Default::default()
    }
}
