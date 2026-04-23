use serde::{Deserialize, Serialize};

const GITHUB_REPO: &str = "will-17173/zectrix-desktop";

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateInfo {
    current_version: String,
    latest_version: String,
    has_update: bool,
    release_url: String,
    release_notes: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    html_url: String,
    body: Option<String>,
}

#[tauri::command]
pub async fn check_for_update() -> Result<UpdateInfo, String> {
    let current_version = env!("CARGO_PKG_VERSION");

    let client = reqwest::Client::builder()
        .user_agent("Zectrix-Desktop-Updater")
        .build()
        .map_err(|e| e.to_string())?;

    let url = format!(
        "https://api.github.com/repos/{}/releases/latest",
        GITHUB_REPO
    );

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("网络请求失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("GitHub API 返回错误: {}", response.status()));
    }

    let release: GitHubRelease = response
        .json()
        .await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    // 去掉 tag_name 的 'v' 前缀进行比较
    let latest_version = release.tag_name.trim_start_matches('v');

    let has_update = compare_versions(current_version, latest_version)?;

    Ok(UpdateInfo {
        current_version: current_version.to_string(),
        latest_version: latest_version.to_string(),
        has_update,
        release_url: release.html_url,
        release_notes: release.body,
    })
}

fn compare_versions(current: &str, latest: &str) -> Result<bool, String> {
    let current_parts: Vec<u32> = current
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();

    let latest_parts: Vec<u32> = latest
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();

    if current_parts.is_empty() || latest_parts.is_empty() {
        return Err("无效的版本格式".to_string());
    }

    // 比较每个版本号部分
    for i in 0..std::cmp::max(current_parts.len(), latest_parts.len()) {
        let current_val = current_parts.get(i).copied().unwrap_or(0);
        let latest_val = latest_parts.get(i).copied().unwrap_or(0);

        if latest_val > current_val {
            return Ok(true);
        }
        if latest_val < current_val {
            return Ok(false);
        }
    }

    // 版本相同
    Ok(false)
}

#[tauri::command]
pub fn get_current_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}