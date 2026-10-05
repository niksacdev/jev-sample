use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use tower::ServiceExt;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[tokio::test]
async fn health_returns_exact_json_contract() -> TestResult {
    let response = jev_sample::http::router()
        .oneshot(Request::builder().uri("/health").body(Body::empty())?)
        .await?;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");
    assert_eq!(
        to_bytes(response.into_body(), 1024).await?.as_ref(),
        br#"{"status":"ok"}"#
    );
    Ok(())
}

#[tokio::test]
async fn unknown_route_returns_not_found() -> TestResult {
    let response = jev_sample::http::router()
        .oneshot(Request::builder().uri("/unknown").body(Body::empty())?)
        .await?;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert!(to_bytes(response.into_body(), 1024).await?.is_empty());
    Ok(())
}

#[tokio::test]
async fn health_rejects_post() -> TestResult {
    let response = jev_sample::http::router()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/health")
                .body(Body::empty())?,
        )
        .await?;

    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    assert_eq!(response.headers()[header::ALLOW], "GET,HEAD");
    Ok(())
}

#[tokio::test]
async fn health_head_returns_headers_without_body() -> TestResult {
    let response = jev_sample::http::router()
        .oneshot(
            Request::builder()
                .method("HEAD")
                .uri("/health")
                .body(Body::empty())?,
        )
        .await?;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");
    assert!(to_bytes(response.into_body(), 1024).await?.is_empty());
    Ok(())
}
