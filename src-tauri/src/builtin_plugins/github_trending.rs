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
