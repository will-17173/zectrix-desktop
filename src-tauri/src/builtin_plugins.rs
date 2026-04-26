use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginConfigOptionItem {
    pub value: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginConfigOption {
    pub name: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_type: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub options: Vec<PluginConfigOptionItem>,
    pub default: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuiltinPlugin {
    pub id: String,
    pub name: String,
    pub description: String,
    pub code: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub config: Vec<PluginConfigOption>,
}

pub fn list_builtin_plugins() -> Vec<BuiltinPlugin> {
    vec![
        BuiltinPlugin {
            id: "cat-random".to_string(),
            name: "随机显示猫猫".to_string(),
            description: "随机获取一张猫猫图片并推送到设备".to_string(),
            code: r#"(async function() {
const data = await fetchJson("https://cataas.com/cat?width=400&height=300&json=true");
const imageDataUrl = await fetchBase64(data.url);
return { type: "image", imageDataUrl: imageDataUrl, title: "随机猫猫" };
})()"#
                .to_string(),
            config: vec![],
        },
        BuiltinPlugin {
            id: "dog-random".to_string(),
            name: "随机显示狗狗".to_string(),
            description: "随机获取一张狗狗图片并推送到设备".to_string(),
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
        },
        BuiltinPlugin {
            id: "duck-random".to_string(),
            name: "随机显示鸭子".to_string(),
            description: "随机获取一张鸭子图片并推送到设备".to_string(),
            code: r#"(async function() {
const data = await fetchJson("https://random-d.uk/api/random");
const imageDataUrl = await fetchBase64(data.url);
return { type: "image", imageDataUrl: imageDataUrl, title: "随机鸭子" };
})()"#
                .to_string(),
            config: vec![],
        },
        BuiltinPlugin {
            id: "waifu-random".to_string(),
            name: "随机显示 Waifu".to_string(),
            description: "随机获取一张动漫图片并推送到设备".to_string(),
            config: vec![
                PluginConfigOption {
                    name: "type".to_string(),
                    label: "分类".to_string(),
                    input_type: None,
                    options: vec![
                        PluginConfigOptionItem { value: "sfw".to_string(), label: "SFW".to_string() },
                        PluginConfigOptionItem { value: "nsfw".to_string(), label: "NSFW".to_string() },
                    ],
                    default: "sfw".to_string(),
                },
            ],
            code: r#"(async function() {
const type = config.type || 'sfw';
const sfwCategories = ['waifu', 'neko', 'shinobu', 'megumin', 'bully', 'cuddle', 'cry', 'hug', 'awoo', 'kiss', 'lick', 'pat', 'smug', 'bonk', 'yeet', 'blush', 'smile', 'wave', 'highfive', 'handhold', 'nom', 'bite', 'glomp', 'slap', 'kill', 'kick', 'happy', 'wink', 'poke', 'dance', 'cringe'];
const nsfwCategories = ['waifu', 'neko', 'trap', 'blowjob'];
const categories = type === 'sfw' ? sfwCategories : nsfwCategories;
const category = categories[Math.floor(Math.random() * categories.length)];
const data = await fetchJson(`https://api.waifu.pics/${type}/${category}`);
const url = data.url;
if (!url) throw new Error('未获取到图片');
const imageDataUrl = await fetchBase64(url);
return { type: 'image', imageDataUrl: imageDataUrl, title: '随机 Waifu' };
})()"#
                .to_string(),
        },
        BuiltinPlugin {
            id: "text-to-qrcode".to_string(),
            name: "文本转二维码".to_string(),
            description: "将输入的文本转换为二维码图片".to_string(),
            config: vec![
                PluginConfigOption {
                    name: "text".to_string(),
                    label: "文本内容".to_string(),
                    input_type: Some("text".to_string()),
                    options: vec![],
                    default: "".to_string(),
                },
            ],
            code: r#"(async function() {
const text = config.text;
if (!text) throw new Error('请输入文本内容');
const imageDataUrl = await generateQrCode(text);
return { type: 'image', imageDataUrl: imageDataUrl, title: '二维码' };
})()"#
                .to_string(),
        },
        BuiltinPlugin {
            id: "poetry-random".to_string(),
            name: "随机古诗词".to_string(),
            description: "随机获取一首古诗词并推送到设备".to_string(),
            config: vec![],
            code: r#"(async function() {
const data = await fetchJson("https://v1.jinrishici.com/all.json");
const text = `「${data.origin}」\n${data.author}\n\n${data.content}`;
return { type: 'textImage', text: text, title: '随机古诗词', style: { fontSize: 36 } };
})()"#
                .to_string(),
        },
    ]
}

pub fn find_builtin_plugin(id: &str) -> Option<BuiltinPlugin> {
    list_builtin_plugins()
        .into_iter()
        .find(|plugin| plugin.id == id)
}