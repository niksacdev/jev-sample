use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use futures_util::StreamExt;
use jev_sample::{
    http::{OperatorAuth, telemetry_router},
    telemetry::{HISTORY_LIMIT, LiveTelemetry},
};
use tower::ServiceExt;
use tracing_subscriber::prelude::*;

const KEY: &str = "live-test-operator-key-at-least-32-bytes";

#[test]
fn instrumentation_is_allowlisted_redacted_ordered_and_bounded()
-> Result<(), Box<dyn std::error::Error>> {
    let logs = LiveTelemetry::default();
    let subscriber = tracing_subscriber::registry().with(logs.clone());
    tracing::subscriber::with_default(subscriber, || {
        tracing::info!(
            message = "private-prompt-marker",
            api_key = "private-key-marker"
        );
        for index in 0..300 {
            tracing::info!(
                event = "workflow_event",
                stage = "plan_started",
                actor = "planner",
                model = "fixture-model",
                outcome = "running",
                sequence = index,
                message = "private-prompt-marker",
                request_body = "private-body-marker",
                api_key = "private-key-marker",
                run_id = "run\x1b]52;attack\x07"
            );
        }
    });
    let (history, mut receiver) = logs.subscribe()?;
    assert_eq!(history.len(), HISTORY_LIMIT);
    assert_eq!(history.front().ok_or("missing entry")?.sequence, 45);
    assert_eq!(history.back().ok_or("missing entry")?.sequence, 300);
    let encoded = serde_json::to_string(&history)?;
    assert!(!encoded.contains("private-"));
    assert!(!encoded.contains("\\u001b"));
    let subscriber = tracing_subscriber::registry().with(logs.clone());
    tracing::subscriber::with_default(subscriber, || {
        tracing::info!(
            event = "workflow_event",
            stage = "plan_completed",
            outcome = "succeeded"
        );
    });
    let received = receiver.try_recv()?;
    assert_eq!(received.sequence, 301);
    assert_eq!(
        received.fields.get("stage").ok_or("missing stage")?,
        "plan_completed"
    );
    Ok(())
}

#[tokio::test]
async fn stream_requires_auth_origin_and_reports_live_events_before_completion()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let logs = LiveTelemetry::default();
    let app = telemetry_router(logs.clone(), OperatorAuth::new(Some(KEY.into()))?);
    let unauthorized = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/operator/telemetry")
                .body(Body::empty())?,
        )
        .await?;
    assert_eq!(unauthorized.status(), StatusCode::UNAUTHORIZED);
    let denied = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/operator/telemetry")
                .header("authorization", format!("Bearer {KEY}"))
                .header("origin", "https://untrusted.example")
                .body(Body::empty())?,
        )
        .await?;
    assert_eq!(denied.status(), StatusCode::FORBIDDEN);
    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/operator/telemetry")
                .header("authorization", format!("Bearer {KEY}"))
                .body(Body::empty())?,
        )
        .await?;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get("cache-control")
            .ok_or("missing header")?,
        "no-store"
    );
    let mut stream = response.into_body().into_data_stream();
    let ready = stream.next().await.ok_or("missing ready")??;
    assert!(std::str::from_utf8(&ready)?.contains("event: ready"));
    let subscriber = std::sync::Arc::new(tracing_subscriber::registry().with(logs));
    tracing::subscriber::with_default(subscriber.clone(), || {
        tracing::info!(
            event = "workflow_event",
            stage = "reply_started",
            actor = "planner",
            model = "fixture-model",
            outcome = "running"
        );
    });
    let frame = tokio::time::timeout(std::time::Duration::from_secs(1), stream.next())
        .await?
        .ok_or("missing live frame")??;
    let frame = std::str::from_utf8(&frame)?;
    assert!(frame.contains("reply_started"));
    assert!(!frame.contains("reply_completed"));
    tracing::subscriber::with_default(subscriber, || {
        tracing::info!(
            event = "workflow_event",
            stage = "reply_completed",
            actor = "planner",
            model = "fixture-model",
            outcome = "succeeded"
        );
    });
    let frame = tokio::time::timeout(std::time::Duration::from_secs(1), stream.next())
        .await?
        .ok_or("missing completion frame")??;
    assert!(std::str::from_utf8(&frame)?.contains("reply_completed"));
    Ok(())
}

