mod config;
use axum::{Router, routing::get};
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use crate::config::Config;

#[tokio::main]
async fn main() {
    let config = Config::load().unwrap_or_else(|error| {
        eprintln!("invalid config: {error}");
        std::process::exit(1);
    });

    let filter = EnvFilter::try_new(&config.log_level).unwrap_or_else(|error| {
        eprintln!("invalid log level {error}");
        std::process::exit(1);
    });
    tracing_subscriber::fmt().with_env_filter(filter).init();

    let app = Router::new()
        .route("/api/health", get(health))
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind((config.host, config.port))
        .await
        .unwrap();
    tracing::info!(addr = %listener.local_addr().unwrap(), "listening");
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> &'static str {
    "ok"
}
