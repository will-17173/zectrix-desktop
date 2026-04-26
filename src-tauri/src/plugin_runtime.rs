use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use reqwest::blocking::Client;
use rquickjs::promise::MaybePromise;
use rquickjs::{
    Context, Ctx, Error as JsError, Exception, Function, Runtime, String as JsString, Value,
};
use std::collections::HashMap;
use std::time::Duration;

use qrcode::QrCode;

pub async fn run_plugin_code(
    code: &str,
    config: HashMap<String, String>,
) -> anyhow::Result<serde_json::Value> {
    let code = code.to_owned();
    let execution = tokio::task::spawn_blocking(move || run_plugin_code_blocking(&code, config));

    match tokio::time::timeout(Duration::from_secs(360), execution).await {
        Ok(joined) => match joined {
            Ok(result) => result,
            Err(error) => anyhow::bail!("插件执行失败: {error}"),
        },
        Err(_) => anyhow::bail!("插件执行超时"),
    }
}

fn run_plugin_code_blocking(
    code: &str,
    config: HashMap<String, String>,
) -> anyhow::Result<serde_json::Value> {
    let runtime = Runtime::new()?;
    let context = Context::full(&runtime)?;

    context.with(|ctx| {
        install_helpers(ctx.clone())?;

        // 注入 config 对象到 JS 全局
        let config_value = rquickjs_serde::to_value(ctx.clone(), &config)
            .map_err(|error| Exception::throw_message(&ctx, &error.to_string()))?;
        ctx.globals().set("config", config_value)?;

        let maybe_promise = ctx
            .eval::<MaybePromise<'_>, _>(code)
            .map_err(|error| quickjs_error_to_anyhow(&ctx, error))?;
        let value = maybe_promise
            .finish::<Value<'_>>()
            .map_err(|error| quickjs_error_to_anyhow(&ctx, error))?;

        Ok(rquickjs_serde::from_value(value)?)
    })
}

fn install_helpers<'js>(ctx: Ctx<'js>) -> anyhow::Result<()> {
    let globals = ctx.globals();

    globals.set("echoJson", Function::new(ctx.clone(), echo_json)?)?;
    globals.set("fetchJson", Function::new(ctx.clone(), fetch_json_js)?)?;
    globals.set("fetchJsonWithHeaders", Function::new(ctx.clone(), fetch_json_with_headers_js)?)?;
    globals.set("fetchText", Function::new(ctx.clone(), fetch_text_js)?)?;
    globals.set("fetchBase64", Function::new(ctx.clone(), fetch_base64_js)?)?;
    globals.set("fetchBase64WithHeaders", Function::new(ctx.clone(), fetch_base64_with_headers_js)?)?;
    globals.set("generateQrCode", Function::new(ctx.clone(), generate_qrcode_js)?)?;
    globals.set("postJson", Function::new(ctx.clone(), post_json_js)?)?;
    globals.set("postJsonWithHeaders", Function::new(ctx.clone(), post_json_with_headers_js)?)?;
    globals.set("sleep", Function::new(ctx.clone(), sleep_js)?)?;

    Ok(())
}

fn echo_json<'js>(value: Value<'js>) -> Value<'js> {
    value
}

fn fetch_json_js<'js>(ctx: Ctx<'js>, url: String) -> rquickjs::Result<Value<'js>> {
    let json = fetch_json_blocking(&url, None)
        .map_err(|error| Exception::throw_message(&ctx, &error.to_string()))?;

    rquickjs_serde::to_value(ctx.clone(), json)
        .map_err(|error| Exception::throw_message(&ctx, &error.to_string()))
}

fn fetch_json_with_headers_js<'js>(ctx: Ctx<'js>, url: String, headers: HashMap<String, String>) -> rquickjs::Result<Value<'js>> {
    let json = fetch_json_blocking(&url, Some(headers))
        .map_err(|error| Exception::throw_message(&ctx, &error.to_string()))?;

    rquickjs_serde::to_value(ctx.clone(), json)
        .map_err(|error| Exception::throw_message(&ctx, &error.to_string()))
}

