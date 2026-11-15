use axum::{body::Bytes, extract::rejection::BytesRejection};

use jmetro_core::{Project, load_project};

use crate::error::{ApiError, AppJson};

pub async fn validate_handler(
    body: Result<Bytes, BytesRejection>,
) -> Result<AppJson<Project>, ApiError> {
    let body = body?;
    let text =
        std::str::from_utf8(&body).map_err(|error| ApiError::BadRequest(error.to_string()))?;

    let project = load_project(text)?;

    Ok(AppJson(project))
}
