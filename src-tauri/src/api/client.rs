use serde::{Deserialize, Deserializer, Serialize};

const BASE_URL: &str = "https://cloud.zectrix.com/open/v1";

fn deserialize_null_string_as_default<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

fn deserialize_optional_string_as_i32<'de, D>(deserializer: D) -> Result<Option<i32>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<serde_json::Value>::deserialize(deserializer)?;
    match opt {
        None => Ok(None),
        Some(serde_json::Value::Null) => Ok(None),
        Some(serde_json::Value::String(s)) => s.parse::<i32>().map(Some).map_err(|_| {
            serde::de::Error::custom(format!("Failed to parse '{}' as i32", s))
        }),
        Some(serde_json::Value::Number(n)) => n.as_i64().map(|v| Some(v as i32)).ok_or_else(|| {
            serde::de::Error::custom("Number out of i32 range")
        }),
        Some(other) => Err(serde::de::Error::custom(format!(
            "Expected string or number, got {}",
            other
        ))),
    }
}

#[derive(Debug, Deserialize)]
pub struct ApiResponse<T> {
    pub code: i32,
    pub data: T,
    #[allow(dead_code)]
    pub msg: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiDevice {
    pub device_id: String,
    pub alias: String,
    pub board: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct ApiTodo {
    pub id: i64,
    pub title: String,
    #[serde(deserialize_with = "deserialize_null_string_as_default")]
    pub description: String,
    pub due_date: Option<String>,
    pub due_time: Option<String>,
    #[serde(default)]
    pub repeat_type: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_string_as_i32")]
    pub repeat_weekday: Option<i32>,
    #[serde(default, deserialize_with = "deserialize_optional_string_as_i32")]
    pub repeat_month: Option<i32>,
    #[serde(default, deserialize_with = "deserialize_optional_string_as_i32")]
    pub repeat_day: Option<i32>,
    pub status: i32,
    pub priority: i32,
    #[serde(default)]
    pub completed: bool,
    pub device_id: Option<String>,
    pub device_name: Option<String>,
    pub create_date: Option<String>,
    pub update_date: Option<serde_json::Value>,
}

pub async fn fetch_devices(api_key: &str) -> anyhow::Result<Vec<ApiDevice>> {
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{BASE_URL}/devices"))
        .header("X-API-Key", api_key)
        .send()
        .await?;

    let status = resp.status();
    if status == reqwest::StatusCode::UNAUTHORIZED {
        anyhow::bail!("API Key 无效");
    }
    if !status.is_success() {
        anyhow::bail!("API 请求失败: {status}");
    }

    let body: ApiResponse<Vec<ApiDevice>> = resp.json().await?;
    Ok(body.data)
}

pub async fn validate_device(api_key: &str, device_id: &str) -> anyhow::Result<ApiDevice> {
    let devices = fetch_devices(api_key).await?;
    devices
        .into_iter()
        .find(|d| d.device_id.eq_ignore_ascii_case(device_id))
        .ok_or_else(|| anyhow::anyhow!("设备 {device_id} 未找到"))
}

pub async fn fetch_todos(api_key: &str) -> anyhow::Result<Vec<ApiTodo>> {
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{BASE_URL}/todos"))
        .header("X-API-Key", api_key)
        .send()
        .await?;

    let status = resp.status();
    if !status.is_success() {
        anyhow::bail!("获取待办列表失败: {status}");
    }

    let body: ApiResponse<Vec<ApiTodo>> = resp.json().await?;
    Ok(body.data)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTodoBody {
    pub title: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repeat_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repeat_weekday: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repeat_month: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repeat_day: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_id: Option<String>,
}

pub async fn create_cloud_todo(api_key: &str, body: &CreateTodoBody) -> anyhow::Result<ApiTodo> {
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{BASE_URL}/todos"))
        .header("X-API-Key", api_key)
        .json(body)
        .send()
        .await?;

    let status = resp.status();
    if !status.is_success() {
        anyhow::bail!("创建云端待办失败: {status}");
    }

    let api_resp: ApiResponse<ApiTodo> = resp.json().await?;
    Ok(api_resp.data)
}

pub async fn update_cloud_todo(
    api_key: &str,
    id: i64,
    body: &CreateTodoBody,
) -> anyhow::Result<ApiTodo> {
    let client = reqwest::Client::new();
    let resp = client
        .put(format!("{BASE_URL}/todos/{id}"))
        .header("X-API-Key", api_key)
        .json(body)
        .send()
        .await?;

    let status = resp.status();
    if !status.is_success() {
        anyhow::bail!("更新云端待办失败: {status}");
    }

    let api_resp: ApiResponse<ApiTodo> = resp.json().await?;
    Ok(api_resp.data)
}

pub async fn complete_cloud_todo(api_key: &str, id: i64) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let resp = client
        .put(format!("{BASE_URL}/todos/{id}/complete"))
        .header("X-API-Key", api_key)
        .send()
        .await?;

    let status = resp.status();
    if !status.is_success() {
        anyhow::bail!("完成云端待办失败: {status}");
    }

    Ok(())
}

fn is_successful_todo_delete(status: reqwest::StatusCode) -> bool {
    status.is_success() || status == reqwest::StatusCode::NOT_FOUND
}

pub async fn delete_cloud_todo(api_key: &str, id: i64) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let resp = client
        .delete(format!("{BASE_URL}/todos/{id}"))
        .header("X-API-Key", api_key)
        .send()
        .await?;

    let status = resp.status();
    if !is_successful_todo_delete(status) {
        anyhow::bail!("删除云端待办失败: {status}");
    }

    Ok(())
}

#[derive(Debug, Serialize)]
pub struct StructuredTextBody {
    pub title: String,
    pub body: String,
    #[serde(skip_serializing_if = "Option::is_none", rename = "pageId")]
    pub page_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PlainTextBody {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none", rename = "fontSize")]
    pub font_size: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "pageId")]
    pub page_id: Option<String>,
}