fn fetch_text_js<'js>(ctx: Ctx<'js>, url: String) -> rquickjs::Result<String> {
    fetch_text_blocking(&url)
        .map_err(|error| Exception::throw_message(&ctx, &error.to_string()))
}

fn fetch_json_blocking(url: &str, headers: Option<HashMap<String, String>>) -> anyhow::Result<serde_json::Value> {
    let mut request = blocking_client()?.get(url);

    if let Some(h) = headers {
        for (key, value) in h {
            request = request.header(&key, &value);
        }
    }

    let response = request.send()?;
    let status = response.status();

    if !status.is_success() {
        anyhow::bail!("HTTP 请求失败: {status}");
    }

    Ok(response.json()?)
}

fn fetch_text_blocking(url: &str) -> anyhow::Result<String> {
    let response = blocking_client()?.get(url).send()?;
    let status = response.status();

    if !status.is_success() {
        anyhow::bail!("HTTP 请求失败: {status}");
    }

    Ok(response.text()?)
}

fn fetch_base64_js<'js>(ctx: Ctx<'js>, url: String) -> rquickjs::Result<String> {
    fetch_base64_blocking(&url, None)
        .map_err(|error| Exception::throw_message(&ctx, &error.to_string()))
}

fn fetch_base64_with_headers_js<'js>(ctx: Ctx<'js>, url: String, headers: HashMap<String, String>) -> rquickjs::Result<String> {
    fetch_base64_blocking(&url, Some(headers))
        .map_err(|error| Exception::throw_message(&ctx, &error.to_string()))
}

fn fetch_base64_blocking(url: &str, headers: Option<HashMap<String, String>>) -> anyhow::Result<String> {
    let mut request = blocking_client()?.get(url);

    if let Some(h) = headers {
        for (key, value) in h {
            request = request.header(&key, &value);
        }
    }

    let response = request.send()?;
    let status = response.status();

    if !status.is_success() {
        anyhow::bail!("HTTP 请求失败: {status}");
    }

    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("image/jpeg")
        .to_string();

    let bytes = response.bytes()?;
    let b64 = BASE64.encode(bytes);
    Ok(format!("data:{content_type};base64,{b64}"))
}

fn generate_qrcode_js<'js>(ctx: Ctx<'js>, text: String) -> rquickjs::Result<String> {
    generate_qrcode_blocking(&text)
        .map_err(|error| Exception::throw_message(&ctx, &error.to_string()))
}

fn generate_qrcode_blocking(text: &str) -> anyhow::Result<String> {
    let code = QrCode::new(text)?;
    let image = code.render::<image::Luma<u8>>().build();

    let mut buffer = Vec::new();
    image
        .write_to(&mut std::io::Cursor::new(&mut buffer), image::ImageFormat::Png)
        .map_err(|e| anyhow::anyhow!("Failed to encode PNG: {}", e))?;

    let b64 = BASE64.encode(&buffer);
    Ok(format!("data:image/png;base64,{b64}"))
}

fn post_json_js<'js>(ctx: Ctx<'js>, url: String, body_str: String) -> rquickjs::Result<Value<'js>> {
    let body: serde_json::Value = serde_json::from_str(&body_str)
        .map_err(|error| Exception::throw_message(&ctx, &format!("JSON 解析失败: {}", error)))?;
    let json = post_json_blocking(&url, body, None)
        .map_err(|error| Exception::throw_message(&ctx, &error.to_string()))?;

    rquickjs_serde::to_value(ctx.clone(), json)
        .map_err(|error| Exception::throw_message(&ctx, &error.to_string()))
}