#[tokio::test]
async fn slow_stream_gaps_and_connection_limits_are_explicit()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let logs = LiveTelemetry::default();
    let app = telemetry_router(logs.clone(), OperatorAuth::new(Some(KEY.into()))?);
    let request = || {
        Request::builder()
            .uri("/v1/operator/telemetry")
            .header("authorization", format!("Bearer {KEY}"))
            .body(Body::empty())
    };
    let mut responses = Vec::new();
    for _ in 0..4 {
        responses.push(app.clone().oneshot(request()?).await?);
    }

    let exhausted = app.clone().oneshot(request()?).await?;
    assert_eq!(exhausted.status(), StatusCode::TOO_MANY_REQUESTS);
    let response = responses.pop().ok_or("missing response")?;
    let mut stream = response.into_body().into_data_stream();
    let _ = stream.next().await;
    let subscriber = tracing_subscriber::registry().with(logs);
    tracing::subscriber::with_default(subscriber, || {
        for _ in 0..300 {
            tracing::info!(event = "http_request_received", route = "assessment_submit");
        }
    });
    let gap = stream.next().await.ok_or("missing gap")??;
    assert!(std::str::from_utf8(&gap)?.contains("event: gap"));
    drop(stream);
    let restored = app.oneshot(request()?).await?;
    assert_eq!(restored.status(), StatusCode::OK);
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn stream_expires_at_fifteen_minutes_without_reconnecting()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = telemetry_router(
        LiveTelemetry::default(),
        OperatorAuth::new(Some(KEY.into()))?,
    );
    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/operator/telemetry")
                .header("authorization", format!("Bearer {KEY}"))
                .body(Body::empty())?,
        )
        .await?;
    let mut stream = response.into_body().into_data_stream();
    let _ = stream.next().await;
    tokio::time::advance(std::time::Duration::from_secs(900)).await;
    let frame = stream.next().await.ok_or("missing expiry")??;
    assert!(std::str::from_utf8(&frame)?.contains("event: closed"));
    assert!(stream.next().await.is_none());
    Ok(())
}

#[test]
fn typed_attempt_failures_are_normalized_without_payloads() -> Result<(), Box<dyn std::error::Error>>
{
    let logs = LiveTelemetry::default();
    let subscriber = tracing_subscriber::registry().with(logs.clone());
    tracing::subscriber::with_default(subscriber, || {
        tracing::info!(
            event = "assessment_attempt_completed",
            failure_code = Some("deadline"),
            request_body = "private-body"
        );
        tracing::info!(
            event = "decision_attempt_finished",
            failure = Some("invalid_response"),
            response_body = "private-body"
        );
    });
    let (history, _) = logs.subscribe()?;
    assert_eq!(history.len(), 2);
    assert_eq!(
        history[0].fields.get("code").ok_or("missing failure")?,
        "deadline"
    );
    assert_eq!(
        history[1].fields.get("code").ok_or("missing failure")?,
        "invalid_response"
    );
    assert!(!serde_json::to_string(&history)?.contains("private-body"));
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn expiry_precedes_replay_and_releases_capacity_at_closure()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let logs = LiveTelemetry::default();
    let subscriber = tracing_subscriber::registry().with(logs.clone());
    tracing::subscriber::with_default(subscriber, || {
        tracing::info!(event = "workflow_event", stage = "plan_started");
    });
    let app = telemetry_router(logs, OperatorAuth::new(Some(KEY.into()))?);
    let request = || {
        Request::builder()
            .uri("/v1/operator/telemetry")
            .header("authorization", format!("Bearer {KEY}"))
            .body(Body::empty())
    };
    let response = app.clone().oneshot(request()?).await?;
    let mut stream = response.into_body().into_data_stream();
    let _ = stream.next().await;
    tokio::time::advance(std::time::Duration::from_secs(900)).await;
    let frame = stream.next().await.ok_or("missing expiry")??;
    let frame = std::str::from_utf8(&frame)?;
    assert!(frame.contains("event: closed"));
    assert!(frame.contains("15-minute limit"));
    assert!(!frame.contains("plan_started"));
    let mut responses = Vec::new();
    for _ in 0..4 {
        let response = app.clone().oneshot(request()?).await?;
        assert_eq!(response.status(), StatusCode::OK);
        responses.push(response);
    }
    assert!(stream.next().await.is_none());
    Ok(())
}