pub async fn push_structured_text(
    api_key: &str,
    device_id: &str,
    title: &str,
    body: &str,
    page_id: Option<u32>,
) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let url = format!("{BASE_URL}/devices/{device_id}/display/structured-text");
    let payload = StructuredTextBody {
        title: title.to_string(),
        body: body.to_string(),
        page_id: page_id.map(|p| p.to_string()),
    };
    let resp = client
        .post(&url)
        .header("X-API-Key", api_key)
        .json(&payload)
        .send()
        .await?;

    let status = resp.status();
    if !status.is_success() {
        anyhow::bail!("推送失败: {status}");
    }

    let api_resp: ApiResponse<serde_json::Value> = resp.json().await?;
    if api_resp.code != 0 {
        anyhow::bail!(
            "推送失败: {}",
            api_resp.msg.unwrap_or_else(|| "未知错误".to_string())
        );
    }

    Ok(())
}

pub async fn push_text(
    api_key: &str,
    device_id: &str,
    text: &str,
    font_size: Option<u32>,
    page_id: Option<u32>,
) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let url = format!("{BASE_URL}/devices/{device_id}/display/text");
    let payload = PlainTextBody {
        text: text.to_string(),
        font_size,
        page_id: page_id.map(|p| p.to_string()),
    };
    let resp = client
        .post(&url)
        .header("X-API-Key", api_key)
        .json(&payload)
        .send()
        .await?;

    let status = resp.status();
    if !status.is_success() {
        anyhow::bail!("推送文本失败: {status}");
    }

    let api_resp: ApiResponse<serde_json::Value> = resp.json().await?;
    if api_resp.code != 0 {
        anyhow::bail!(
            "推送文本失败: {}",
            api_resp.msg.unwrap_or_else(|| "未知错误".to_string())
        );
    }

    Ok(())
}

