use std::time::Duration;

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use jev_sample::{
    http::{Application, router, router_with},
    jev::{Failure, Jev, MODEL},
};
use serde_json::{Value, json};
use tower::ServiceExt;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{body_partial_json, header, method, path},
};

type TestResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

fn provider_response() -> Value {
    json!({
        "model": MODEL,
        "answers": {
            "claim": {"type": "noul", "noul": 0.95},
            "policy_change": {"type": "noul", "noul": 0.8},
            "customer_details": {"type": "noul", "noul": 0.1},
            "billing": {"type": "noul", "noul": 0.79}
        },
        "usage": {"input_tokens": 123, "output_tokens": 12}
    })
}

fn client(server: &MockServer) -> Result<Jev, Box<dyn std::error::Error + Send + Sync>> {
    Jev::new(
        format!("{}/v1/systemone", server.uri()),
        "test-key",
        Duration::from_secs(2),
    )
}

async fn post(app: Router, body: Value) -> TestResultResponse {
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/messages")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))?,
        )
        .await?;
    let status = response.status();
    let value = serde_json::from_slice(&to_bytes(response.into_body(), 64 * 1024).await?)?;
    Ok((status, value))
}

type TestResultResponse = Result<(StatusCode, Value), Box<dyn std::error::Error + Send + Sync>>;

async fn inspect(app: Router) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/operator/runs")
                .body(Body::empty())?,
        )
        .await?;
    assert_eq!(response.status(), StatusCode::OK);
    Ok(serde_json::from_slice(
        &to_bytes(response.into_body(), 128 * 1024).await?,
    )?)
}

#[test]
fn browser_contracts_match_rust() {
    assert_eq!(
        jev_sample::contracts::typescript(),
        include_str!("../web/src/contracts.ts")
    );
}

