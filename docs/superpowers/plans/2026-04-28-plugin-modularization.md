# 插件模块化 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将 `src-tauri/src/builtin_plugins.rs`（597 行单文件）拆分为目录模块，每个内置插件独立一个文件。

**Architecture:** 新建 `src-tauri/src/builtin_plugins/` 目录，原文件删除。`mod.rs` 保留所有公共类型定义和 `list_builtin_plugins()` / `find_builtin_plugin()`，每个插件文件只导出一个 `pub fn plugin() -> BuiltinPlugin`。对外接口和 `lib.rs` 的引用方式完全不变。

**Tech Stack:** Rust，无新依赖。

---

## 文件变更总览

| 操作 | 路径 |
|------|------|
| 删除 | `src-tauri/src/builtin_plugins.rs` |
| 新建 | `src-tauri/src/builtin_plugins/mod.rs` |
| 新建 | `src-tauri/src/builtin_plugins/comfyui.rs` |
| 新建 | `src-tauri/src/builtin_plugins/cat.rs` |
| 新建 | `src-tauri/src/builtin_plugins/dog.rs` |
| 新建 | `src-tauri/src/builtin_plugins/duck.rs` |
| 新建 | `src-tauri/src/builtin_plugins/waifu.rs` |
| 新建 | `src-tauri/src/builtin_plugins/qrcode.rs` |
| 新建 | `src-tauri/src/builtin_plugins/poetry.rs` |
| 新建 | `src-tauri/src/builtin_plugins/github_actions.rs` |
| 新建 | `src-tauri/src/builtin_plugins/bilibili.rs` |
| 新建 | `src-tauri/src/builtin_plugins/github_trending.rs` |

---

### Task 1: 创建目录模块结构（mod.rs + 类型定义）

**Files:**
- Create: `src-tauri/src/builtin_plugins/mod.rs`

- [ ] **Step 1: 新建目录和 mod.rs，迁移类型定义和模块声明**

新建文件 `src-tauri/src/builtin_plugins/mod.rs`，内容如下（类型定义从原文件完整复制，模块声明先列全，各 plugin 函数暂时 stub）：

```rust
use serde::Serialize;

mod bilibili;
mod cat;
mod comfyui;
mod dog;
mod duck;
mod github_actions;
mod github_trending;
mod poetry;
mod qrcode;
mod waifu;

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
        comfyui::plugin(),
        cat::plugin(),
        dog::plugin(),
        duck::plugin(),
        waifu::plugin(),
        qrcode::plugin(),
        poetry::plugin(),
        github_actions::plugin(),
        bilibili::plugin(),
        github_trending::plugin(),
    ]
}

pub fn find_builtin_plugin(id: &str) -> Option<BuiltinPlugin> {
    list_builtin_plugins()
        .into_iter()
        .find(|plugin| plugin.id == id)
}
```

- [ ] **Step 2: 删除原 builtin_plugins.rs**

```bash
rm src-tauri/src/builtin_plugins.rs
```

- [ ] **Step 3: 验证 mod.rs 路径被 lib.rs 正确识别（暂不编译，各子模块文件还未创建）**

此时编译会报 "file not found" 错误，这是预期的——继续后续 Task 创建各子模块文件后再编译。

---

### Task 2: 迁移 cat / dog / duck / waifu 插件

**Files:**
- Create: `src-tauri/src/builtin_plugins/cat.rs`
- Create: `src-tauri/src/builtin_plugins/dog.rs`
- Create: `src-tauri/src/builtin_plugins/duck.rs`
- Create: `src-tauri/src/builtin_plugins/waifu.rs`

- [ ] **Step 1: 创建 cat.rs**

```rust
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
```

- [ ] **Step 2: 创建 dog.rs**

```rust
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
```

- [ ] **Step 3: 创建 duck.rs**

```rust
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
```

- [ ] **Step 4: 创建 waifu.rs**

```rust
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
```

---

### Task 3: 迁移 qrcode / poetry 插件

**Files:**
- Create: `src-tauri/src/builtin_plugins/qrcode.rs`
- Create: `src-tauri/src/builtin_plugins/poetry.rs`

- [ ] **Step 1: 创建 qrcode.rs**

```rust
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
```

- [ ] **Step 2: 创建 poetry.rs**

```rust
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
```

---

### Task 4: 迁移 github_actions / bilibili / github_trending 插件

**Files:**
- Create: `src-tauri/src/builtin_plugins/github_actions.rs`
- Create: `src-tauri/src/builtin_plugins/bilibili.rs`
- Create: `src-tauri/src/builtin_plugins/github_trending.rs`

