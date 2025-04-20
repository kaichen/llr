use axum::{
    body::Body,
    extract::{Extension, OriginalUri},
    http::{header, HeaderMap, HeaderValue, Request, Response, StatusCode},
};
use reqwest::Method;
use tracing::{error, info};
use axum::body::to_bytes;
use chrono::Utc;
use tokio::fs::{create_dir_all, OpenOptions};
use tokio::io::AsyncWriteExt;

use crate::state::AppState;

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

    let target = build_target_url(&state.upstream, &original_uri)
        .ok_or(StatusCode::BAD_REQUEST)?;

    let (parts, body) = req.into_parts();
    let bytes = to_bytes(body, 2 * 1024 * 1024)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // -------- log REQUEST body --------
    let host_owned = state
        .upstream
        .host()
        .unwrap_or("unknown")
        .to_string();
    if state.dump_body {
        log_body(&host_owned, true, &bytes).await;
    }

    let forward_req = state
        .client
        .request(Method::POST, target)
        .headers(build_headers(&parts.headers, &state.auth_header)?)
        .body(bytes.clone())
        .build()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let resp = state.client.execute(forward_req).await.map_err(|e| {
        error!("Upstream error: {}", e);
        StatusCode::BAD_GATEWAY
    })?;

    Ok(convert_response(resp, &host_owned, &state).await?)
}

fn build_target_url(base: &axum::http::Uri, ori: &axum::http::Uri) -> Option<reqwest::Url> {
    let mut url = base.to_string();
    if !url.ends_with('/') { url.push('/'); }
    url.push_str(ori.path().trim_start_matches('/'));
    if let Some(q) = ori.query() {
        url.push('?');
        url.push_str(q);
    }
    reqwest::Url::parse(&url).ok()
}

fn build_headers(src: &HeaderMap, auth: &HeaderValue) -> Result<HeaderMap, StatusCode> {
    let mut dst = HeaderMap::new();
    for (name, val) in src.iter() {
        if name == header::HOST || name == header::AUTHORIZATION { continue; }
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
    let bytes = upstream.bytes().await.map_err(|_| StatusCode::BAD_GATEWAY)?;

    // -------- log RESPONSE body --------
    if state.dump_body {
        log_body(host, false, &bytes).await;
    }

    builder
        .body(Body::from(bytes))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}