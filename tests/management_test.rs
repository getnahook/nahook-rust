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
    assert_eq!(result.items[0].event_type_id.as_deref(), Some("evt_abc"));
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
                environment_id: None,
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

    mgmt.endpoints().delete("ws_123", "ep_abc").await.unwrap();
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

    let result = mgmt.applications().list("ws_123", None).await.unwrap();

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
                ..Default::default()
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

// ── Management: applications — maxEndpoints + showEventTypes (tri-state) ──

#[tokio::test]
async fn management_applications_create_omits_unset_cap_fields() {
    let server = MockServer::start().await;

    // Exact body match: a create with neither cap field set must send
    // exactly {"name": ...} — named here so the coverage survives even if
    // the base create test's matcher is ever loosened.
    Mock::given(method("POST"))
        .and(path("/management/v1/workspaces/ws_123/applications"))
        .and(body_json(json!({
            "name": "Plain App"
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "id": "app_new",
            "externalId": null,
            "name": "Plain App",
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
                name: "Plain App".to_string(),
                ..Default::default()
            },
        )
        .await
        .unwrap();

    assert_eq!(result.max_endpoints, None);
    assert!(result.show_event_types);
}

#[tokio::test]
async fn management_applications_create_with_max_endpoints() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/management/v1/workspaces/ws_123/applications"))
        .and(body_json(json!({
            "name": "Capped App",
            "maxEndpoints": 2
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "id": "app_new",
            "externalId": null,
            "name": "Capped App",
            "metadata": {},
            "maxEndpoints": 2,
            "showEventTypes": true,
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
                name: "Capped App".to_string(),
                max_endpoints: Some(2),
                ..Default::default()
            },
        )
        .await
        .unwrap();

    assert_eq!(result.max_endpoints, Some(2));
    assert!(result.show_event_types);
}

#[tokio::test]
async fn management_applications_create_with_show_event_types_false() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/management/v1/workspaces/ws_123/applications"))
        .and(body_json(json!({
            "name": "Hidden App",
            "showEventTypes": false
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "id": "app_new",
            "externalId": null,
            "name": "Hidden App",
            "metadata": {},
            "maxEndpoints": null,
            "showEventTypes": false,
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
                name: "Hidden App".to_string(),
                show_event_types: Some(false),
                ..Default::default()
            },
        )
        .await
        .unwrap();

    assert_eq!(result.max_endpoints, None);
    assert!(!result.show_event_types);
}

#[tokio::test]
async fn management_applications_update_max_endpoints_explicit_null() {
    let server = MockServer::start().await;

    // Some(None) must serialize as an explicit JSON null (clears the cap);
    // body_json is an exact match, so this also pins that nothing else leaks in.
    Mock::given(method("PATCH"))
        .and(path(
            "/management/v1/workspaces/ws_123/applications/app_abc",
        ))
        .and(body_json(json!({
            "maxEndpoints": null
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "app_abc",
            "externalId": null,
            "name": "My App",
            "metadata": {},
            "maxEndpoints": null,
            "showEventTypes": true,
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
        .update(
            "ws_123",
            "app_abc",
            UpdateApplicationOptions {
                max_endpoints: Some(None),
                ..Default::default()
            },
        )
        .await
        .unwrap();

    assert_eq!(result.max_endpoints, None);
}

#[tokio::test]
async fn management_applications_update_omits_unset_cap_fields() {
    let server = MockServer::start().await;

    // None on max_endpoints/show_event_types must be omitted entirely —
    // exact body match proves only "name" is sent.
    Mock::given(method("PATCH"))
        .and(path(
            "/management/v1/workspaces/ws_123/applications/app_abc",
        ))
        .and(body_json(json!({
            "name": "Renamed"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "app_abc",
            "externalId": null,
            "name": "Renamed",
            "metadata": {},
            "maxEndpoints": 5,
            "showEventTypes": true,
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
        .update(
            "ws_123",
            "app_abc",
            UpdateApplicationOptions {
                name: Some("Renamed".to_string()),
                ..Default::default()
            },
        )
        .await
        .unwrap();

    assert_eq!(result.max_endpoints, Some(5));
}

#[tokio::test]
async fn management_applications_update_max_endpoints_set_value() {
    let server = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path(
            "/management/v1/workspaces/ws_123/applications/app_abc",
        ))
        .and(body_json(json!({
            "maxEndpoints": 5,
            "showEventTypes": false
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "app_abc",
            "externalId": null,
            "name": "My App",
            "metadata": {},
            "maxEndpoints": 5,
            "showEventTypes": false,
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
        .update(
            "ws_123",
            "app_abc",
            UpdateApplicationOptions {
                max_endpoints: Some(Some(5)),
                show_event_types: Some(false),
                ..Default::default()
            },
        )
        .await
        .unwrap();

    assert_eq!(result.max_endpoints, Some(5));
    assert!(!result.show_event_types);
}

#[tokio::test]
async fn management_applications_response_defaults_show_event_types_when_absent() {
    let server = MockServer::start().await;

    // Older fixtures / responses without the new fields still deserialize:
    // max_endpoints -> None, show_event_types -> true (server default).
    Mock::given(method("GET"))
        .and(path(
            "/management/v1/workspaces/ws_123/applications/app_abc",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "app_abc",
            "externalId": null,
            "name": "My App",
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

    let result = mgmt.applications().get("ws_123", "app_abc").await.unwrap();

    assert_eq!(result.max_endpoints, None);
    assert!(result.show_event_types);
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
                "eventTypeId": "evt_order",
                "eventTypeName": "order.created",
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

    let result = mgmt.subscriptions().list("ws_123", "ep_abc").await.unwrap();

    assert_eq!(result.data.len(), 1);
    assert_eq!(result.data[0].event_type_id, "evt_order");
    assert_eq!(result.data[0].event_type_name, "order.created");
}

#[tokio::test]
async fn management_subscriptions_create() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path(
            "/management/v1/workspaces/ws_123/endpoints/ep_abc/subscriptions",
        ))
        .and(body_json(json!({
            "eventTypeIds": ["evt_order"]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "subscribed": 1
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
                event_type_ids: vec!["evt_order".to_string()],
            },
        )
        .await
        .unwrap();

    assert_eq!(result.subscribed, 1);
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

// ── Management: environments ──

#[tokio::test]
async fn management_environments_list() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/management/v1/workspaces/ws_123/environments"))
        .and(header("authorization", "Bearer nhm_test123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {
                "id": "env_abc",
                "name": "Production",
                "slug": "production",
                "isDefault": true,
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

    let result = mgmt.environments().list("ws_123").await.unwrap();
    assert_eq!(result.data.len(), 1);
    assert_eq!(result.data[0].id, "env_abc");
    assert_eq!(result.data[0].name, "Production");
    assert!(result.data[0].is_default);
}

#[tokio::test]
async fn management_environments_create() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/management/v1/workspaces/ws_123/environments"))
        .and(header("content-type", "application/json"))
        .and(body_json(json!({
            "name": "Staging",
            "slug": "staging"
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "id": "env_new",
            "name": "Staging",
            "slug": "staging",
            "isDefault": false,
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
        .environments()
        .create(
            "ws_123",
            CreateEnvironmentOptions {
                name: "Staging".to_string(),
                slug: "staging".to_string(),
            },
        )
        .await
        .unwrap();

    assert_eq!(result.id, "env_new");
    assert_eq!(result.slug, "staging");
    assert!(!result.is_default);
}

#[tokio::test]
async fn management_environments_get() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(
            "/management/v1/workspaces/ws_123/environments/env_abc",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "env_abc",
            "name": "Production",
            "slug": "production",
            "isDefault": true,
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

    let result = mgmt.environments().get("ws_123", "env_abc").await.unwrap();
    assert_eq!(result.id, "env_abc");
    assert_eq!(result.name, "Production");
}

#[tokio::test]
async fn management_environments_update() {
    let server = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path(
            "/management/v1/workspaces/ws_123/environments/env_abc",
        ))
        .and(body_json(json!({
            "name": "Pre-production"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "env_abc",
            "name": "Pre-production",
            "slug": "production",
            "isDefault": true,
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
        .environments()
        .update(
            "ws_123",
            "env_abc",
            UpdateEnvironmentOptions {
                name: Some("Pre-production".to_string()),
            },
        )
        .await
        .unwrap();

    assert_eq!(result.name, "Pre-production");
}

#[tokio::test]
async fn management_environments_delete() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path(
            "/management/v1/workspaces/ws_123/environments/env_abc",
        ))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let mgmt = NahookManagement::builder("nhm_test123")
        .base_url(server.uri())
        .build()
        .unwrap();

    mgmt.environments()
        .delete("ws_123", "env_abc")
        .await
        .unwrap();
}

#[tokio::test]
async fn management_environments_list_event_type_visibility() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(
            "/management/v1/workspaces/ws_123/environments/env_abc/event-types",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {
                "eventTypeId": "evt_order",
                "eventTypeName": "order.created",
                "published": true
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
        .environments()
        .list_event_type_visibility("ws_123", "env_abc")
        .await
        .unwrap();

    assert_eq!(result.data.len(), 1);
    assert_eq!(result.data[0].event_type_name, "order.created");
    assert!(result.data[0].published);
}

#[tokio::test]
async fn management_environments_set_event_type_visibility() {
    let server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path(
            "/management/v1/workspaces/ws_123/environments/env_abc/event-types/evt_order/visibility",
        ))
        .and(header("content-type", "application/json"))
        .and(body_json(json!({
            "published": true
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "eventTypeId": "evt_order",
            "eventTypeName": "order.created",
            "published": true
        })))
        .expect(1)
        .mount(&server)
        .await;

    let mgmt = NahookManagement::builder("nhm_test123")
        .base_url(server.uri())
        .build()
        .unwrap();

    let result = mgmt
        .environments()
        .set_event_type_visibility(
            "ws_123",
            "env_abc",
            "evt_order",
            SetVisibilityOptions { published: true },
        )
        .await
        .unwrap();

    assert_eq!(result.event_type_name, "order.created");
    assert!(result.published);
}

// ── Management: deliveries ──

#[tokio::test]
async fn list_returns_paginated_data_and_next_cursor() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(
            "/management/v1/workspaces/ws_abc/endpoints/ep_1/deliveries",
        ))
        .and(header("authorization", "Bearer nhm_test123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "deliveries": [
                {
                    "id": "del_a",
                    "idempotencyKey": "k1",
                    "endpointId": "ep_1",
                    "status": "delivered",
                    "totalAttempts": 1,
                    "firstAttemptAt": "2026-05-28T14:30:59Z",
                    "deliveredAt": "2026-05-28T14:30:59Z",
                    "nextRetryAt": null,
                    "hasPayload": true,
                    "createdAt": "2026-05-28T14:30:59Z",
                    "updatedAt": "2026-05-28T14:30:59Z"
                },
                {
                    "id": "del_b",
                    "idempotencyKey": "k2",
                    "endpointId": "ep_1",
                    "status": "failed",
                    "totalAttempts": 3,
                    "firstAttemptAt": "2026-05-28T14:31:00Z",
                    "deliveredAt": null,
                    "nextRetryAt": null,
                    "hasPayload": false,
                    "createdAt": "2026-05-28T14:31:00Z",
                    "updatedAt": "2026-05-28T14:31:00Z"
                }
            ],
            "nextCursor": "opaque-token-aaa"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let mgmt = NahookManagement::builder("nhm_test123")
        .base_url(server.uri())
        .build()
        .unwrap();

    let result = mgmt
        .deliveries()
        .list("ws_abc", "ep_1", None)
        .await
        .unwrap();

    assert_eq!(result.data.len(), 2);
    assert_eq!(result.data[0].id, "del_a");
    assert_eq!(result.data[1].id, "del_b");
    assert_eq!(result.next_cursor.as_deref(), Some("opaque-token-aaa"));
}

#[tokio::test]
async fn list_returns_null_cursor_when_last_page() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(
            "/management/v1/workspaces/ws_abc/endpoints/ep_1/deliveries",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "deliveries": [],
            "nextCursor": null
        })))
        .expect(1)
        .mount(&server)
        .await;

    let mgmt = NahookManagement::builder("nhm_test123")
        .base_url(server.uri())
        .build()
        .unwrap();

    let result = mgmt
        .deliveries()
        .list("ws_abc", "ep_1", None)
        .await
        .unwrap();

    assert!(result.data.is_empty());
    assert!(result.next_cursor.is_none());
}

#[tokio::test]
async fn list_forwards_query_params() {
    use wiremock::matchers::query_param;
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(
            "/management/v1/workspaces/ws_abc/endpoints/ep_1/deliveries",
        ))
        .and(query_param("limit", "25"))
        .and(query_param("cursor", "opaque-token-xyz"))
        .and(query_param("status", "failed"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "deliveries": [],
            "nextCursor": null
        })))
        .expect(1)
        .mount(&server)
        .await;

    let mgmt = NahookManagement::builder("nhm_test123")
        .base_url(server.uri())
        .build()
        .unwrap();

    mgmt.deliveries()
        .list(
            "ws_abc",
            "ep_1",
            Some(ListDeliveriesOptions {
                limit: Some(25),
                cursor: Some("opaque-token-xyz".to_string()),
                status: Some("failed".to_string()),
            }),
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn list_omits_unset_query_params() {
    use wiremock::matchers::query_param_is_missing;
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(
            "/management/v1/workspaces/ws_abc/endpoints/ep_1/deliveries",
        ))
        .and(query_param_is_missing("limit"))
        .and(query_param_is_missing("cursor"))
        .and(query_param_is_missing("status"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "deliveries": [],
            "nextCursor": null
        })))
        .expect(1)
        .mount(&server)
        .await;

    let mgmt = NahookManagement::builder("nhm_test123")
        .base_url(server.uri())
        .build()
        .unwrap();

    mgmt.deliveries()
        .list("ws_abc", "ep_1", None)
        .await
        .unwrap();
}

#[tokio::test]
async fn get_returns_metadata_without_envelope_by_default() {
    use wiremock::matchers::query_param_is_missing;
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/management/v1/workspaces/ws_abc/deliveries/del_a"))
        .and(query_param_is_missing("include"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "del_a",
            "idempotencyKey": "k1",
            "endpointId": "ep_1",
            "status": "delivered",
            "totalAttempts": 1,
            "firstAttemptAt": "2026-05-28T14:30:59Z",
            "deliveredAt": "2026-05-28T14:30:59Z",
            "nextRetryAt": null,
            "hasPayload": true,
            "createdAt": "2026-05-28T14:30:59Z",
            "updatedAt": "2026-05-28T14:30:59Z"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let mgmt = NahookManagement::builder("nhm_test123")
        .base_url(server.uri())
        .build()
        .unwrap();

    let delivery = mgmt
        .deliveries()
        .get("ws_abc", "del_a", None)
        .await
        .unwrap();

    assert_eq!(delivery.id, "del_a");
    assert!(delivery.has_payload);
    assert!(delivery.payload.is_none());
}

#[tokio::test]
async fn get_with_include_payload_returns_envelope() {
    use wiremock::matchers::query_param;
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/management/v1/workspaces/ws_abc/deliveries/del_a"))
        .and(query_param("include", "payload"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "del_a",
            "idempotencyKey": "k1",
            "endpointId": "ep_1",
            "status": "delivered",
            "totalAttempts": 1,
            "firstAttemptAt": "2026-05-28T14:30:59Z",
            "deliveredAt": "2026-05-28T14:30:59Z",
            "nextRetryAt": null,
            "hasPayload": true,
            "createdAt": "2026-05-28T14:30:59Z",
            "updatedAt": "2026-05-28T14:30:59Z",
            "payload": {
                "status": "available",
                "data": { "orderId": "ord_123" },
                "contentType": "application/json"
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let mgmt = NahookManagement::builder("nhm_test123")
        .base_url(server.uri())
        .build()
        .unwrap();

    let delivery = mgmt
        .deliveries()
        .get(
            "ws_abc",
            "del_a",
            Some(GetDeliveryOptions {
                include_payload: true,
            }),
        )
        .await
        .unwrap();

    match delivery.payload {
        Some(PayloadEnvelope::Available { data, content_type }) => {
            assert_eq!(data, json!({ "orderId": "ord_123" }));
            assert_eq!(content_type, "application/json");
        }
        other => panic!("Expected Available envelope, got: {:?}", other),
    }
}

#[tokio::test]
async fn get_returns_forbidden_envelope_for_plan_gated_workspace() {
    use wiremock::matchers::query_param;
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/management/v1/workspaces/ws_abc/deliveries/del_a"))
        .and(query_param("include", "payload"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "del_a",
            "idempotencyKey": "k1",
            "endpointId": "ep_1",
            "status": "delivered",
            "totalAttempts": 1,
            "firstAttemptAt": null,
            "deliveredAt": "2026-05-28T14:30:59Z",
            "nextRetryAt": null,
            "hasPayload": true,
            "createdAt": "2026-05-28T14:30:59Z",
            "updatedAt": "2026-05-28T14:30:59Z",
            "payload": { "status": "forbidden" }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let mgmt = NahookManagement::builder("nhm_test123")
        .base_url(server.uri())
        .build()
        .unwrap();

    let delivery = mgmt
        .deliveries()
        .get(
            "ws_abc",
            "del_a",
            Some(GetDeliveryOptions {
                include_payload: true,
            }),
        )
        .await
        .unwrap();

    assert!(matches!(delivery.payload, Some(PayloadEnvelope::Forbidden)));
}

#[tokio::test]
async fn get_attempts_returns_array() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(
            "/management/v1/workspaces/ws_abc/deliveries/del_a/attempts",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {
                "id": "att_1",
                "attemptNumber": 1,
                "status": "failed",
                "responseStatusCode": 502,
                "responseTimeMs": 142,
                "errorMessage": "Bad gateway",
                "createdAt": "2026-05-28T14:31:00Z"
            },
            {
                "id": "att_2",
                "attemptNumber": 2,
                "status": "success",
                "responseStatusCode": 200,
                "responseTimeMs": 88,
                "errorMessage": null,
                "createdAt": "2026-05-28T14:31:30Z"
            }
        ])))
        .expect(1)
        .mount(&server)
        .await;

    let mgmt = NahookManagement::builder("nhm_test123")
        .base_url(server.uri())
        .build()
        .unwrap();

    let attempts = mgmt
        .deliveries()
        .get_attempts("ws_abc", "del_a")
        .await
        .unwrap();

    assert_eq!(attempts.len(), 2);
    assert_eq!(attempts[0].attempt_number, 1);
    assert_eq!(attempts[0].response_status_code, Some(502));
    assert_eq!(attempts[1].status, "success");
    assert_eq!(attempts[1].error_message, None);
}

// ── PayloadEnvelope forward-compatibility (NAH-163) ──

#[test]
fn payload_envelope_unknown_status_preserves_status_string() {
    // Simulate the server adding a sixth envelope status the SDK doesn't
    // know about. The strict tagged-enum derive would have failed here;
    // the manual impl maps it to Unknown { status } and preserves the
    // original string for callers to log or branch on.
    let json_str = r#"{"status": "quarantined"}"#;
    let envelope: PayloadEnvelope = serde_json::from_str(json_str).unwrap();
    match envelope {
        PayloadEnvelope::Unknown { status } => assert_eq!(status, "quarantined"),
        other => panic!("Expected Unknown {{ status: \"quarantined\" }}, got: {other:?}"),
    }
}

#[test]
fn payload_envelope_known_statuses_round_trip() {
    // Regression: each known variant must serialize to the same wire shape
    // the server produces, and re-deserialize to the same variant. Catches
    // the case where the manual Serialize impl drifts from the original
    // tagged behaviour.
    let cases = [
        (
            r#"{"status":"available","data":{"orderId":"ord_1"},"contentType":"application/json"}"#,
            "available",
        ),
        (r#"{"status":"forbidden"}"#, "forbidden"),
        (r#"{"status":"processing"}"#, "processing"),
        (r#"{"status":"not_found"}"#, "not_found"),
        (r#"{"status":"error"}"#, "error"),
    ];
    for (input, expected_status) in cases {
        let envelope: PayloadEnvelope = serde_json::from_str(input).unwrap();
        let re_serialized = serde_json::to_value(&envelope).unwrap();
        assert_eq!(
            re_serialized["status"].as_str(),
            Some(expected_status),
            "round-trip failed for {input}",
        );
        // For Available, also assert the data and contentType fields survive
        // — the status assertion alone misses a symmetric data↔contentType
        // swap in Serialize.
        if expected_status == "available" {
            assert_eq!(re_serialized["data"]["orderId"], "ord_1");
            assert_eq!(re_serialized["contentType"], "application/json");
        }
        // Re-deserializing the re-serialized form should yield the same
        // variant — ensures Serialize and Deserialize are inverses.
        let round_trip: PayloadEnvelope = serde_json::from_value(re_serialized).unwrap();
        // Use Debug equality via formatting (PayloadEnvelope doesn't impl PartialEq).
        assert_eq!(format!("{round_trip:?}"), format!("{envelope:?}"));
    }
}

#[test]
fn payload_envelope_unknown_status_round_trip_preserves_original_string() {
    // Regression: an Unknown variant must serialize back to the original
    // status string verbatim — not "unknown", not a debug repr. If the
    // Serialize impl drifts to emit "unknown" literally, this catches it.
    let unknown = PayloadEnvelope::Unknown {
        status: "quarantined".to_string(),
    };
    let json = serde_json::to_string(&unknown).unwrap();
    assert_eq!(json, r#"{"status":"quarantined"}"#);

    // And deserializing the re-emitted form lands back in Unknown.
    let round_trip: PayloadEnvelope = serde_json::from_str(&json).unwrap();
    match round_trip {
        PayloadEnvelope::Unknown { status } => assert_eq!(status, "quarantined"),
        other => panic!("Expected Unknown after round-trip, got: {other:?}"),
    }
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

// ── ERR-05: 403 + token_disabled is auth error ──

#[test]
fn api_error_403_token_disabled_is_auth_error() {
    use nahook::error::ApiError;
    let err = ApiError {
        status: 403,
        code: "token_disabled".to_string(),
        message: "Token disabled".to_string(),
        retry_after: None,
    };
    assert!(err.is_auth_error());
    assert!(!err.is_retryable());
}

// ── ERR-06: 403 + other code is NOT auth error ──

#[test]
fn api_error_403_other_code_is_not_auth_error() {
    use nahook::error::ApiError;
    let err = ApiError {
        status: 403,
        code: "forbidden".to_string(),
        message: "Forbidden".to_string(),
        retry_after: None,
    };
    assert!(!err.is_auth_error());
}

// ── ERR-10: Network error wraps original cause ──

#[tokio::test]
async fn network_error_wraps_original_cause() {
    use nahook::error::{NahookError, NetworkError};

    // Build a reqwest::Error by making a request to an unreachable address
    let reqwest_err = reqwest::Client::new()
        .get("http://[::1]:1") // unreachable port
        .timeout(std::time::Duration::from_millis(1))
        .send()
        .await
        .unwrap_err();

    let network_err = NetworkError { cause: reqwest_err };

    // Verify the cause is accessible via Display
    let display = format!("{}", network_err);
    assert!(
        display.starts_with("Network error:"),
        "Expected 'Network error:' prefix, got: {display}"
    );

    // Verify it converts into NahookError::Network
    let nahook_err: NahookError = network_err.into();
    match &nahook_err {
        NahookError::Network(inner) => {
            // std::error::Error::source() should return the reqwest::Error
            use std::error::Error;
            assert!(
                inner.source().is_some(),
                "NetworkError.source() should return the wrapped reqwest::Error"
            );
        }
        _ => panic!("Expected NahookError::Network, got: {nahook_err:?}"),
    }
}

// ── ERR-11: Timeout error stores timeout value ──

#[test]
fn timeout_error_stores_timeout_value() {
    use nahook::error::{NahookError, TimeoutError};

    let timeout_err = TimeoutError { timeout_ms: 5000 };
    assert_eq!(timeout_err.timeout_ms, 5000);

    let display = format!("{}", timeout_err);
    assert_eq!(display, "Request timed out after 5000ms");

    // Verify it converts into NahookError::Timeout
    let nahook_err: NahookError = timeout_err.into();
    match &nahook_err {
        NahookError::Timeout(inner) => {
            assert_eq!(inner.timeout_ms, 5000);
        }
        _ => panic!("Expected NahookError::Timeout, got: {nahook_err:?}"),
    }
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
        .and(header(
            "user-agent",
            format!("nahook-rust/{}", env!("CARGO_PKG_VERSION")).as_str(),
        ))
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
