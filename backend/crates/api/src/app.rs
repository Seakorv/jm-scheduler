use axum::{Router, extract::DefaultBodyLimit};
use tower_http::{compression::CompressionLayer, trace::TraceLayer};

use crate::routes;

pub fn app() -> Router {
    routes::router()
        .layer(DefaultBodyLimit::max(1024 * 1024))
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
}
