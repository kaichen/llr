use axum::http::Uri;
use clap::Parser;
use std::str::FromStr;

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

    /// Dump LLM API request and response body
    #[arg(long, default_value_t = false)]
    pub dump_body: bool,

    /// Treat incoming requests as Anthropic `/v1/messages` schema and translate
    /// them to OpenAI `/v1/chat/completions` before forwarding.
    #[arg(long, default_value_t = false)]
    pub anthropic_mode: bool,
}

impl Args {
    pub fn log_level(&self) -> &str {
        &self.log
    }
    pub fn upstream_uri(&self) -> Uri {
        Uri::from_str(&self.upstream).expect("invalid upstream url")
    }
    pub fn anthropic_mode(&self) -> bool {
        self.anthropic_mode
    }
}