#[tokio::test]
async fn baseline_returns_application_contract_not_provider_details() -> TestResult {
    let app = router();
    let (status, reply) = post(
        app.clone(),
        json!({"message": "File a claim; update my address; delay my premium."}),
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(reply["state"], "review_required");
    assert_eq!(reply["tasks"].as_array().map(Vec::len), Some(3));
    for field in ["model", "assessor", "signals", "input_tokens"] {
        assert!(reply.get(field).is_none());
    }
    let runs = inspect(app).await?;
    assert_eq!(runs[0]["assessor"], "keyword_baseline");
    assert!(runs[0]["signals"][0]["probability"].is_null());
    assert!(runs[0].get("message").is_none());
    Ok(())
}

#[tokio::test]
async fn invalid_input_never_creates_a_run() -> TestResult {
    let app = router();
    for body in [
        json!({"message": ""}),
        json!({"message": " "}),
        json!({"message": "x".repeat(4001)}),
        json!({"message": "claim", "model": "jev-latest"}),
        json!({"message": 7}),
    ] {
        let (status, _) = post(app.clone(), body).await?;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    }
    assert_eq!(inspect(app).await?, json!([]));
    Ok(())
}

#[tokio::test]
async fn body_and_browser_origin_are_limited() -> TestResult {
    let app = router();
    let (status, body) = post(app.clone(), json!({"message": "x".repeat(9000)})).await?;
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(body["code"], "invalid_request");
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/messages")
                .header("content-type", "application/json")
                .header("origin", "https://untrusted.example")
                .body(Body::from(r#"{"message":"claim"}"#))?,
        )
        .await?;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_eq!(inspect(app).await?, json!([]));
    Ok(())
}

#[tokio::test]
async fn unrecognized_request_requires_clarification_and_run_history_is_bounded() -> TestResult {
    let app = router();
    let (_, reply) = post(app.clone(), json!({"message": "Hello"})).await?;
    assert_eq!(reply["state"], "clarification_required");
    assert_eq!(reply["tasks"], json!([]));
    for _ in 1..100 {
        assert_eq!(
            post(app.clone(), json!({"message": "Hello"})).await?.0,
            StatusCode::OK
        );
    }
    assert_eq!(
        post(app.clone(), json!({"message": "Hello"})).await?.0,
        StatusCode::TOO_MANY_REQUESTS
    );
    assert_eq!(inspect(app).await?.as_array().map(Vec::len), Some(100));
    Ok(())
}

#[tokio::test]
async fn real_adapter_sends_documented_questions_and_preserves_provenance() -> TestResult {
    let server = MockServer::start().await;
    let questions = jev_sample::agent::INTENTS
        .iter()
        .map(|(_, id, instruction)| {
            (
                id.to_string(),
                json!({"type": "noul", "instructions": instruction}),
            )
        })
        .collect::<serde_json::Map<_, _>>();
    Mock::given(method("POST"))
        .and(path("/v1/systemone"))
        .and(header("authorization", "Bearer test-key"))
        .and(body_partial_json(
            json!({"model": MODEL, "state": "synthetic request", "questions": questions}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(provider_response()))
        .expect(1)
        .mount(&server)
        .await;
    let app = router_with(Application::new(Some(client(&server)?)));
    let (status, reply) = post(app.clone(), json!({"message": "synthetic request"})).await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(reply["tasks"].as_array().map(Vec::len), Some(2));
    let run = inspect(app).await?;
    assert_eq!(run[0]["model"], MODEL);
    assert_eq!(run[0]["input_tokens"], 123);
    assert_eq!(run[0]["signals"][3]["matched"], false);
    assert_eq!(run[0]["tasks"][0]["state"], "review_required");
    Ok(())
}

#[tokio::test]
async fn authentication_uses_each_configured_key_not_a_literal_placeholder() -> TestResult {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(provider_response()))
        .expect(2)
        .mount(&server)
        .await;
    for suffix in ["first", "second"] {
        let key = format!("dummy-{suffix}");
        Jev::new(
            format!("{}/v1/systemone", server.uri()),
            &key,
            Duration::from_secs(2),
        )?
        .assess("synthetic")
        .await
        .map_err(|failure| format!("unexpected failure: {failure:?}"))?;
    }
    let requests = server
        .received_requests()
        .await
        .ok_or("No requests captured")?;
    assert_eq!(requests.len(), 2);
    for (request, suffix) in requests.iter().zip(["first", "second"]) {
        let actual = request
            .headers
            .get("authorization")
            .ok_or("Missing authorization")?
            .to_str()?;
        let expected = ["Bearer ", "dummy-", suffix].concat();
        assert_eq!(actual, expected);
    }
    assert_ne!(
        requests[0].headers["authorization"],
        requests[1].headers["authorization"]
    );
    Ok(())
}

#[tokio::test]
async fn provider_failure_is_visible_without_fallback_or_retry() -> TestResult {
    for (status, code) in [
        (401, "provider_authentication"),
        (429, "provider_rate_limited"),
        (529, "provider_rate_limited"),
        (500, "provider_error"),
        (422, "provider_error"),
        (302, "provider_error"),
    ] {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(status))
            .expect(1)
            .mount(&server)
            .await;
        let app = router_with(Application::new(Some(client(&server)?)));
        let (status, body) = post(app.clone(), json!({"message": "claim"})).await?;
        assert_eq!(status, StatusCode::BAD_GATEWAY);
        assert_eq!(body["code"], code);
        let run = inspect(app).await?;
        assert_eq!(run[0]["state"], "failed");
        assert_eq!(run[0]["failure_code"], code);
        assert_eq!(run[0]["tasks"], json!([]));
    }
    Ok(())
}

#[tokio::test]
async fn adapter_rejects_missing_extra_wrong_and_invalid_answers() -> TestResult {
    let original = provider_response();
    let mut cases = vec![];
    let mut changed = original.clone();
    changed["model"] = json!("other");
    cases.push(changed);
    let mut changed = original.clone();
    changed["answers"]["claim"] = json!({"type":"noul","noul":1.1});
    cases.push(changed);
    let mut changed = original.clone();
    changed["answers"]["claim"] = json!({"type":"noul","noul":-0.1});
    cases.push(changed);
    let mut changed = original.clone();
    changed["answers"]["claim"] = json!({"type":"choice","noul":0.9});
    cases.push(changed);
    let mut changed = original.clone();
    changed["answers"] = json!({});
    cases.push(changed);
    let mut changed = original.clone();
    changed["answers"]["extra"] = json!({"type":"noul","noul":0.5});
    cases.push(changed);
    let mut changed = original.clone();
    changed["usage"] = json!({});
    cases.push(changed);
    let mut changed = original;
    changed["answers"]["claim"]["noul"] = json!("NaN");
    cases.push(changed);
    for case in cases {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_json(case))
            .expect(1)
            .mount(&server)
            .await;
        assert_eq!(
            client(&server)?.assess("synthetic").await.err(),
            Some(Failure::InvalidResponse)
        );
    }
    Ok(())
}

#[tokio::test]
async fn adapter_bounds_response_and_deadline() -> TestResult {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_string("x".repeat(32769)))
        .expect(1)
        .mount(&server)
        .await;
    assert_eq!(
        client(&server)?.assess("synthetic").await.err(),
        Some(Failure::ResponseTooLarge)
    );
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
        .expect(1)
        .mount(&server)
        .await;
    assert_eq!(
        client(&server)?.assess("synthetic").await.err(),
        Some(Failure::InvalidResponse)
    );
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(2)))
        .expect(1)
        .mount(&server)
        .await;
    let jev = Jev::new(server.uri(), "test-key", Duration::from_millis(100))?;
    assert_eq!(jev.assess("synthetic").await.err(), Some(Failure::Timeout));
    assert!(Jev::new(server.uri(), "\n", Duration::from_secs(1)).is_err());
    Ok(())
}
