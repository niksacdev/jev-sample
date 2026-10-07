use std::{collections::BTreeMap, sync::Arc, time::Duration};

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use jev_sample::{
    application::{ExecutionLimits, ServicingService},
    assessment::Assessor,
    contracts::AssessorId,
    domain::Message,
    jev::{Failure, Jev, MODEL},
    routing::RoutingPolicy,
};
mod support;
use serde_json::{Value, json};
use support::router;
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

fn service_with_assessors(assessors: Vec<(AssessorId, Arc<dyn Assessor>)>) -> ServicingService {
    let policy = RoutingPolicy::from_json(include_str!("../config/routing.json"))
        .unwrap_or_else(|error| panic!("Invalid checked-in test policy: {error}"));
    let limits = ExecutionLimits::from_json(include_str!("../config/execution.json"))
        .unwrap_or_else(|error| panic!("Invalid checked-in test limits: {error}"));
    ServicingService::with_assessors(
        assessors.into_iter().collect::<BTreeMap<_, _>>(),
        policy,
        limits,
    )
}

async fn inspect(app: Router) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/operator/runs")
                .header(
                    "authorization",
                    format!("Bearer {}", support::TEST_OPERATOR_KEY),
                )
                .body(Body::empty())?,
        )
        .await?;
    assert_eq!(response.status(), StatusCode::OK);
    Ok(serde_json::from_slice(
        &to_bytes(response.into_body(), 128 * 1024).await?,
    )?)
}

async fn get_json(app: Router, uri: &str, authorization: Option<&str>) -> TestResultResponse {
    let mut request = Request::builder().uri(uri);
    if let Some(value) = authorization {
        request = request.header("authorization", value);
    }
    let response = app.oneshot(request.body(Body::empty())?).await?;
    let status = response.status();
    let value = serde_json::from_slice(&to_bytes(response.into_body(), 128 * 1024).await?)?;
    Ok((status, value))
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
    assert_eq!(reply["execution_trace"][0]["stage"], "request_admitted");
    assert_eq!(reply["execution_trace"][4]["stage"], "run_completed");
    for field in ["model", "assessor", "signals", "input_tokens"] {
        assert!(reply.get(field).is_none());
    }
    let runs = inspect(app).await?;
    assert_eq!(runs[0]["run"]["assessor"], "keyword_baseline");
    assert_eq!(
        runs[0]["execution_trace"].as_array().map(|events| events
            .iter()
            .map(|event| event["stage"].as_str())
            .collect::<Vec<_>>()),
        Some(vec![
            Some("request_admitted"),
            Some("assessment_started"),
            Some("assessment_completed"),
            Some("routing_completed"),
            Some("run_completed"),
        ])
    );
    assert!(runs[0]["run"].get("execution_trace").is_none());
    assert!(runs[0]["run"]["signals"][0]["probability"].is_null());
    assert_eq!(
        runs[0]["provider_exchange"]["request_body"],
        "File a claim; update my address; delay my premium."
    );
    assert!(runs[0]["provider_exchange"]["response_body"].is_string());
    assert!(runs[0]["run"].get("provider_exchange").is_none());
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
    // Independent vendor request fixture: do not derive the assertion from production definitions.
    let questions = json!({
        "claim": {"type":"noul","instructions":"Does the customer ask to file or service an insurance claim?"},
        "policy_change": {"type":"noul","instructions":"Does the customer request a policy, beneficiary, endorsement or coverage change?"},
        "customer_details": {"type":"noul","instructions":"Does the customer ask to change their contact details or address?"},
        "billing": {"type":"noul","instructions":"Does the customer request help with premiums, payment timing or discounts?"}
    });
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
    let app = support::router_for(support::service(Arc::new(client(&server)?)));
    let (status, reply) = post(app.clone(), json!({"message": "synthetic request"})).await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(reply["tasks"].as_array().map(Vec::len), Some(2));
    let run = inspect(app).await?;
    assert_eq!(run[0]["run"]["model"], MODEL);
    assert_eq!(run[0]["run"]["input_tokens"], 123);
    assert_eq!(run[0]["run"]["signals"][3]["matched"], false);
    assert_eq!(run[0]["run"]["tasks"][0]["state"], "review_required");
    assert_eq!(
        serde_json::from_str::<Value>(
            run[0]["provider_exchange"]["request_body"]
                .as_str()
                .ok_or("Missing Jev request body")?
        )?["state"],
        "synthetic request"
    );
    assert_eq!(
        serde_json::from_str::<Value>(
            run[0]["provider_exchange"]["response_body"]
                .as_str()
                .ok_or("Missing Jev response body")?
        )?["model"],
        MODEL
    );
    Ok(())
}