pub async fn push_image(
    api_key: &str,
    device_id: &str,
    image_bytes: Vec<u8>,
    page_id: u32,
) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let url = format!("{BASE_URL}/devices/{device_id}/display/image");

    let part = reqwest::multipart::Part::bytes(image_bytes)
        .file_name("image.png")
        .mime_str("image/png")?;
    let form = reqwest::multipart::Form::new()
        .part("images", part)
        .text("pageId", page_id.to_string());

    let resp = client
        .post(&url)
        .header("X-API-Key", api_key)
        .multipart(form)
        .send()
        .await?;

    let status = resp.status();
    if !status.is_success() {
        anyhow::bail!("推送图片失败: {status}");
    }

    let api_resp: ApiResponse<serde_json::Value> = resp.json().await?;
    if api_resp.code != 0 {
        anyhow::bail!(
            "推送图片失败: {}",
            api_resp.msg.unwrap_or_else(|| "未知错误".to_string())
        );
    }

    Ok(())
}

pub async fn delete_page(
    api_key: &str,
    device_id: &str,
    page_id: Option<u32>,
) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let url = if let Some(pid) = page_id {
        format!("{BASE_URL}/devices/{device_id}/display/pages/{pid}")
    } else {
        format!("{BASE_URL}/devices/{device_id}/display/pages")
    };

    let resp = client
        .delete(&url)
        .header("X-API-Key", api_key)
        .send()
        .await?;

    let status = resp.status();
    if !status.is_success() {
        anyhow::bail!("删除页面失败: {status}");
    }

    let api_resp: ApiResponse<serde_json::Value> = resp.json().await?;
    if api_resp.code != 0 {
        anyhow::bail!(
            "删除页面失败: {}",
            api_resp.msg.unwrap_or_else(|| "未知错误".to_string())
        );
    }

    Ok(())
}

// ── Error types ─────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug)]
pub enum ApiClientError {
    InvalidApiKey,
    DeviceNotFound,
    Offline,
    PushFailed(String),
    LocalStorage(String),
    ImageProcessing(String),
}

impl ApiClientError {
    #[allow(dead_code)]
    pub fn user_message(&self) -> String {
        match self {
            Self::InvalidApiKey => "API Key 无效，请重新配置".into(),
            Self::DeviceNotFound => "设备不存在，请检查 MAC 地址".into(),
            Self::Offline => "网络错误，当前处于离线状态".into(),
            Self::PushFailed(_) => "推送失败，请稍后重试".into(),
            Self::LocalStorage(_) => "本地数据读写失败，建议重启应用".into(),
            Self::ImageProcessing(_) => "图片处理失败，请检查图片格式".into(),
        }
    }
}

impl std::fmt::Display for ApiClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.user_message())
    }
}

impl std::error::Error for ApiClientError {}

#[allow(dead_code)]
pub fn map_api_error(status_code: u16) -> ApiClientError {
    match status_code {
        401 => ApiClientError::InvalidApiKey,
        404 => ApiClientError::DeviceNotFound,
        _ => ApiClientError::PushFailed(format!("HTTP {status_code}")),
    }
}

#[allow(dead_code)]
pub fn map_network_error() -> ApiClientError {
    ApiClientError::Offline
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_http_status_and_network_failures_to_user_messages() {
        assert_eq!(
            map_api_error(401).user_message(),
            "API Key 无效，请重新配置"
        );
        assert_eq!(
            map_api_error(404).user_message(),
            "设备不存在，请检查 MAC 地址"
        );
        assert_eq!(
            map_network_error().user_message(),
            "网络错误，当前处于离线状态"
        );
    }

    #[test]
    fn todo_delete_treats_not_found_as_success() {
        assert!(is_successful_todo_delete(reqwest::StatusCode::NO_CONTENT));
        assert!(is_successful_todo_delete(reqwest::StatusCode::NOT_FOUND));
        assert!(!is_successful_todo_delete(reqwest::StatusCode::INTERNAL_SERVER_ERROR));
    }
}
