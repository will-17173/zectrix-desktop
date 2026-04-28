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
