use nahook::types::{
    CreateApplicationOptions, CreateEndpointOptions, CreateEventTypeOptions,
    CreateSubscriptionOptions, UpdateApplicationOptions, UpdateEndpointOptions,
    UpdateEventTypeOptions, CreateSubscriptionResult,
};
use nahook::{NahookError, NahookManagement};

/// Test environment configuration loaded from env vars.
struct TestConfig {
    api_url: String,
    mgmt_token: String,
    workspace_id: String,
}

/// Attempts to load the test configuration from environment variables.
/// Returns `None` if any required variable is missing, printing a skip message.
fn load_config() -> Option<TestConfig> {
    let api_url = match std::env::var("NAHOOK_TEST_API_URL") {
        Ok(v) => v,
        Err(_) => {
            println!("NAHOOK_TEST_API_URL not set — skipping management integration tests");
            return None;
        }
    };
    let mgmt_token = match std::env::var("NAHOOK_TEST_MGMT_TOKEN") {
        Ok(v) => v,
        Err(_) => {
            println!("NAHOOK_TEST_MGMT_TOKEN not set — skipping management integration tests");
            return None;
        }
    };
    let workspace_id = match std::env::var("NAHOOK_TEST_WORKSPACE_ID") {
        Ok(v) => v,
        Err(_) => {
            println!("NAHOOK_TEST_WORKSPACE_ID not set — skipping management integration tests");
            return None;
        }
    };

    Some(TestConfig {
        api_url,
        mgmt_token,
        workspace_id,
    })
}

/// Build a NahookManagement client pointing at the test API.
fn build_mgmt(api_url: &str, token: &str) -> NahookManagement {
    NahookManagement::builder(token)
        .base_url(api_url)
        .build()
        .expect("failed to build NahookManagement")
}

/// Generate a unique suffix for test resource names.
fn unique_suffix() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .to_string()
}

// ── Event Types CRUD ──

#[tokio::test]
async fn test_event_types_crud() {
    let cfg = match load_config() {
        Some(c) => c,
        None => return,
    };

    let mgmt = build_mgmt(&cfg.api_url, &cfg.mgmt_token);
    let ws = &cfg.workspace_id;
    let suffix = unique_suffix();
    let event_name = format!("test.mgmt.evt.{}", suffix);

    // Create
    let created = mgmt
        .event_types()
        .create(
            ws,
            CreateEventTypeOptions {
                name: event_name.clone(),
                description: Some("integration test event type".to_string()),
            },
        )
        .await
        .expect("create event type should succeed");

    assert!(
        created.id.starts_with("evt_"),
        "event type id should start with 'evt_', got: {}",
        created.id
    );
    assert_eq!(created.name, event_name);
    assert_eq!(
        created.description.as_deref(),
        Some("integration test event type")
    );

    // List
    let list = mgmt
        .event_types()
        .list(ws)
        .await
        .expect("list event types should succeed");

    assert!(
        list.data.iter().any(|et| et.id == created.id),
        "created event type should appear in list"
    );

    // Get
    let fetched = mgmt
        .event_types()
        .get(ws, &created.id)
        .await
        .expect("get event type should succeed");

    assert_eq!(fetched.id, created.id);
    assert_eq!(fetched.name, event_name);

    // Update
    let updated = mgmt
        .event_types()
        .update(
            ws,
            &created.id,
            UpdateEventTypeOptions {
                description: Some("updated description".to_string()),
            },
        )
        .await
        .expect("update event type should succeed");

    assert_eq!(updated.id, created.id);
    assert_eq!(updated.description.as_deref(), Some("updated description"));

    // Delete
    mgmt.event_types()
        .delete(ws, &created.id)
        .await
        .expect("delete event type should succeed");

    // Verify deleted (get should 404)
    let err = mgmt
        .event_types()
        .get(ws, &created.id)
        .await
        .expect_err("get deleted event type should fail");

    match &err {
        NahookError::Api(api_err) => {
            assert_eq!(api_err.status, 404, "expected 404 after delete");
        }
        other => panic!("expected NahookError::Api, got: {:?}", other),
    }
}

// ── Endpoints CRUD ──

