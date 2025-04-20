use axum::http::{HeaderValue, Uri};
use reqwest::Client;
use std::sync::Arc;
use anyhow::{Result, Context};

use crate::cli::Args;

#[derive(Clone)]
pub struct AppState {
    pub client: Arc<Client>,
    pub upstream: Uri,
    pub auth_header: HeaderValue,
    pub dump_body: bool,
}

impl TryFrom<&Args> for AppState {
    type Error = anyhow::Error;

    fn try_from(args: &Args) -> Result<Self> {
        let auth_header = HeaderValue::from_str(&args.api_key)
            .context("Invalid api_key header")?;

        Ok(Self {
            client: Arc::new(Client::new()),
            upstream: args.upstream_uri(),
            auth_header,
            dump_body: args.dump_body,
        })
    }
} 