use axum::http::{Method, Uri};
use chrono::Utc;
use reqwest::Url;
use tokio::fs::{create_dir_all, OpenOptions};
use tokio::io::AsyncWriteExt;
use tracing::{debug, error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn init(filter: &str) {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(filter))
        .with(tracing_subscriber::fmt::layer())
        .init();
}

pub async fn log_body(host: &str, is_request: bool, bytes: &[u8]) {
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

pub fn log_request_info(method: &Method, uri: &Uri) {
    info!("{} {}", method, uri);
}

pub fn log_target_url(url: &str) {
    info!("target: {}", url);
}

pub fn log_transformation(from_path: &str, to_path: &str) {
    info!("Transformation: {} -> {}", from_path, to_path);
}

pub fn log_transformed_body(body: &[u8]) {
    info!("Transformed body: {}", String::from_utf8_lossy(body));
}

pub fn log_upstream_error(error: &reqwest::Error) {
    error!("Upstream error: {}", error);
}

pub fn log_request_details(method: &Method, url: &Url, headers: &str) {
    debug!("Forwarding request: {} {}\nHeaders:\n{}", method, url, headers);
}
