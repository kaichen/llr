use clap::Parser;
use std::str::FromStr;
use axum::http::Uri;

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Args {
    /// Upstream LLM base url, e.g. https://api.openai.com
    #[arg(long)]
    pub upstream: String,

    /// Port to listen on
    #[arg(long, default_value = "8080")]
    pub port: u16,

    /// Authorization header value, e.g. "Bearer sk-XXXX"
    #[arg(long)]
    pub api_key: String,

    /// RUST_LOG style filter, default `info`
    #[arg(long, default_value = "info")]
    pub log: String,
}

impl Args {
    pub fn log_level(&self) -> &str { &self.log }
    pub fn upstream_uri(&self) -> Uri {
        Uri::from_str(&self.upstream).expect("invalid upstream url")
    }
} 