- [ ] **Step 1: 创建 github_actions.rs**

```rust
use crate::builtin_plugins::{BuiltinPlugin, PluginConfigOption};

pub fn plugin() -> BuiltinPlugin {
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
    }
}
```

- [ ] **Step 2: 创建 bilibili.rs**

```rust
use crate::builtin_plugins::{BuiltinPlugin, PluginConfigOption};

pub fn plugin() -> BuiltinPlugin {
    BuiltinPlugin {
        id: "bilibili-uploader-info".to_string(),
        name: "B站UP主信息".to_string(),
        description: "获取B站UP主的账号信息、粉丝数据、播放统计和最新视频".to_string(),
        category: "社交".to_string(),
        config: vec![PluginConfigOption {
            name: "userId".to_string(),
            label: "用户ID".to_string(),
            input_type: Some("text".to_string()),
            options: vec![],
            default: "".to_string(),
        }],
        code: r#"(async function() {
    const userId = config.userId;
    if (!userId) throw new Error('请输入用户ID');

    const headers = {
        'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36',
        'Referer': 'https://www.bilibili.com/'
    };

    const cardInfo = await fetchJsonWithHeaders('https://api.bilibili.com/x/web-interface/card?mid=' + userId + '&photo=true', headers);
    if (cardInfo.code !== 0) throw new Error('获取用户信息失败: ' + cardInfo.message);
    const name = cardInfo.data.card.name;
    const mid = cardInfo.data.card.mid;
    const sign = cardInfo.data.card.sign || '暂无签名';

    await sleep(2000);

    const relationStat = await fetchJsonWithHeaders('https://api.bilibili.com/x/relation/stat?vmid=' + userId, headers);
    const following = relationStat.data?.following || 0;
    const follower = relationStat.data?.follower || 0;

    await sleep(2000);

    const upstat = await fetchJsonWithHeaders('https://api.bilibili.com/x/space/upstat?mid=' + userId, headers);
    const archiveView = upstat.data?.archive?.view || 0;
    const likes = upstat.data?.likes || 0;

    await sleep(2000);

    const arcSearch = await fetchJsonWithHeaders('https://api.bilibili.com/x/space/arc/search?mid=' + userId + '&pn=1&ps=20', headers);
    if (arcSearch.code !== 0) {
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

    const text = name + '(' + mid + ')' + '\n' +
        sign + '\n' +
        '关注: ' + following + ' 粉丝: ' + follower + '\n' +
        '总播放: ' + archiveView + ' 总点赞: ' + likes + '\n' +
        '最新视频:\n' + latestVideoText;

    return { type: 'text', text: text, title: 'B站UP主信息', fontSize: 20 };
})()"#
            .to_string(),
        ..Default::default()
    }
}
```

- [ ] **Step 3: 创建 github_trending.rs**

