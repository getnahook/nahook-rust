use nahook::types::*;
use nahook::{NahookClient, NahookManagement};
use serde_json::json;
use wiremock::matchers::{body_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// ── Client construction ──

#[test]
fn client_rejects_invalid_api_key() {
    let result = NahookClient::new("bad_key");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        format!("{err}").contains("must start with 'nhk_'"),
        "Expected error about nhk_ prefix, got: {err}"
    );
}

#[test]
fn client_accepts_valid_api_key() {
    let result = NahookClient::builder("nhk_us_test123")
        .base_url("https://localhost:1234")
        .build();
    assert!(result.is_ok());
}

#[test]
fn management_rejects_invalid_token() {
    let result = NahookManagement::new("bad_token");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        format!("{err}").contains("must start with 'nhm_'"),
        "Expected error about nhm_ prefix, got: {err}"
    );
}

#[test]
fn management_accepts_valid_token() {
    let result = NahookManagement::builder("nhm_test123")
        .base_url("https://localhost:1234")
        .build();
    assert!(result.is_ok());
}

// ── Client: send ──

#[tokio::test]
async fn client_send_calls_correct_endpoint() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/ingest/ep_123"))
        .and(header("authorization", "Bearer nhk_us_test123"))
        .and(header("accept", "application/json"))
        .and(header("content-type", "application/json"))
        .respond_with(ResponseTemplate::new(202).set_body_json(json!({
            "deliveryId": "del_abc",
            "idempotencyKey": "key-1",
            "status": "accepted"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = NahookClient::builder("nhk_us_test123")
        .base_url(server.uri())
        .build()
        .unwrap();

    let result = client
        .send(
            "ep_123",
            SendOptions {
                payload: json!({"test": true}),
                idempotency_key: Some("key-1".to_string()),
            },
        )
        .await
        .unwrap();

    assert_eq!(result.delivery_id, "del_abc");
    assert_eq!(result.status, "accepted");
    assert_eq!(result.idempotency_key, "key-1");
}

#[tokio::test]
async fn client_send_auto_generates_idempotency_key() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/ingest/ep_456"))
        .respond_with(ResponseTemplate::new(202).set_body_json(json!({
            "deliveryId": "del_xyz",
            "idempotencyKey": "auto-generated",
            "status": "accepted"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = NahookClient::builder("nhk_us_test123")
        .base_url(server.uri())
        .build()
        .unwrap();

    let result = client
        .send(
            "ep_456",
            SendOptions {
                payload: json!({"order": "123"}),
                idempotency_key: None,
            },
        )
        .await
        .unwrap();

    assert_eq!(result.delivery_id, "del_xyz");
}

// ── Client: trigger ──

#[tokio::test]
async fn client_trigger_calls_correct_endpoint() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/ingest/event/order.paid"))
        .and(header("authorization", "Bearer nhk_us_test123"))
        .respond_with(ResponseTemplate::new(202).set_body_json(json!({
            "eventTypeId": "evt_abc",
            "deliveryIds": ["del_1", "del_2"],
            "status": "accepted"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = NahookClient::builder("nhk_us_test123")
        .base_url(server.uri())
        .build()
        .unwrap();

    let result = client
        .trigger(
            "order.paid",
            TriggerOptions {
                payload: json!({"orderId": "123"}),
                metadata: None,
            },
        )
        .await
        .unwrap();

    assert_eq!(result.event_type_id, "evt_abc");
    assert_eq!(result.delivery_ids, vec!["del_1", "del_2"]);
    assert_eq!(result.status, "accepted");
}

// ── Client: send_batch ──

#[tokio::test]
async fn client_send_batch_calls_correct_endpoint() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/ingest/batch"))
        .respond_with(ResponseTemplate::new(202).set_body_json(json!({
            "items": [{
                "index": 0,
                "deliveryId": "del_abc",
                "status": "accepted"
            }]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = NahookClient::builder("nhk_us_test123")
        .base_url(server.uri())
        .build()
        .unwrap();

    let result = client
        .send_batch(vec![SendBatchItem {
            endpoint_id: "ep_123".to_string(),
            payload: json!({"test": true}),
            idempotency_key: None,
        }])
        .await
        .unwrap();

    assert_eq!(result.items.len(), 1);
    assert_eq!(result.items[0].delivery_id.as_deref(), Some("del_abc"));
    assert_eq!(result.items[0].status.as_deref(), Some("accepted"));
}

// ── Client: trigger_batch ──

#[tokio::test]
async fn client_trigger_batch_calls_correct_endpoint() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/ingest/event/batch"))
        .respond_with(ResponseTemplate::new(202).set_body_json(json!({
            "items": [{
                "index": 0,
                "eventTypeId": "evt_abc",
                "deliveryIds": [],
                "status": "accepted"
            }]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = NahookClient::builder("nhk_us_test123")
        .base_url(server.uri())
        .build()
        .unwrap();

    let result = client
        .trigger_batch(vec![TriggerBatchItem {
            event_type: "order.paid".to_string(),
            payload: json!({"orderId": "123"}),
            metadata: None,
        }])
        .await
        .unwrap();

    assert_eq!(result.items.len(), 1);
    assert_eq!(
        result.items[0].event_type_id.as_deref(),
        Some("evt_abc")
    );
}

// ── Client: error handling ──

#[tokio::test]
async fn client_returns_api_error_on_404() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/ingest/ep_missing"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "error": {
                "code": "not_found",
                "message": "Endpoint not found"
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = NahookClient::builder("nhk_us_test123")
        .base_url(server.uri())
        .build()
        .unwrap();

    let err = client
        .send(
            "ep_missing",
            SendOptions {
                payload: json!({}),
                idempotency_key: None,
            },
        )
        .await
        .unwrap_err();

    match err {
        nahook::NahookError::Api(api_err) => {
            assert_eq!(api_err.status, 404);
            assert_eq!(api_err.code, "not_found");
            assert_eq!(api_err.message, "Endpoint not found");
            assert!(api_err.is_not_found());
            assert!(!api_err.is_retryable());
        }
        _ => panic!("Expected ApiError, got: {err:?}"),
    }
}

#[tokio::test]
async fn client_returns_api_error_on_429() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/ingest/ep_rate"))
        .respond_with(
            ResponseTemplate::new(429)
                .set_body_json(json!({
                    "error": {
                        "code": "rate_limited",
                        "message": "Too many requests"
                    }
                }))
                .insert_header("retry-after", "5"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = NahookClient::builder("nhk_us_test123")
        .base_url(server.uri())
        .build()
        .unwrap();

    let err = client
        .send(
            "ep_rate",
            SendOptions {
                payload: json!({}),
                idempotency_key: None,
            },
        )
        .await
        .unwrap_err();

    match err {
        nahook::NahookError::Api(api_err) => {
            assert_eq!(api_err.status, 429);
            assert!(api_err.is_rate_limited());
            assert!(api_err.is_retryable());
            assert_eq!(api_err.retry_after, Some(5));
        }
        _ => panic!("Expected ApiError, got: {err:?}"),
    }
}

// ── Management: endpoints ──

#[tokio::test]
async fn management_endpoints_list() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/management/v1/workspaces/ws_123/endpoints"))
        .and(header("authorization", "Bearer nhm_test123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {
                "id": "ep_abc",
                "url": "https://example.com/webhook",
                "description": null,
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

    let mgmt = NahookManagement::builder("nhm_test123")
        .base_url(server.uri())
        .build()
        .unwrap();

    let result = mgmt.endpoints().list("ws_123").await.unwrap();
    assert_eq!(result.data.len(), 1);
    assert_eq!(result.data[0].id, "ep_abc");
    assert_eq!(result.data[0].url, "https://example.com/webhook");
    assert!(result.data[0].is_active);
}

#[tokio::test]
async fn management_endpoints_create() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/management/v1/workspaces/ws_123/endpoints"))
        .and(header("content-type", "application/json"))
        .and(body_json(json!({
            "url": "https://example.com/hook"
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "id": "ep_new",
            "url": "https://example.com/hook",
            "description": null,
            "isActive": true,
            "type": "webhook",
            "config": {},
            "createdAt": "2024-01-01T00:00:00Z",
            "updatedAt": "2024-01-01T00:00:00Z"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let mgmt = NahookManagement::builder("nhm_test123")
        .base_url(server.uri())
        .build()
        .unwrap();

    let result = mgmt
        .endpoints()
        .create(
            "ws_123",
            CreateEndpointOptions {
                url: "https://example.com/hook".to_string(),
                endpoint_type: None,
                description: None,
                metadata: None,
                config: None,
                auth_username: None,
                auth_password: None,
            },
        )
        .await
        .unwrap();

    assert_eq!(result.id, "ep_new");
    assert_eq!(result.url, "https://example.com/hook");
}

#[tokio::test]
async fn management_endpoints_get() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/management/v1/workspaces/ws_123/endpoints/ep_abc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "ep_abc",
            "url": "https://example.com/webhook",
            "description": "Test endpoint",
            "isActive": true,
            "type": "webhook",
            "config": {},
            "createdAt": "2024-01-01T00:00:00Z",
            "updatedAt": "2024-01-01T00:00:00Z"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let mgmt = NahookManagement::builder("nhm_test123")
        .base_url(server.uri())
        .build()
        .unwrap();

    let result = mgmt.endpoints().get("ws_123", "ep_abc").await.unwrap();
    assert_eq!(result.id, "ep_abc");
    assert_eq!(result.description, Some("Test endpoint".to_string()));
}

#[tokio::test]
async fn management_endpoints_update() {
    let server = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path("/management/v1/workspaces/ws_123/endpoints/ep_abc"))
        .and(body_json(json!({
            "description": "Updated"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "ep_abc",
            "url": "https://example.com/webhook",
            "description": "Updated",
            "isActive": true,
            "type": "webhook",
            "config": {},
            "createdAt": "2024-01-01T00:00:00Z",
            "updatedAt": "2024-01-02T00:00:00Z"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let mgmt = NahookManagement::builder("nhm_test123")
        .base_url(server.uri())
        .build()
        .unwrap();

    let result = mgmt
        .endpoints()
        .update(
            "ws_123",
            "ep_abc",
            UpdateEndpointOptions {
                url: None,
                description: Some("Updated".to_string()),
                metadata: None,
                is_active: None,
            },
        )
        .await
        .unwrap();

    assert_eq!(result.description, Some("Updated".to_string()));
}

#[tokio::test]
async fn management_endpoints_delete() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/management/v1/workspaces/ws_123/endpoints/ep_abc"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let mgmt = NahookManagement::builder("nhm_test123")
        .base_url(server.uri())
        .build()
        .unwrap();

    mgmt.endpoints()
        .delete("ws_123", "ep_abc")
        .await
        .unwrap();
}

// ── Management: event types ──

#[tokio::test]
async fn management_event_types_list() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/management/v1/workspaces/ws_123/event-types"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {
                "id": "evt_abc",
                "name": "order.paid",
                "description": null,
                "createdAt": "2024-01-01T00:00:00Z"
            }
        ])))
        .expect(1)
        .mount(&server)
        .await;

    let mgmt = NahookManagement::builder("nhm_test123")
        .base_url(server.uri())
        .build()
        .unwrap();

    let result = mgmt.event_types().list("ws_123").await.unwrap();
    assert_eq!(result.data.len(), 1);
    assert_eq!(result.data[0].name, "order.paid");
}

#[tokio::test]
async fn management_event_types_create() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/management/v1/workspaces/ws_123/event-types"))
        .and(body_json(json!({
            "name": "order.shipped"
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "id": "evt_new",
            "name": "order.shipped",
            "description": null,
            "createdAt": "2024-01-01T00:00:00Z"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let mgmt = NahookManagement::builder("nhm_test123")
        .base_url(server.uri())
        .build()
        .unwrap();

    let result = mgmt
        .event_types()
        .create(
            "ws_123",
            CreateEventTypeOptions {
                name: "order.shipped".to_string(),
                description: None,
            },
        )
        .await
        .unwrap();

    assert_eq!(result.name, "order.shipped");
}

// ── Management: applications ──

#[tokio::test]
async fn management_applications_list() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/management/v1/workspaces/ws_123/applications"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {
                "id": "app_abc",
                "externalId": null,
                "name": "My App",
                "metadata": {},
                "createdAt": "2024-01-01T00:00:00Z",
                "updatedAt": "2024-01-01T00:00:00Z"
            }
        ])))
        .expect(1)
        .mount(&server)
        .await;

    let mgmt = NahookManagement::builder("nhm_test123")
        .base_url(server.uri())
        .build()
        .unwrap();

    let result = mgmt
        .applications()
        .list("ws_123", None)
        .await
        .unwrap();

    assert_eq!(result.data.len(), 1);
    assert_eq!(result.data[0].name, "My App");
}

#[tokio::test]
async fn management_applications_create() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/management/v1/workspaces/ws_123/applications"))
        .and(body_json(json!({
            "name": "New App"
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "id": "app_new",
            "externalId": null,
            "name": "New App",
            "metadata": {},
            "createdAt": "2024-01-01T00:00:00Z",
            "updatedAt": "2024-01-01T00:00:00Z"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let mgmt = NahookManagement::builder("nhm_test123")
        .base_url(server.uri())
        .build()
        .unwrap();

    let result = mgmt
        .applications()
        .create(
            "ws_123",
            CreateApplicationOptions {
                name: "New App".to_string(),
                external_id: None,
                metadata: None,
            },
        )
        .await
        .unwrap();

    assert_eq!(result.name, "New App");
}

#[tokio::test]
async fn management_applications_list_endpoints() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(
            "/management/v1/workspaces/ws_123/applications/app_abc/endpoints",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {
                "id": "ep_in_app",
                "url": "https://example.com/app-hook",
                "description": null,
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

    let mgmt = NahookManagement::builder("nhm_test123")
        .base_url(server.uri())
        .build()
        .unwrap();

    let result = mgmt
        .applications()
        .list_endpoints("ws_123", "app_abc")
        .await
        .unwrap();

    assert_eq!(result.data.len(), 1);
    assert_eq!(result.data[0].id, "ep_in_app");
}

// ── Management: subscriptions ──

#[tokio::test]
async fn management_subscriptions_list() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(
            "/management/v1/workspaces/ws_123/endpoints/ep_abc/subscriptions",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {
                "id": "sub_123",
                "endpointId": "ep_abc",
                "eventTypeId": "evt_order",
                "createdAt": "2024-01-01T00:00:00Z"
            }
        ])))
        .expect(1)
        .mount(&server)
        .await;

    let mgmt = NahookManagement::builder("nhm_test123")
        .base_url(server.uri())
        .build()
        .unwrap();

    let result = mgmt
        .subscriptions()
        .list("ws_123", "ep_abc")
        .await
        .unwrap();

    assert_eq!(result.data.len(), 1);
    assert_eq!(result.data[0].endpoint_id, "ep_abc");
}

#[tokio::test]
async fn management_subscriptions_create() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path(
            "/management/v1/workspaces/ws_123/endpoints/ep_abc/subscriptions",
        ))
        .and(body_json(json!({
            "eventTypeId": "evt_order"
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "id": "sub_new",
            "endpointId": "ep_abc",
            "eventTypeId": "evt_order",
            "createdAt": "2024-01-01T00:00:00Z"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let mgmt = NahookManagement::builder("nhm_test123")
        .base_url(server.uri())
        .build()
        .unwrap();

    let result = mgmt
        .subscriptions()
        .create(
            "ws_123",
            "ep_abc",
            CreateSubscriptionOptions {
                event_type_id: "evt_order".to_string(),
            },
        )
        .await
        .unwrap();

    assert_eq!(result.event_type_id, "evt_order");
}

#[tokio::test]
async fn management_subscriptions_delete() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path(
            "/management/v1/workspaces/ws_123/endpoints/ep_abc/subscriptions/evt_order",
        ))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let mgmt = NahookManagement::builder("nhm_test123")
        .base_url(server.uri())
        .build()
        .unwrap();

    mgmt.subscriptions()
        .delete("ws_123", "ep_abc", "evt_order")
        .await
        .unwrap();
}

// ── Management: portal sessions ──

#[tokio::test]
async fn management_portal_sessions_create() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path(
            "/management/v1/workspaces/ws_123/applications/app_abc/portal",
        ))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "url": "https://portal.nahook.com/session/abc",
            "code": "portal_code_123",
            "expiresAt": "2024-01-02T00:00:00Z"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let mgmt = NahookManagement::builder("nhm_test123")
        .base_url(server.uri())
        .build()
        .unwrap();

    let result = mgmt
        .portal_sessions()
        .create("ws_123", "app_abc", None)
        .await
        .unwrap();

    assert_eq!(result.url, "https://portal.nahook.com/session/abc");
    assert_eq!(result.code, "portal_code_123");
}

// ── Error type helpers ──

#[test]
fn api_error_helpers() {
    use nahook::error::ApiError;

    let err_500 = ApiError {
        status: 500,
        code: "internal".to_string(),
        message: "Server error".to_string(),
        retry_after: None,
    };
    assert!(err_500.is_retryable());
    assert!(!err_500.is_auth_error());

    let err_429 = ApiError {
        status: 429,
        code: "rate_limited".to_string(),
        message: "Too many requests".to_string(),
        retry_after: Some(5),
    };
    assert!(err_429.is_retryable());
    assert!(err_429.is_rate_limited());

    let err_401 = ApiError {
        status: 401,
        code: "unauthorized".to_string(),
        message: "Invalid token".to_string(),
        retry_after: None,
    };
    assert!(err_401.is_auth_error());
    assert!(!err_401.is_retryable());

    let err_403 = ApiError {
        status: 403,
        code: "token_disabled".to_string(),
        message: "Token disabled".to_string(),
        retry_after: None,
    };
    assert!(err_403.is_auth_error());

    let err_403_other = ApiError {
        status: 403,
        code: "forbidden".to_string(),
        message: "Forbidden".to_string(),
        retry_after: None,
    };
    assert!(!err_403_other.is_auth_error());

    let err_404 = ApiError {
        status: 404,
        code: "not_found".to_string(),
        message: "Not found".to_string(),
        retry_after: None,
    };
    assert!(err_404.is_not_found());

    let err_400 = ApiError {
        status: 400,
        code: "validation".to_string(),
        message: "Bad request".to_string(),
        retry_after: None,
    };
    assert!(err_400.is_validation_error());
}

// ── Error helper: granular tests ──

#[test]
fn api_error_500_is_retryable() {
    use nahook::error::ApiError;
    let err = ApiError {
        status: 500,
        code: "internal".to_string(),
        message: "Internal server error".to_string(),
        retry_after: None,
    };
    assert!(err.is_retryable());
}

#[test]
fn api_error_429_is_retryable() {
    use nahook::error::ApiError;
    let err = ApiError {
        status: 429,
        code: "rate_limited".to_string(),
        message: "Too many requests".to_string(),
        retry_after: Some(5),
    };
    assert!(err.is_retryable());
}

#[test]
fn api_error_404_not_retryable() {
    use nahook::error::ApiError;
    let err = ApiError {
        status: 404,
        code: "not_found".to_string(),
        message: "Not found".to_string(),
        retry_after: None,
    };
    assert!(!err.is_retryable());
}

#[test]
fn api_error_401_is_auth_error() {
    use nahook::error::ApiError;
    let err = ApiError {
        status: 401,
        code: "unauthorized".to_string(),
        message: "Unauthorized".to_string(),
        retry_after: None,
    };
    assert!(err.is_auth_error());
}

#[test]
fn api_error_404_is_not_found() {
    use nahook::error::ApiError;
    let err = ApiError {
        status: 404,
        code: "not_found".to_string(),
        message: "Not found".to_string(),
        retry_after: None,
    };
    assert!(err.is_not_found());
}

#[test]
fn api_error_429_is_rate_limited() {
    use nahook::error::ApiError;
    let err = ApiError {
        status: 429,
        code: "rate_limited".to_string(),
        message: "Rate limited".to_string(),
        retry_after: None,
    };
    assert!(err.is_rate_limited());
}

#[test]
fn api_error_400_is_validation_error() {
    use nahook::error::ApiError;
    let err = ApiError {
        status: 400,
        code: "validation".to_string(),
        message: "Bad request".to_string(),
        retry_after: None,
    };
    assert!(err.is_validation_error());
}

// ── URL percent-encoding ──

#[tokio::test]
async fn url_encodes_path_segments() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/management/v1/workspaces/ws%20123/endpoints"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .expect(1)
        .mount(&server)
        .await;

    let mgmt = NahookManagement::builder("nhm_test123")
        .base_url(server.uri())
        .build()
        .unwrap();

    let result = mgmt.endpoints().list("ws 123").await.unwrap();
    assert_eq!(result.data.len(), 0);
}

// ── User-Agent header ──

#[tokio::test]
async fn sends_correct_user_agent() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/management/v1/workspaces/ws_123/endpoints"))
        .and(header("user-agent", "nahook-rust/0.1.0"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .expect(1)
        .mount(&server)
        .await;

    let mgmt = NahookManagement::builder("nhm_test123")
        .base_url(server.uri())
        .build()
        .unwrap();

    mgmt.endpoints().list("ws_123").await.unwrap();
}
