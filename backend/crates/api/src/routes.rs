mod health;
mod project;
mod schedule;

use axum::{
    Router,
    routing::{get, post},
};

pub fn router() -> Router {
    Router::new()
        .route("/api/health", get(health::health))
        .route("/api/evaluate", post(schedule::evaluate_handler))
        .route("/api/check-edit", post(schedule::check_edit_handler))
        .route("/api/candidates", post(schedule::candidates_handler))
        .route("/api/project/validate", post(project::validate_handler))
}
