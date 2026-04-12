//! Negative / resilience tests (NEG-01 through NEG-06).
//!
//! These verify the SDK handles malformed, empty, and unexpected server
//! responses gracefully — without panicking.

use nahook::NahookManagement;
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Helper to build a management client pointed at a mock server.
fn build_mgmt(uri: &str) -> NahookManagement {
    NahookManagement::builder("nhm_test123")
        .base_url(uri)
        .build()
        .unwrap()
}

// ── NEG-01: Malformed JSON on 200 ──

#[tokio::test]
async fn neg_01_malformed_json_response() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/management/v1/workspaces/ws_1/endpoints"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string("{invalid json!!!")
                .insert_header("content-type", "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let mgmt = build_mgmt(&server.uri());
    let result = mgmt.endpoints().list("ws_1").await;
    assert!(result.is_err(), "NEG-01: malformed JSON should produce an error");
}

// ── NEG-02: Empty body on 200 ──

#[tokio::test]
async fn neg_02_empty_body_on_200() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/management/v1/workspaces/ws_1/endpoints"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string("")
                .insert_header("content-type", "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let mgmt = build_mgmt(&server.uri());
    let result = mgmt.endpoints().list("ws_1").await;
    assert!(result.is_err(), "NEG-02: empty body on 200 should produce an error");
}

// ── NEG-03: 5xx with HTML body ──

#[tokio::test]
async fn neg_03_5xx_html_body() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/management/v1/workspaces/ws_1/endpoints"))
        .respond_with(
            ResponseTemplate::new(503)
                .set_body_string("<html><body>Service Unavailable</body></html>")
                .insert_header("content-type", "text/html"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let mgmt = build_mgmt(&server.uri());
    let err = mgmt.endpoints().list("ws_1").await.unwrap_err();

    match err {
        nahook::NahookError::Api(api_err) => {
            assert_eq!(api_err.status, 503, "NEG-03: status should be 503");
            assert!(api_err.is_retryable(), "NEG-03: 503 should be retryable");
        }
        _ => panic!("NEG-03: expected ApiError, got: {err:?}"),
    }
}

// ── NEG-04: 5xx with completely empty body ──

#[tokio::test]
async fn neg_04_5xx_empty_body() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/management/v1/workspaces/ws_1/endpoints"))
        .respond_with(
            ResponseTemplate::new(500)
                .set_body_string("")
                .insert_header("content-type", "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let mgmt = build_mgmt(&server.uri());
    let err = mgmt.endpoints().list("ws_1").await.unwrap_err();

    match err {
        nahook::NahookError::Api(api_err) => {
            assert_eq!(api_err.status, 500, "NEG-04: status should be 500");
            assert!(api_err.is_retryable(), "NEG-04: 500 should be retryable");
        }
        _ => panic!("NEG-04: expected ApiError, got: {err:?}"),
    }
}

// ── NEG-05: Response with unknown extra fields ──

#[tokio::test]
async fn neg_05_unknown_extra_fields_ignored() {
    let server = MockServer::start().await;

    // Return an endpoint with extra unknown fields
    Mock::given(method("GET"))
        .and(path("/management/v1/workspaces/ws_1/endpoints"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {
                "id": "ep_1",
                "url": "https://example.com",
                "isActive": true,
                "type": "webhook",
                "config": {},
                "createdAt": "2024-01-01T00:00:00Z",
                "updatedAt": "2024-01-01T00:00:00Z",
                "unknownField": "should_be_ignored",
                "nested": { "deep": true }
            }
        ])))
        .expect(1)
        .mount(&server)
        .await;

    let mgmt = build_mgmt(&server.uri());
    let result = mgmt.endpoints().list("ws_1").await;
    assert!(result.is_ok(), "NEG-05: unknown fields should be silently ignored");
    let list = result.unwrap();
    assert_eq!(list.data.len(), 1);
    assert_eq!(list.data[0].id, "ep_1");
}

// ── NEG-06: Response missing optional fields ──

#[tokio::test]
async fn neg_06_missing_optional_fields() {
    let server = MockServer::start().await;

    // Return an endpoint with only required fields, all optionals missing
    Mock::given(method("GET"))
        .and(path("/management/v1/workspaces/ws_1/endpoints"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {
                "id": "ep_1",
                "url": "https://example.com",
                "isActive": true,
                "type": "webhook",
                "config": {},
                "createdAt": "2024-01-01T00:00:00Z",
                "updatedAt": "2024-01-01T00:00:00Z"
            }
        ])))
        .expect(1)
        .mount(&server)
        .await;

    let mgmt = build_mgmt(&server.uri());
    let result = mgmt.endpoints().list("ws_1").await;
    assert!(result.is_ok(), "NEG-06: missing optional fields should default gracefully");
    let list = result.unwrap();
    assert_eq!(list.data.len(), 1);
    assert_eq!(list.data[0].id, "ep_1");
    assert!(list.data[0].description.is_none());
    assert!(list.data[0].secret.is_none());
    assert!(list.data[0].metadata.is_none());
}