#[tokio::test]
async fn test_endpoints_crud() {
    let cfg = match load_config() {
        Some(c) => c,
        None => return,
    };

    let mgmt = build_mgmt(&cfg.api_url, &cfg.mgmt_token);
    let ws = &cfg.workspace_id;
    let suffix = unique_suffix();
    let url = format!("https://example.com/webhook/mgmt-test-{}", suffix);

    // Create
    let created = mgmt
        .endpoints()
        .create(
            ws,
            CreateEndpointOptions {
                url: url.clone(),
                endpoint_type: None,
                description: Some("integration test endpoint".to_string()),
                metadata: None,
                config: None,
                auth_username: None,
                auth_password: None,
            },
        )
        .await
        .expect("create endpoint should succeed");

    assert!(
        created.id.starts_with("ep_"),
        "endpoint id should start with 'ep_', got: {}",
        created.id
    );
    assert_eq!(created.url, url);
    assert!(created.is_active, "new endpoint should be active");

    // List
    let list = mgmt
        .endpoints()
        .list(ws)
        .await
        .expect("list endpoints should succeed");

    assert!(
        list.data.iter().any(|ep| ep.id == created.id),
        "created endpoint should appear in list"
    );

    // Get
    let fetched = mgmt
        .endpoints()
        .get(ws, &created.id)
        .await
        .expect("get endpoint should succeed");

    assert_eq!(fetched.id, created.id);
    assert_eq!(fetched.url, url);

    // Update
    let updated = mgmt
        .endpoints()
        .update(
            ws,
            &created.id,
            UpdateEndpointOptions {
                url: None,
                description: Some("updated endpoint description".to_string()),
                metadata: None,
                is_active: Some(false),
            },
        )
        .await
        .expect("update endpoint should succeed");

    assert_eq!(updated.id, created.id);
    assert_eq!(
        updated.description.as_deref(),
        Some("updated endpoint description")
    );
    assert!(!updated.is_active, "endpoint should be inactive after update");

    // Delete
    mgmt.endpoints()
        .delete(ws, &created.id)
        .await
        .expect("delete endpoint should succeed");

    // Verify deleted
    let err = mgmt
        .endpoints()
        .get(ws, &created.id)
        .await
        .expect_err("get deleted endpoint should fail");

    match &err {
        NahookError::Api(api_err) => {
            assert_eq!(api_err.status, 404, "expected 404 after delete");
        }
        other => panic!("expected NahookError::Api, got: {:?}", other),
    }
}

// ── Applications CRUD ──

#[tokio::test]
async fn test_applications_crud() {
    let cfg = match load_config() {
        Some(c) => c,
        None => return,
    };

    let mgmt = build_mgmt(&cfg.api_url, &cfg.mgmt_token);
    let ws = &cfg.workspace_id;
    let suffix = unique_suffix();
    let app_name = format!("Test App {}", suffix);

    // Create
    let created = mgmt
        .applications()
        .create(
            ws,
            CreateApplicationOptions {
                name: app_name.clone(),
                external_id: Some(format!("ext-{}", suffix)),
                metadata: None,
            },
        )
        .await
        .expect("create application should succeed");

    assert!(
        created.id.starts_with("app_"),
        "application id should start with 'app_', got: {}",
        created.id
    );
    assert_eq!(created.name, app_name);
    assert_eq!(
        created.external_id.as_deref(),
        Some(format!("ext-{}", suffix)).as_deref()
    );

    // List
    let list = mgmt
        .applications()
        .list(ws, None)
        .await
        .expect("list applications should succeed");

    assert!(
        list.data.iter().any(|a| a.id == created.id),
        "created application should appear in list"
    );

    // Get
    let fetched = mgmt
        .applications()
        .get(ws, &created.id)
        .await
        .expect("get application should succeed");

    assert_eq!(fetched.id, created.id);
    assert_eq!(fetched.name, app_name);

    // Update
    let updated_name = format!("Updated App {}", suffix);
    let updated = mgmt
        .applications()
        .update(
            ws,
            &created.id,
            UpdateApplicationOptions {
                name: Some(updated_name.clone()),
                metadata: None,
            },
        )
        .await
        .expect("update application should succeed");

    assert_eq!(updated.id, created.id);
    assert_eq!(updated.name, updated_name);

    // Delete
    mgmt.applications()
        .delete(ws, &created.id)
        .await
        .expect("delete application should succeed");

    // Verify deleted
    let err = mgmt
        .applications()
        .get(ws, &created.id)
        .await
        .expect_err("get deleted application should fail");

    match &err {
        NahookError::Api(api_err) => {
            assert_eq!(api_err.status, 404, "expected 404 after delete");
        }
        other => panic!("expected NahookError::Api, got: {:?}", other),
    }
}