#[tokio::test]
async fn available_assessors_are_selectable_and_each_receives_the_same_query() -> TestResult {
    use jev_sample::baseline::KeywordBaseline;

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/systemone"))
        .respond_with(ResponseTemplate::new(200).set_body_json(provider_response()))
        .expect(1)
        .mount(&server)
        .await;
    let app = support::router_for(service_with_assessors(vec![
        (AssessorId::Code, Arc::new(KeywordBaseline)),
        (AssessorId::Jev, Arc::new(client(&server)?)),
    ]));
    let (status, options) = get_json(app.clone(), "/v1/assessors", None).await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(options[0]["available"], true);
    assert_eq!(options[1]["available"], true);
    assert_eq!(options[2]["available"], false);

    let query = "A synthetic claim request";
    for assessor in ["code", "jev"] {
        let (status, reply) =
            post(app.clone(), json!({"message": query, "assessor": assessor})).await?;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(reply["state"], "review_required");
    }
    let details = inspect(app).await?;
    assert_eq!(details.as_array().map(Vec::len), Some(2));
    let jev_run = details
        .as_array()
        .and_then(|runs| runs.iter().find(|run| run["run"]["assessor"] == "jev"))
        .ok_or("Jev run missing")?;
    assert_eq!(
        serde_json::from_str::<Value>(
            jev_run["provider_exchange"]["request_body"]
                .as_str()
                .ok_or("Missing Jev request body")?
        )?["state"],
        query
    );
    assert_eq!(jev_run["provider_exchange"]["response_status"], 200);
    Ok(())
}

#[tokio::test]
async fn operator_exchange_requires_auth_and_is_not_returned_to_employee_view() -> TestResult {
    let app = support::router();
    let (_, _) = post(app.clone(), json!({"message":"synthetic claim"})).await?;
    let (status, unauthorized) = get_json(app.clone(), "/v1/operator/runs", None).await?;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(unauthorized["code"], "operator_auth_required");

    let (status, employee_runs) = get_json(app.clone(), "/v1/employee/runs", None).await?;
    assert_eq!(status, StatusCode::OK);
    assert!(employee_runs.as_array().is_some());
    assert!(employee_runs[0].get("provider_exchange").is_none());
    assert!(employee_runs[0].get("execution_trace").is_none());
    let (status, invalid_key) = get_json(
        app,
        "/v1/operator/runs",
        Some("Bearer invalid-operator-key"),
    )
    .await?;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(invalid_key["code"], "operator_auth_required");
    Ok(())
}

#[tokio::test]
async fn missing_operator_key_fails_closed_and_short_keys_are_rejected() -> TestResult {
    use jev_sample::http::{OperatorAuth, router_with};

    assert!(OperatorAuth::new(Some("short".into())).is_err());
    assert!(OperatorAuth::new(Some(format!("a{}", " ".repeat(31)))).is_err());
    assert!(OperatorAuth::new(Some("x".repeat(31))).is_err());
    assert!(OperatorAuth::new(Some("x".repeat(32))).is_ok());
    let auth = OperatorAuth::new(None)?;
    let app = router_with(
        support::service(Arc::new(jev_sample::baseline::KeywordBaseline)),
        auth,
    );
    let (status, body) = get_json(app, "/v1/operator/runs", None).await?;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body["code"], "operator_auth_unconfigured");
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
        .assess(&Message::new("synthetic".into())?)
        .await
        .result
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
    for (provider_status, code) in [
        (401, "provider_authentication"),
        (429, "provider_rate_limited"),
        (529, "provider_rate_limited"),
        (500, "provider_error"),
        (422, "provider_error"),
        (302, "provider_error"),
    ] {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(provider_status))
            .expect(1)
            .mount(&server)
            .await;
        let app = support::router_for(support::service(Arc::new(client(&server)?)));
        let (http_status, body) = post(app.clone(), json!({"message": "claim"})).await?;
        assert_eq!(http_status, StatusCode::BAD_GATEWAY);
        assert_eq!(body["code"], code);
        let run = inspect(app).await?;
        assert_eq!(run[0]["run"]["state"], "failed");
        assert_eq!(run[0]["run"]["failure_code"], code);
        assert_eq!(run[0]["execution_trace"][3]["stage"], "run_failed");
        assert_eq!(run[0]["execution_trace"][3]["outcome"], code);
        assert_eq!(run[0]["run"]["tasks"], json!([]));
        assert_eq!(
            run[0]["provider_exchange"]["response_status"],
            provider_status
        );
        assert!(run[0]["provider_exchange"]["response_body"].is_string());
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
            client(&server)?
                .assess(&Message::new("synthetic".into())?)
                .await
                .result
                .err(),
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
        client(&server)?
            .assess(&Message::new("synthetic".into())?)
            .await
            .result
            .err(),
        Some(Failure::ResponseTooLarge)
    );
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
        .expect(1)
        .mount(&server)
        .await;
    assert_eq!(
        client(&server)?
            .assess(&Message::new("synthetic".into())?)
            .await
            .result
            .err(),
        Some(Failure::InvalidResponse)
    );
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(2)))
        .expect(1)
        .mount(&server)
        .await;
    let jev = Jev::new(server.uri(), "test-key", Duration::from_millis(100))?;
    assert_eq!(
        jev.assess(&Message::new("synthetic".into())?)
            .await
            .result
            .err(),
        Some(Failure::Timeout)
    );
    assert!(Jev::new(server.uri(), "\n", Duration::from_secs(1)).is_err());
    Ok(())
}
