use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use serde_json::Value;
use tower::ServiceExt;

use super::app;

const EXAMPLE: &str = include_str!("../../../../../examples/2026.jvms.json");

fn post(uri: &str, body: impl Into<Body>) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .body(body.into())
        .unwrap()
}

/// Sends one request through the whole app (routes + layers) without a network.
async fn send(request: Request<Body>) -> (StatusCode, Value) {
    let response = app().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, body)
}

#[tokio::test]
async fn health_is_ok() {
    let request = Request::builder()
        .uri("/api/health")
        .body(Body::empty())
        .unwrap();

    let response = app().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(&bytes[..], b"ok");
}

#[tokio::test]
async fn evaluate_example_is_ok() {
    let (status, body) = send(post("/api/evaluate", EXAMPLE)).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["lines"].as_array().unwrap().len(), 25);
}

#[tokio::test]
async fn broken_json_is_bad_request() {
    let (status, body) = send(post("/api/evaluate", "{ broken")).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["code"], "badRequest");
}

#[tokio::test]
async fn future_file_version_is_unprocessable() {
    let (status, body) = send(post("/api/project/validate", r#"{ "schemaVersion": 67 }"#)).await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["code"], "unsupportedVersion");
}

#[tokio::test]
async fn body_over_the_limit_is_too_large() {
    let (status, body) = send(post("/api/project/validate", vec![b' '; 2_000_000])).await;

    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(body["code"], "payloadTooLarge");
}

#[tokio::test]
async fn responses_are_compressed_when_asked() {
    let mut request = post("/api/evaluate", EXAMPLE);
    request
        .headers_mut()
        .insert(header::ACCEPT_ENCODING, "gzip".parse().unwrap());

    let response = app().oneshot(request).await.unwrap();

    assert_eq!(response.headers()[header::CONTENT_ENCODING], "gzip");
}

#[tokio::test]
async fn get_on_a_post_route_is_not_allowed() {
    let request = Request::builder()
        .uri("/api/evaluate")
        .body(Body::empty())
        .unwrap();

    let response = app().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
}
