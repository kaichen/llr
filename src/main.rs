use std::net::SocketAddr;

use anyhow::Result;
use axum::routing::any;
use axum::{Router, Extension};
use tower_http::trace::TraceLayer;
use tokio::net::TcpListener;
use clap::Parser;

use light_local_router::logging;
use light_local_router::proxy;
use light_local_router::cli;
use light_local_router::state::AppState;

#[tokio::main]
async fn main() -> Result<()> {
    let args = cli::Args::parse();

    logging::init(&args.log_level());

    let state = AppState::try_from(&args)?;

    let app = Router::new()
        .route("/{*path}", any(proxy::handle))
        .layer(Extension(state))
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([0, 0, 0, 0], args.port));
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