fn post_json_with_headers_js<'js>(ctx: Ctx<'js>, url: String, body_str: String, headers: HashMap<String, String>) -> rquickjs::Result<Value<'js>> {
    let body: serde_json::Value = serde_json::from_str(&body_str)
        .map_err(|error| Exception::throw_message(&ctx, &format!("JSON 解析失败: {}", error)))?;
    let json = post_json_blocking(&url, body, Some(headers))
        .map_err(|error| Exception::throw_message(&ctx, &error.to_string()))?;

    rquickjs_serde::to_value(ctx.clone(), json)
        .map_err(|error| Exception::throw_message(&ctx, &error.to_string()))
}

fn post_json_blocking(url: &str, body: serde_json::Value, headers: Option<HashMap<String, String>>) -> anyhow::Result<serde_json::Value> {
    let mut request = blocking_client()?.post(url).json(&body);

    if let Some(h) = headers {
        for (key, value) in h {
            request = request.header(&key, &value);
        }
    }

    let response = request.send()?;
    let status = response.status();

    if !status.is_success() {
        let error_text = response.text()?;
        anyhow::bail!("POST 请求失败 ({}): {}", status, error_text);
    }

    Ok(response.json()?)
}

fn sleep_js<'js>(ctx: Ctx<'js>, ms: i32) -> rquickjs::Result<()> {
    sleep_blocking(ms)
        .map_err(|error| Exception::throw_message(&ctx, &error.to_string()))
}

fn sleep_blocking(ms: i32) -> anyhow::Result<()> {
    if ms < 0 {
        anyhow::bail!("sleep 时间不能为负数");
    }
    std::thread::sleep(Duration::from_millis(ms as u64));
    Ok(())
}

fn blocking_client() -> anyhow::Result<Client> {
    Ok(Client::builder().timeout(Duration::from_secs(15)).build()?)
}

fn quickjs_error_to_anyhow(ctx: &Ctx<'_>, error: JsError) -> anyhow::Error {
    match error {
        JsError::Exception => anyhow::anyhow!(quickjs_exception_message(ctx)),
        other => anyhow::anyhow!(other.to_string()),
    }
}

fn quickjs_exception_message(ctx: &Ctx<'_>) -> String {
    let caught = ctx.catch();

    ctx.globals()
        .get::<_, Function>("String")
        .and_then(|string_fn| string_fn.call::<_, JsString>((caught,)))
        .and_then(|value| value.to_string())
        .unwrap_or_else(|_| "JavaScript exception".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn runs_async_iife_plugin_code() {
        let output = run_plugin_code(
            "(async function() { return { type: 'text', text: 'hello' }; })()",
            HashMap::new(),
        )
            .await
            .unwrap();

        assert_eq!(output["type"], "text");
        assert_eq!(output["text"], "hello");
    }

    #[tokio::test]
    async fn allows_await_on_plain_helper_values() {
        let output = run_plugin_code(
            "(async function() { const value = await echoJson({ ok: true }); return { type: 'text', text: String(value.ok) }; })()",
            HashMap::new(),
        )
            .await
            .unwrap();

        assert_eq!(output["text"], "true");
    }

    #[tokio::test]
    async fn reports_js_errors() {
        let err = run_plugin_code("(async function() { throw new Error('boom'); })()", HashMap::new())
            .await
            .unwrap_err()
            .to_string();

        assert!(err.contains("boom"));
    }

    #[tokio::test]
    async fn rejects_bare_return_code() {
        let err = run_plugin_code("return { type: 'text', text: 'hello' };", HashMap::new())
            .await
            .unwrap_err()
            .to_string();

        assert!(err.contains("return"));
    }

    #[tokio::test]
    async fn config_is_available_in_js() {
        let mut config = HashMap::new();
        config.insert("type".to_string(), "sfw".to_string());
        let output = run_plugin_code(
            "(async function() { return { type: 'text', text: config.type }; })()",
            config,
        )
            .await
            .unwrap();

        assert_eq!(output["text"], "sfw");
    }
}
