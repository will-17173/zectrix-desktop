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
const content = data.content.replace(/，/g, '，\n').replace(/。/g, '。\n').replace(/！/g, '！\n').replace(/？/g, '？\n').replace(/；/g, '；\n');
const text = `「${data.origin}」\n${data.author}\n\n${content}`;
return { type: 'text', text: text, title: '随机古诗词', fontSize: 24 };
})()"#
                .to_string(),
        },
        BuiltinPlugin {
            id: "github-actions".to_string(),
            name: "GitHub Actions 监控".to_string(),
            description: "监控指定仓库的 GitHub Actions 运行状态".to_string(),
            config: vec![
                PluginConfigOption {
                    name: "token".to_string(),
                    label: "GitHub Token".to_string(),
                    input_type: Some("password".to_string()),
                    options: vec![],
                    default: "".to_string(),
                },
                PluginConfigOption {
                    name: "repo".to_string(),
                    label: "仓库 (owner/repo)".to_string(),
                    input_type: Some("text".to_string()),
                    options: vec![],
                    default: "".to_string(),
                },
                PluginConfigOption {
                    name: "limit".to_string(),
                    label: "显示数量".to_string(),
                    input_type: None,
                    options: vec![
                        PluginConfigOptionItem { value: "5".to_string(), label: "5 条".to_string() },
                        PluginConfigOptionItem { value: "10".to_string(), label: "10 条".to_string() },
                        PluginConfigOptionItem { value: "20".to_string(), label: "20 条".to_string() },
                    ],
                    default: "10".to_string(),
                },
            ],
            code: r#"(async function() {
const token = config.token;
const repo = config.repo;
const limit = parseInt(config.limit) || 10;

if (!token) throw new Error('请配置 GitHub Token');
if (!repo) throw new Error('请配置仓库 (owner/repo)');

const headers = {
    'Authorization': `Bearer ${token}`,
    'Accept': 'application/vnd.github+json',
    'X-GitHub-Api-Version': '2022-11-28',
    'User-Agent': 'Zectrix-Note-Plugin'
};

// 获取仓库信息
const repoInfo = await fetchJsonWithHeaders(`https://api.github.com/repos/${repo}`, headers);
const stars = repoInfo.stargazers_count || 0;
const forks = repoInfo.forks_count || 0;
const openIssues = repoInfo.open_issues_count || 0;
const lastPush = new Date(repoInfo.pushed_at).toLocaleString('zh-CN', { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' });

// 获取 Actions 运行记录
const actionsData = await fetchJsonWithHeaders(`https://api.github.com/repos/${repo}/actions/runs?per_page=${limit}`, headers);

const runs = actionsData.workflow_runs || [];

const repoSummary = `Stars: ${stars} | Forks: ${forks} | Issues: ${openIssues} | 最近推送: ${lastPush}`;

if (runs.length === 0) {
    return { type: 'text', text: `${repo}\n${repoSummary}\n\n暂无 Actions 运行记录`, title: 'GitHub Actions' };
}

const lines = runs.map(run => {
    const statusIcon = run.status === 'completed'
        ? (run.conclusion === 'success' ? '✓' : '✗')
        : (run.status === 'in_progress' ? '⏳' : '○');
    const statusText = run.status === 'completed'
        ? run.conclusion || 'unknown'
        : run.status;
    const branch = run.head_branch;
    const name = run.name.substring(0, 20);
    const time = new Date(run.created_at).toLocaleString('zh-CN', { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' });
    return `${statusIcon} ${name} [${branch}] ${statusText} ${time}`;
});

const text = `GitHub: ${repo}\n${repoSummary}\n\n${lines.join('\n')}`;
return { type: 'text', text: text, title: 'GitHub Actions', fontSize: 18 };
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
