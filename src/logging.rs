use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn init(filter: &str) {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(filter))
        .with(tracing_subscriber::fmt::layer())
        .init();
} 