```rust
use crate::builtin_plugins::{BuiltinPlugin, PluginConfigOption, PluginConfigOptionItem};

pub fn plugin() -> BuiltinPlugin {
    BuiltinPlugin {
        id: "github-trending".to_string(),
        name: "GitHub Trending".to_string(),
        description: "获取 GitHub 上当前最热门的项目排行榜".to_string(),
        category: "编程".to_string(),
        config: vec![
            PluginConfigOption {
                name: "language".to_string(),
                label: "编程语言".to_string(),
                input_type: None,
                options: vec![
                    PluginConfigOptionItem { value: "all".to_string(), label: "全部".to_string() },
                    PluginConfigOptionItem { value: "javascript".to_string(), label: "JavaScript".to_string() },
                    PluginConfigOptionItem { value: "typescript".to_string(), label: "TypeScript".to_string() },
                    PluginConfigOptionItem { value: "python".to_string(), label: "Python".to_string() },
                    PluginConfigOptionItem { value: "java".to_string(), label: "Java".to_string() },
                    PluginConfigOptionItem { value: "go".to_string(), label: "Go".to_string() },
                    PluginConfigOptionItem { value: "rust".to_string(), label: "Rust".to_string() },
                    PluginConfigOptionItem { value: "c".to_string(), label: "C".to_string() },
                    PluginConfigOptionItem { value: "cpp".to_string(), label: "C++".to_string() },
                    PluginConfigOptionItem { value: "swift".to_string(), label: "Swift".to_string() },
                    PluginConfigOptionItem { value: "kotlin".to_string(), label: "Kotlin".to_string() },
                    PluginConfigOptionItem { value: "ruby".to_string(), label: "Ruby".to_string() },
                    PluginConfigOptionItem { value: "php".to_string(), label: "PHP".to_string() },
                ],
                default: "all".to_string(),
            },
            PluginConfigOption {
                name: "since".to_string(),
                label: "时间范围".to_string(),
                input_type: None,
                options: vec![
                    PluginConfigOptionItem { value: "daily".to_string(), label: "今日".to_string() },
                    PluginConfigOptionItem { value: "weekly".to_string(), label: "本周".to_string() },
                    PluginConfigOptionItem { value: "monthly".to_string(), label: "本月".to_string() },
                ],
                default: "daily".to_string(),
            },
        ],
        code: r#"(async function() {
    const language = config.language || 'all';
    const since = config.since || 'daily';

    const now = new Date();
    let days = since === 'daily' ? 1 : (since === 'weekly' ? 7 : 30);
    const dateThreshold = new Date(now - days * 24 * 60 * 60 * 1000);
    const dateStr = dateThreshold.toISOString().split('T')[0];

    let query = 'stars:>500 pushed:>' + dateStr;
    if (language !== 'all') {
        query += ' language:' + language;
    }

    const headers = {
        'Accept': 'application/vnd.github+json',
        'X-GitHub-Api-Version': '2022-11-28',
        'User-Agent': 'Zectrix-Note-Plugin'
    };

    const url = 'https://api.github.com/search/repositories?q=' + encodeURIComponent(query) + '&sort=stars&order=desc&per_page=10';
    const data = await fetchJsonWithHeaders(url, headers);

    if (!data.items || data.items.length === 0) {
        return { type: 'text', text: '暂无热门项目', title: 'GitHub Trending', fontSize: 18 };
    }

    const lines = data.items.map((repo) => {
        const name = repo.full_name;
        const stars = repo.stargazers_count;
        const lang = repo.language || '-';
        return name + ' | 星: ' + stars + ' | ' + lang;
    });

    const langText = language === 'all' ? '全部' : language;
    const sinceText = since === 'daily' ? '今日' : (since === 'weekly' ? '本周' : '本月');
    const text = 'GitHub Trending | ' + langText + ' | ' + sinceText + '\n\n' + lines.join('\n');
    return { type: 'text', text: text, title: 'GitHub Trending', fontSize: 18 };
})()"#
            .to_string(),
        ..Default::default()
    }
}
```

---

### Task 5: 迁移 comfyui 插件

**Files:**
- Create: `src-tauri/src/builtin_plugins/comfyui.rs`

- [ ] **Step 1: 创建 comfyui.rs**

```rust
use crate::builtin_plugins::{BuiltinPlugin, PluginConfigOption};

pub fn plugin() -> BuiltinPlugin {
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

    let workflow;
    try {
        workflow = JSON.parse(workflowStr);
    } catch (e) {
        throw new Error('工作流 JSON 格式错误: ' + e.message);
    }

    const nodeIds = Object.keys(workflow);
    if (nodeIds.length === 0) {
        throw new Error('工作流为空');
    }

    const firstNode = workflow[nodeIds[0]];
    if (!firstNode.class_type) {
        throw new Error('请使用 "Save (API Format)" 导出工作流');
    }

    if (!workflow[promptNodeId]) {
        throw new Error('找不到提示词节点 "' + promptNodeId + '"，可用节点: ' + nodeIds.join(', '));
    }
    if (!workflow[promptNodeId].inputs) {
        throw new Error('节点 ' + promptNodeId + ' 没有 inputs 字段');
    }

    workflow[promptNodeId].inputs[promptField] = prompt;

    if (randomizeSeed && workflow[seedNodeId] && workflow[seedNodeId].inputs) {
        workflow[seedNodeId].inputs[seedField] = Math.floor(Math.random() * 1000000000);
    }

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
    }
}
```

---

### Task 6: 编译验证并提交

**Files:**
- 所有新建文件已在 Task 1-5 完成

- [ ] **Step 1: 编译验证**

```bash
cargo build --manifest-path src-tauri/Cargo.toml 2>&1 | head -50
```

期望：编译成功，0 个 error。若有 warning 可忽略。

- [ ] **Step 2: 运行 Rust 测试**

```bash
cargo test --manifest-path src-tauri/Cargo.toml 2>&1 | tail -20
```

期望：所有测试通过（test result: ok）。

- [ ] **Step 3: 提交**

```bash
git add src-tauri/src/builtin_plugins/
git rm src-tauri/src/builtin_plugins.rs
git commit -m "refactor: split builtin_plugins into per-plugin modules"
```
