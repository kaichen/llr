use axum::body::to_bytes;
use axum::{
    body::Body,
    extract::{Extension, OriginalUri},
    http::{header, HeaderMap, HeaderValue, Request, Response, StatusCode},
};
use chrono::Utc;
use reqwest::Method;
use tokio::fs::{create_dir_all, OpenOptions};
use tokio::io::AsyncWriteExt;
use tracing::{error, info, debug};

use crate::state::AppState;
use crate::transform;

async fn log_body(host: &str, is_request: bool, bytes: &[u8]) {
    // best‑effort; on failure just emit a tracing error and continue
    if let Err(e) = async {
        create_dir_all("logs").await?;
        let file_path = format!("logs/{}.log", host);
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)
            .await?;

        let ts = Utc::now().to_rfc3339();
        let direction = if is_request { "REQUEST" } else { "RESPONSE" };

        file.write_all(format!("\n----- {} {} -----\n", direction, ts).as_bytes())
            .await?;
        file.write_all(bytes).await?;
        file.write_all(b"\n").await?;
        Ok::<(), std::io::Error>(())
    }
    .await
    {
        error!("failed to write body log: {}", e);
    }
}

pub async fn handle(
    Extension(state): Extension<AppState>,
    OriginalUri(original_uri): OriginalUri,
    req: Request<Body>,
) -> Result<Response<Body>, StatusCode> {
    info!("{} {}", req.method(), original_uri);

    let (parts, body) = req.into_parts();
    let bytes = to_bytes(body, 2 * 1024 * 1024)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // -------- optional Anthropic → OpenAI rewrite --------
    let (mut path, mut fwd_body) = (original_uri.path().to_string(), bytes.clone());
    if state.anthropic_mode {
        if let Some((new_path, new_body)) = transform::anthropic_to_openai(&path, &fwd_body) {
            info!("Anthropic → OpenAI rewrite: {} -> {}", path, new_path);
            path = new_path;
            info!(
                "Anthropic → OpenAI body: {}",
                String::from_utf8_lossy(&new_body)
            );
            fwd_body = axum::body::Bytes::from(new_body);
        }
    }

    // -------- log REQUEST body --------
    let host_owned = state.upstream.host().unwrap_or("unknown").to_string();
    if state.dump_body {
        log_body(&host_owned, true, &fwd_body[..]).await;
    }

    // Build target URL with (possibly) rewritten path -----------------------
    let target = {
        let mut base = state.upstream.to_string();
        if !base.ends_with('/') {
            base.push('/');
        }
        base.push_str(path.trim_start_matches('/'));
        if let Some(q) = original_uri.query() {
            base.push('?');
            base.push_str(q);
        }
        reqwest::Url::parse(&base).map_err(|_| StatusCode::BAD_REQUEST)?
    };
    info!("target: {}", target);

    let forward_req = state
        .client
        .request(Method::POST, target)
        .headers(build_headers(&parts.headers, &state.auth_header)?)
        .body(fwd_body.clone())
        .build()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 打印除 body 以外的请求内容
    {
        let method = forward_req.method();
        let url = forward_req.url();
        let mut headers_log = String::new();
        for (name, value) in forward_req.headers().iter() {
            if name == header::AUTHORIZATION {
                headers_log.push_str(&format!("{}: <REDACTED>\n", name));
            } else {
                headers_log.push_str(&format!("{}: {:?}\n", name, value));
            }
        }
        debug!("Forwarding request: {} {}\nHeaders:\n{}", method, url, headers_log);
    }

    let resp = state.client.execute(forward_req).await.map_err(|e| {
        error!("Upstream error: {}", e);
        StatusCode::BAD_GATEWAY
    })?;

    Ok(convert_response(resp, &host_owned, &state).await?)
}

fn build_headers(src: &HeaderMap, auth: &HeaderValue) -> Result<HeaderMap, StatusCode> {
    let mut dst = HeaderMap::new();
    for (name, val) in src.iter() {
        if name == header::HOST || name == header::AUTHORIZATION {
            continue;
        }
        dst.insert(name, val.clone());
    }
    dst.insert(header::AUTHORIZATION, auth.clone());
    Ok(dst)
}

async fn convert_response(
    upstream: reqwest::Response,
    host: &str,
    state: &AppState,
) -> Result<Response<Body>, StatusCode> {
    let status = upstream.status();
    let mut builder = Response::builder().status(status);
    if let Some(h) = builder.headers_mut() {
        h.extend(upstream.headers().clone());
    }
    let bytes = upstream
        .bytes()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    // -------- log RESPONSE body --------
    if state.dump_body {
        log_body(host, false, &bytes).await;
    }

    builder
        .body(Body::from(bytes))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
