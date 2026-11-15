mod app;
mod config;
mod error;
mod routes;

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

    let listener = tokio::net::TcpListener::bind((config.host, config.port))
        .await
        .unwrap();
    tracing::info!(addr = %listener.local_addr().unwrap(), "listening");
    axum::serve(listener, app::app()).await.unwrap();
}
