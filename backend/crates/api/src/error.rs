use axum::Json;
use axum::extract::FromRequest;
use axum::extract::rejection::{BytesRejection, JsonRejection};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use jmetro_core::LoadError;
use serde::Serialize;
use ts_rs::TS;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("{0}")]
    BadRequest(String),
    #[error("unsupported file version {0}")]
    UnsupportedVersion(u32),
    #[error("internal error")]
    Internal(String),
    #[error("request body is too large")]
    PayloadTooLarge,
}

#[derive(Serialize, TS)]
#[ts(export)]
struct ErrorBody {
    code: &'static str,
    message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code) = match &self {
            ApiError::BadRequest(_) => (StatusCode::BAD_REQUEST, "badRequest"),
            ApiError::UnsupportedVersion(_) => {
                (StatusCode::UNPROCESSABLE_ENTITY, "unsupportedVersion")
            }
            ApiError::Internal(details) => {
                tracing::error!(details = %details, "internal error");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal")
            }
            ApiError::PayloadTooLarge => (StatusCode::PAYLOAD_TOO_LARGE, "payloadTooLarge"),
        };

        let body = ErrorBody {
            code,
            message: self.to_string(),
        };

        (status, Json(body)).into_response()
    }
}

impl From<LoadError> for ApiError {
    fn from(error: LoadError) -> Self {
        match error {
            LoadError::InvalidJson(error) => ApiError::BadRequest(error.to_string()),
            LoadError::UnsupportedVersion(version) => ApiError::UnsupportedVersion(version),
            LoadError::Invalid(issues) => {
                let issues: Vec<String> = issues.iter().map(ToString::to_string).collect();
                ApiError::BadRequest(format!("invalid project: {}", issues.join(", ")))
            }
        }
    }
}

#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(ApiError))]
pub struct AppJson<T>(pub T);

impl<T: Serialize> IntoResponse for AppJson<T> {
    fn into_response(self) -> Response {
        match serde_json::to_value(&self.0) {
            Ok(value) => Json(value).into_response(),
            Err(error) => ApiError::Internal(error.to_string()).into_response(),
        }
    }
}

impl From<JsonRejection> for ApiError {
    fn from(rejection: JsonRejection) -> Self {
        check_payload_size(rejection.status(), rejection.body_text())
    }
}

impl From<BytesRejection> for ApiError {
    fn from(rejection: BytesRejection) -> Self {
        check_payload_size(rejection.status(), rejection.body_text())
    }
}

fn check_payload_size(status: StatusCode, text: String) -> ApiError {
    if status == StatusCode::PAYLOAD_TOO_LARGE {
        ApiError::PayloadTooLarge
    } else {
        ApiError::BadRequest(text)
    }
}

// Claude generated tests
#[cfg(test)]
mod tests {
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, header};
    use jmetro_core::load_project;
    use serde_json::Value;

    use super::*;

    const EXAMPLE: &str = include_str!("../../../../examples/2026.jvms.json");

    async fn status_and_body(error: ApiError) -> (StatusCode, Value) {
        let response = error.into_response();
        let status = response.status();
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        (status, serde_json::from_slice(&bytes).unwrap())
    }

    #[tokio::test]
    async fn bad_request_is_400_with_its_message() {
        let (status, body) = status_and_body(ApiError::BadRequest(String::from("broken"))).await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["code"], "badRequest");
        assert_eq!(body["message"], "broken");
    }

    #[tokio::test]
    async fn unsupported_version_is_422() {
        let (status, body) = status_and_body(ApiError::UnsupportedVersion(67)).await;

        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(body["code"], "unsupportedVersion");
        assert_eq!(body["message"], "unsupported file version 67");
    }

    #[tokio::test]
    async fn internal_is_500_and_hides_the_details() {
        let (status, body) =
            status_and_body(ApiError::Internal(String::from("secret database password"))).await;

        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(body["code"], "internal");
        assert_eq!(body["message"], "internal error");
    }

    #[test]
    fn broken_json_file_is_a_bad_request() {
        let error: ApiError = load_project("not json").unwrap_err().into();

        assert!(matches!(error, ApiError::BadRequest(_)));
    }

    #[test]
    fn newer_file_version_stays_unsupported_version() {
        let error: ApiError = load_project(r#"{ "schemaVersion": 67 }"#)
            .unwrap_err()
            .into();

        assert!(matches!(error, ApiError::UnsupportedVersion(67)));
    }

    #[test]
    fn invalid_project_lists_the_issues() {
        let mut project: Value = serde_json::from_str(EXAMPLE).unwrap();
        project["name"] = Value::from("  ");

        let error: ApiError = load_project(&project.to_string()).unwrap_err().into();

        let ApiError::BadRequest(message) = error else {
            panic!("expected BadRequest, got {error:?}");
        };
        assert!(message.starts_with("invalid project: "));
        assert!(message.contains("project name is empty"));
    }

    #[tokio::test]
    async fn broken_request_json_is_rejected_with_api_error() {
        let request = Request::builder()
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from("{ not json"))
            .unwrap();

        let result = AppJson::<Value>::from_request(request, &()).await;

        assert!(matches!(result, Err(ApiError::BadRequest(_))));
    }

    #[tokio::test]
    async fn valid_request_json_is_extracted() {
        let request = Request::builder()
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(r#"{ "a": 1 }"#))
            .unwrap();

        let Ok(AppJson(value)) = AppJson::<Value>::from_request(request, &()).await else {
            panic!("valid JSON was rejected");
        };
        assert_eq!(value["a"], 1);
    }
}
