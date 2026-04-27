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
    #[serde(default)]
    pub code: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub config: Vec<PluginConfigOption>,
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub supports_loop: bool,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub category: String,
}

impl Default for BuiltinPlugin {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            description: String::new(),
            code: String::new(),
            config: vec![],
            supports_loop: true,
            category: String::new(),
        }
    }
}

#[allow(dead_code)]
fn default_true() -> bool {
    true
}

fn is_true(value: &bool) -> bool {
    *value
}

pub fn list_builtin_plugins() -> Vec<BuiltinPlugin> {
    vec![
        BuiltinPlugin {
            id: "comfyui-image".to_string(),
            name: "ComfyUI 生图".to_string(),
            description: "调用 ComfyUI 生成图片并推送到设备".to_string(),
            category: "AI".to_string(),
            config: vec![
                PluginConfigOption {
                    name: "comfyuiUrl".to_string(),
                    label: "ComfyUI 地址".to_string(),
                    input_type: Some("text".to_string()),
                    options: vec![],
                    default: "http://127.0.0.1:8188".to_string(),
                },
                PluginConfigOption {
                    name: "workflow".to_string(),
                    label: "工作流 JSON".to_string(),
                    input_type: Some("textarea".to_string()),
                    options: vec![],
                    default: "".to_string(),
                },
                PluginConfigOption {
                    name: "promptNodeId".to_string(),
                    label: "提示词节点 ID".to_string(),
                    input_type: Some("text".to_string()),
                    options: vec![],
                    default: "6".to_string(),
                },
                PluginConfigOption {
                    name: "promptField".to_string(),
                    label: "提示词字段名".to_string(),
                    input_type: Some("text".to_string()),
                    options: vec![],
                    default: "text".to_string(),
                },
                PluginConfigOption {
                    name: "prompt".to_string(),
                    label: "提示词".to_string(),
                    input_type: Some("text".to_string()),
                    options: vec![],
                    default: "".to_string(),
                },
                PluginConfigOption {
                    name: "seedNodeId".to_string(),
                    label: "Seed 节点 ID".to_string(),
                    input_type: Some("text".to_string()),
                    options: vec![],
                    default: "3".to_string(),
                },
                PluginConfigOption {
                    name: "seedField".to_string(),
                    label: "Seed 字段名".to_string(),
                    input_type: Some("text".to_string()),
                    options: vec![],
                    default: "seed".to_string(),
                },
                PluginConfigOption {
                    name: "randomizeSeed".to_string(),
                    label: "随机 Seed".to_string(),
                    input_type: Some("checkbox".to_string()),
                    options: vec![],
                    default: "true".to_string(),
                },
            ],
            code: r#"(async function() {
    const comfyuiUrl = config.comfyuiUrl || 'http://127.0.0.1:8188';
    const workflowStr = config.workflow;
    const promptNodeId = config.promptNodeId || '6';
    const promptField = config.promptField || 'text';
    const prompt = config.prompt;
    const seedNodeId = config.seedNodeId || '3';
    const seedField = config.seedField || 'seed';
    const randomizeSeed = config.randomizeSeed !== 'false';

    if (!workflowStr || workflowStr.trim() === '') {
        throw new Error('请先点击「配置」按钮填写工作流 JSON');
    }
    if (!prompt || prompt.trim() === '') {
        throw new Error('请输入提示词');
    }

    // 解析工作流 JSON
    let workflow;
    try {
        workflow = JSON.parse(workflowStr);
    } catch (e) {
        throw new Error('工作流 JSON 格式错误: ' + e.message);
    }

    // 检查工作流结构
    const nodeIds = Object.keys(workflow);
    if (nodeIds.length === 0) {
        throw new Error('工作流为空');
    }

    // 检查是否是 API 格式
    const firstNode = workflow[nodeIds[0]];
    if (!firstNode.class_type) {
        throw new Error('请使用 "Save (API Format)" 导出工作流');
    }

    // 检查并替换提示词
    if (!workflow[promptNodeId]) {
        throw new Error('找不到提示词节点 "' + promptNodeId + '"，可用节点: ' + nodeIds.join(', '));
    }
    if (!workflow[promptNodeId].inputs) {
        throw new Error('节点 ' + promptNodeId + ' 没有 inputs 字段');
    }

    workflow[promptNodeId].inputs[promptField] = prompt;

    // 随机化 seed
    if (randomizeSeed && workflow[seedNodeId] && workflow[seedNodeId].inputs) {
        workflow[seedNodeId].inputs[seedField] = Math.floor(Math.random() * 1000000000);
    }

    // 提交工作流
    let result;
    try {
        result = await postJson(comfyuiUrl + '/prompt', JSON.stringify({ prompt: workflow }));
    } catch (e) {
        throw new Error('无法连接 ComfyUI (' + comfyuiUrl + ')');
    }

    const promptId = result.prompt_id;
    if (!promptId) {
        throw new Error('提交失败');
    }

    // 轮询等待生成完成
    let outputs = null;
    let retries = 0;
    const maxRetries = 300;

    while (retries < maxRetries) {
        await sleep(1000);
        retries++;

        const history = await fetchJson(comfyuiUrl + '/history/' + promptId);

        if (history[promptId]) {
            if (history[promptId].status && history[promptId].status.status_str === 'error') {
                throw new Error('生成失败');
            }

            if (history[promptId].outputs) {
                const out = history[promptId].outputs;
                const hasImages = Object.keys(out).some(k => out[k].images && out[k].images.length > 0);
                if (hasImages) {
                    outputs = out;
                    break;
                }
            }
        }
    }

    if (!outputs) {
        throw new Error('生成超时 (' + retries + '秒)');
    }

    // 提取图片
    let imageDataUrl = null;
    for (const nodeId of Object.keys(outputs)) {
        const nodeOutput = outputs[nodeId];
        if (nodeOutput.images && nodeOutput.images.length > 0) {
            const img = nodeOutput.images[0];
            let url = comfyuiUrl + '/view?filename=' + encodeURIComponent(img.filename) + '&type=' + (img.type || 'output');
            if (img.subfolder) url += '&subfolder=' + encodeURIComponent(img.subfolder);
            imageDataUrl = await fetchBase64(url);
            break;
        }
    }

    if (!imageDataUrl) {
        throw new Error('未找到图片输出');
    }

    return { type: 'image', imageDataUrl: imageDataUrl, title: 'ComfyUI 生图' };
})()"#
                .to_string(),
            supports_loop: false,
            ..Default::default()
        },
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
        },
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
        },
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
        },
        BuiltinPlugin {
            id: "waifu-random".to_string(),
            name: "随机显示 Waifu".to_string(),
            description: "随机获取一张动漫图片并推送到设备".to_string(),
            category: "图片".to_string(),
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
    const data = await fetchJson('https://api.waifu.pics/' + type + '/' + category);
    const url = data.url;
    if (!url) throw new Error('未获取到图片');
    const imageDataUrl = await fetchBase64(url);
    return { type: 'image', imageDataUrl: imageDataUrl, title: '随机 Waifu' };
})()"#
                .to_string(),
            ..Default::default()
        },
        BuiltinPlugin {
            id: "text-to-qrcode".to_string(),
            name: "文本转二维码".to_string(),
            description: "将输入的文本转换为二维码图片".to_string(),
            category: "工具".to_string(),
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
            ..Default::default()
        },
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
        },
        BuiltinPlugin {
            id: "github-actions".to_string(),
            name: "GitHub Actions 监控".to_string(),
            description: "监控指定仓库的 GitHub Actions 运行状态".to_string(),
            category: "编程".to_string(),
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

            ],
            code: r#"(async function() {
    const token = config.token;
    const repo = config.repo;
    const limit = 10;

    if (!token) throw new Error('请配置 GitHub Token');
    if (!repo) throw new Error('请配置仓库 (owner/repo)');

    const headers = {
        'Authorization': 'Bearer ' + token,
        'Accept': 'application/vnd.github+json',
        'X-GitHub-Api-Version': '2022-11-28',
        'User-Agent': 'Zectrix-Note-Plugin'
    };

    const repoInfo = await fetchJsonWithHeaders('https://api.github.com/repos/' + repo, headers);
    const stars = repoInfo.stargazers_count || 0;
    const forks = repoInfo.forks_count || 0;
    const openIssues = repoInfo.open_issues_count || 0;
    const lastPush = new Date(repoInfo.pushed_at).toLocaleString('zh-CN', { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' });

    const actionsData = await fetchJsonWithHeaders('https://api.github.com/repos/' + repo + '/actions/runs?per_page=' + limit, headers);
    const runs = actionsData.workflow_runs || [];

    const repoSummary = 'Stars: ' + stars + ' | Forks: ' + forks + ' | Issues: ' + openIssues + ' | 最近推送: ' + lastPush;

    if (runs.length === 0) {
        return { type: 'text', text: repo + '\n' + repoSummary + '\n\n暂无 Actions 运行记录', title: 'GitHub Actions' };
    }

    const lines = runs.map(run => {
        const statusIcon = run.status === 'completed'
            ? (run.conclusion === 'success' ? '✓' : '✗')
            : (run.status === 'in_progress' ? '⏳' : '○');
        const statusText = run.status === 'completed' ? run.conclusion || 'unknown' : run.status;
        const branch = run.head_branch;
        const name = run.name.substring(0, 20);
        const time = new Date(run.created_at).toLocaleString('zh-CN', { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' });
        return statusIcon + ' ' + name + ' [' + branch + '] ' + statusText + ' ' + time;
    });

    const text = 'GitHub: ' + repo + '\n' + repoSummary + '\n\n' + lines.join('\n');
    return { type: 'text', text: text, title: 'GitHub Actions', fontSize: 18 };
})()"#
                .to_string(),
            ..Default::default()
        },
        BuiltinPlugin {
            id: "bilibili-uploader-info".to_string(),
            name: "B站UP主信息".to_string(),
            description: "获取B站UP主的账号信息、粉丝数据、播放统计和最新视频".to_string(),
            category: "社交".to_string(),
            config: vec![
                PluginConfigOption {
                    name: "userId".to_string(),
                    label: "用户ID".to_string(),
                    input_type: Some("text".to_string()),
                    options: vec![],
                    default: "".to_string(),
                },
            ],
            code: r#"(async function() {
    const userId = config.userId;
    if (!userId) throw new Error('请输入用户ID');

    const headers = {
        'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36',
        'Referer': 'https://www.bilibili.com/'
    };

    // 获取用户基本信息 (使用 web-interface/card 接口)
    const cardInfo = await fetchJsonWithHeaders('https://api.bilibili.com/x/web-interface/card?mid=' + userId + '&photo=true', headers);
    if (cardInfo.code !== 0) throw new Error('获取用户信息失败: ' + cardInfo.message);
    const name = cardInfo.data.card.name;
    const mid = cardInfo.data.card.mid;
    const sign = cardInfo.data.card.sign || '暂无签名';

    await sleep(2000);

    // 获取关系统计
    const relationStat = await fetchJsonWithHeaders('https://api.bilibili.com/x/relation/stat?vmid=' + userId, headers);
    const following = relationStat.data?.following || 0;
    const follower = relationStat.data?.follower || 0;

    await sleep(2000);

    // 获取UP主统计
    const upstat = await fetchJsonWithHeaders('https://api.bilibili.com/x/space/upstat?mid=' + userId, headers);
    const archiveView = upstat.data?.archive?.view || 0;
    const likes = upstat.data?.likes || 0;

    await sleep(2000);

    // 获取最新视频
    const arcSearch = await fetchJsonWithHeaders('https://api.bilibili.com/x/space/arc/search?mid=' + userId + '&pn=1&ps=20', headers);
    if (arcSearch.code !== 0) {
        // 视频接口可能被限制，跳过视频信息
        const text = name + '(' + mid + ')' + '\n' +
            sign + '\n' +
            '关注: ' + following + ' 粉丝: ' + follower + '\n' +
            '总播放: ' + archiveView + ' 总点赞: ' + likes + '\n' +
            '最新视频: 暂无数据';
        return { type: 'text', text: text, title: 'B站UP主信息', fontSize: 20 };
    }
    const vlist = arcSearch.data?.list?.vlist || [];
    let latestVideoText = '暂无视频';
    if (vlist.length > 0) {
        const latestVideo = vlist[0];
        latestVideoText = latestVideo.title + '\n播放: ' + latestVideo.play + ' 评论: ' + latestVideo.comment;
    }

    // 格式化输出
    const text = name + '(' + mid + ')' + '\n' +
        sign + '\n' +
        '关注: ' + following + ' 粉丝: ' + follower + '\n' +
        '总播放: ' + archiveView + ' 总点赞: ' + likes + '\n' +
        '最新视频:\n' + latestVideoText;

    return { type: 'text', text: text, title: 'B站UP主信息', fontSize: 20 };
})()"#
                .to_string(),
            ..Default::default()
        },
    ]
}

pub fn find_builtin_plugin(id: &str) -> Option<BuiltinPlugin> {
    list_builtin_plugins()
        .into_iter()
        .find(|plugin| plugin.id == id)
}