// ── Subscriptions Lifecycle ──

#[tokio::test]
async fn test_subscriptions_lifecycle() {
    let cfg = match load_config() {
        Some(c) => c,
        None => return,
    };

    let mgmt = build_mgmt(&cfg.api_url, &cfg.mgmt_token);
    let ws = &cfg.workspace_id;
    let suffix = unique_suffix();

    // Create an endpoint for this test
    let endpoint = mgmt
        .endpoints()
        .create(
            ws,
            CreateEndpointOptions {
                url: format!("https://example.com/webhook/sub-test-{}", suffix),
                endpoint_type: None,
                description: Some("subscription lifecycle test endpoint".to_string()),
                metadata: None,
                config: None,
                auth_username: None,
                auth_password: None,
            },
        )
        .await
        .expect("create endpoint should succeed");

    // Create an event type for this test
    let event_type = mgmt
        .event_types()
        .create(
            ws,
            CreateEventTypeOptions {
                name: format!("test.sub.lifecycle.{}", suffix),
                description: Some("subscription lifecycle test event type".to_string()),
            },
        )
        .await
        .expect("create event type should succeed");

    // Subscribe endpoint to event type (plural array, returns { subscribed: N })
    let result: CreateSubscriptionResult = mgmt
        .subscriptions()
        .create(
            ws,
            &endpoint.id,
            CreateSubscriptionOptions {
                event_type_ids: vec![event_type.id.clone()],
            },
        )
        .await
        .expect("create subscription should succeed");

    assert_eq!(result.subscribed, 1, "should have subscribed 1 event type");

    // List subscriptions — items have { id, eventTypeId, eventTypeName, createdAt }
    let list = mgmt
        .subscriptions()
        .list(ws, &endpoint.id)
        .await
        .expect("list subscriptions should succeed");

    assert!(
        list.data
            .iter()
            .any(|s| s.event_type_id == event_type.id),
        "created subscription should appear in list"
    );

    // Verify eventTypeName is populated
    let matched = list.data.iter().find(|s| s.event_type_id == event_type.id);
    assert!(matched.is_some(), "subscription entry should exist");
    assert!(
        !matched.unwrap().event_type_name.is_empty(),
        "eventTypeName should be populated"
    );

    // Unsubscribe (delete by event type public_id, returns 204)
    mgmt.subscriptions()
        .delete(ws, &endpoint.id, &event_type.id)
        .await
        .expect("delete subscription should succeed");

    // Verify unsubscribed
    let list_after = mgmt
        .subscriptions()
        .list(ws, &endpoint.id)
        .await
        .expect("list subscriptions after delete should succeed");

    assert!(
        !list_after
            .data
            .iter()
            .any(|s| s.event_type_id == event_type.id),
        "deleted subscription should no longer appear in list"
    );

    // Cleanup: delete endpoint and event type
    mgmt.endpoints()
        .delete(ws, &endpoint.id)
        .await
        .expect("cleanup: delete endpoint should succeed");

    mgmt.event_types()
        .delete(ws, &event_type.id)
        .await
        .expect("cleanup: delete event type should succeed");
}

// ── Error tests ──

#[tokio::test]
async fn test_invalid_token_returns_401() {
    let cfg = match load_config() {
        Some(c) => c,
        None => return,
    };

    let mgmt = build_mgmt(&cfg.api_url, "nhm_invalid_token_that_does_not_exist");
    let err = mgmt
        .event_types()
        .list(&cfg.workspace_id)
        .await
        .expect_err("should fail with invalid management token");

    match &err {
        NahookError::Api(api_err) => {
            assert_eq!(api_err.status, 401, "expected 401 status");
            assert!(api_err.is_auth_error(), "should be an auth error");
        }
        other => panic!("expected NahookError::Api, got: {:?}", other),
    }
}
