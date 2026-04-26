use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use reqwest::blocking::Client;
use rquickjs::promise::MaybePromise;
use rquickjs::{
    Context, Ctx, Error as JsError, Exception, Function, Runtime, String as JsString, Value,
};
use std::time::Duration;

pub async fn run_plugin_code(code: &str) -> anyhow::Result<serde_json::Value> {
    let code = code.to_owned();
    let execution = tokio::task::spawn_blocking(move || run_plugin_code_blocking(&code));

    match tokio::time::timeout(Duration::from_secs(20), execution).await {
        Ok(joined) => match joined {
            Ok(result) => result,
            Err(error) => anyhow::bail!("插件执行失败: {error}"),
        },
        Err(_) => anyhow::bail!("插件执行超时"),
    }
}

fn run_plugin_code_blocking(code: &str) -> anyhow::Result<serde_json::Value> {
    let runtime = Runtime::new()?;
    let context = Context::full(&runtime)?;

    context.with(|ctx| {
        install_helpers(ctx.clone())?;

        let wrapped_code = format!("(async function() {{\n{code}\n}})()");
        let maybe_promise = ctx
            .eval::<MaybePromise<'_>, _>(wrapped_code)
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
    globals.set("fetchText", Function::new(ctx.clone(), fetch_text_js)?)?;
    globals.set("fetchBase64", Function::new(ctx.clone(), fetch_base64_js)?)?;

    Ok(())
}

fn echo_json<'js>(value: Value<'js>) -> Value<'js> {
    value
}

fn fetch_json_js<'js>(ctx: Ctx<'js>, url: String) -> rquickjs::Result<Value<'js>> {
    let json = fetch_json_blocking(&url)
        .map_err(|error| Exception::throw_message(&ctx, &error.to_string()))?;

    rquickjs_serde::to_value(ctx.clone(), json)
        .map_err(|error| Exception::throw_message(&ctx, &error.to_string()))
}

fn fetch_text_js<'js>(ctx: Ctx<'js>, url: String) -> rquickjs::Result<String> {
    fetch_text_blocking(&url)
        .map_err(|error| Exception::throw_message(&ctx, &error.to_string()))
}

fn fetch_json_blocking(url: &str) -> anyhow::Result<serde_json::Value> {
    let response = blocking_client()?.get(url).send()?;
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
    fetch_base64_blocking(&url)
        .map_err(|error| Exception::throw_message(&ctx, &error.to_string()))
}

fn fetch_base64_blocking(url: &str) -> anyhow::Result<String> {
    let response = blocking_client()?.get(url).send()?;
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
    async fn runs_sync_returning_plugin_code() {
        let output = run_plugin_code("return { type: 'text', text: 'hello' };")
            .await
            .unwrap();

        assert_eq!(output["type"], "text");
        assert_eq!(output["text"], "hello");
    }

    #[tokio::test]
    async fn allows_await_on_plain_helper_values() {
        let output = run_plugin_code("const value = await echoJson({ ok: true }); return { type: 'text', text: String(value.ok) };")
            .await
            .unwrap();

        assert_eq!(output["text"], "true");
    }

    #[tokio::test]
    async fn reports_js_errors() {
        let err = run_plugin_code("throw new Error('boom');")
            .await
            .unwrap_err()
            .to_string();

        assert!(err.contains("boom"));
    }
}