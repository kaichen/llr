use anyhow::{Context, Result};
use axum::http::{HeaderValue, Uri};
use reqwest::Client;
use std::sync::Arc;

use crate::cli::Args;

#[derive(Clone)]
pub struct AppState {
    pub client: Arc<Client>,
    pub upstream: Uri,
    pub auth_header: HeaderValue,
    pub dump_body: bool,
    pub anthropic_mode: bool,
}

impl TryFrom<&Args> for AppState {
    type Error = anyhow::Error;

    fn try_from(args: &Args) -> Result<Self> {
        let api_key = if args.api_key.trim_start().to_ascii_lowercase().starts_with("Bearer ") {
            args.api_key.clone()
        } else {
            format!("Bearer {}", args.api_key.trim())
        };
        let auth_header = HeaderValue::from_str(&api_key).context("Invalid api_key header")?;

        Ok(Self {
            client: Arc::new(Client::new()),
            upstream: args.upstream_uri(),
            auth_header,
            dump_body: args.dump_body,
            anthropic_mode: args.anthropic_mode(),
        })
    }
}
