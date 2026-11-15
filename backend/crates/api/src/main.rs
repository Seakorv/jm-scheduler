mod config;
use axum::{Router, routing::get};

use crate::config::Config;

#[tokio::main]
async fn main() {
    let config = Config::load().unwrap_or_else(|error| {
        eprintln!("invalid config: {error}");
        std::process::exit(1);
    });

    let app = Router::new().route("/api/health", get(health));
    let listener = tokio::net::TcpListener::bind((config.host, config.port))
        .await
        .unwrap();
    println!("listening on http://{}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> &'static str {
    "ok"
}
