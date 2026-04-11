use nahook::types::{SendBatchItem, SendOptions, TriggerBatchItem, TriggerOptions};
use nahook::{NahookClient, NahookError};
use serde_json::json;

/// Test environment configuration loaded from env vars.
struct TestConfig {
    api_url: String,
    api_key: String,
    disabled_api_key: String,
    endpoint_id: String,
    event_type: String,
}

/// Attempts to load the test configuration from environment variables.
/// Returns `None` if any required variable is missing, printing a skip message.
fn load_config() -> Option<TestConfig> {
    let api_url = match std::env::var("NAHOOK_TEST_API_URL") {
        Ok(v) => v,
        Err(_) => {
            println!("NAHOOK_TEST_API_URL not set — skipping integration tests");
            return None;
        }
    };
    let api_key = match std::env::var("NAHOOK_TEST_API_KEY") {
        Ok(v) => v,
        Err(_) => {
            println!("NAHOOK_TEST_API_KEY not set — skipping integration tests");
            return None;
        }
    };
    let disabled_api_key = match std::env::var("NAHOOK_TEST_DISABLED_API_KEY") {
        Ok(v) => v,
        Err(_) => {
            println!("NAHOOK_TEST_DISABLED_API_KEY not set — skipping integration tests");
            return None;
        }
    };
    let endpoint_id = match std::env::var("NAHOOK_TEST_ENDPOINT_ID") {
        Ok(v) => v,
        Err(_) => {
            println!("NAHOOK_TEST_ENDPOINT_ID not set — skipping integration tests");
            return None;
        }
    };
    let event_type = match std::env::var("NAHOOK_TEST_EVENT_TYPE") {
        Ok(v) => v,
        Err(_) => {
            println!("NAHOOK_TEST_EVENT_TYPE not set — skipping integration tests");
            return None;
        }
    };

    Some(TestConfig {
        api_url,
        api_key,
        disabled_api_key,
        endpoint_id,
        event_type,
    })
}

/// Build a NahookClient pointing at the test API.
fn build_client(api_url: &str, api_key: &str) -> NahookClient {
    NahookClient::builder(api_key)
        .base_url(api_url)
        .build()
        .expect("failed to build NahookClient")
}

// ── Happy-path tests ──

#[tokio::test]
async fn test_send_happy_path() {
    let cfg = match load_config() {
        Some(c) => c,
        None => return,
    };

    let client = build_client(&cfg.api_url, &cfg.api_key);
    let result = client
        .send(
            &cfg.endpoint_id,
            SendOptions {
                payload: json!({"test": "send_happy_path", "ts": now_millis()}),
                idempotency_key: None,
            },
        )
        .await
        .expect("send should succeed");

    assert_eq!(result.status, "accepted");
    assert!(
        result.delivery_id.starts_with("del_"),
        "delivery_id should start with 'del_', got: {}",
        result.delivery_id
    );
}

#[tokio::test]
async fn test_send_idempotency_dedup() {
    let cfg = match load_config() {
        Some(c) => c,
        None => return,
    };

    let client = build_client(&cfg.api_url, &cfg.api_key);
    let idem_key = format!("idem-dedup-{}", uuid::Uuid::new_v4());

    let result1 = client
        .send(
            &cfg.endpoint_id,
            SendOptions {
                payload: json!({"test": "idempotency_dedup"}),
                idempotency_key: Some(idem_key.clone()),
            },
        )
        .await
        .expect("first send should succeed");

    let result2 = client
        .send(
            &cfg.endpoint_id,
            SendOptions {
                payload: json!({"test": "idempotency_dedup"}),
                idempotency_key: Some(idem_key.clone()),
            },
        )
        .await
        .expect("second send should succeed");

    assert_eq!(
        result1.delivery_id, result2.delivery_id,
        "same idempotency key should return same delivery_id"
    );
}

#[tokio::test]
async fn test_send_separate_keys() {
    let cfg = match load_config() {
        Some(c) => c,
        None => return,
    };

    let client = build_client(&cfg.api_url, &cfg.api_key);

    let result1 = client
        .send(
            &cfg.endpoint_id,
            SendOptions {
                payload: json!({"test": "separate_keys_1"}),
                idempotency_key: Some(format!("sep-key-a-{}", uuid::Uuid::new_v4())),
            },
        )
        .await
        .expect("first send should succeed");

    let result2 = client
        .send(
            &cfg.endpoint_id,
            SendOptions {
                payload: json!({"test": "separate_keys_2"}),
                idempotency_key: Some(format!("sep-key-b-{}", uuid::Uuid::new_v4())),
            },
        )
        .await
        .expect("second send should succeed");

    assert_ne!(
        result1.delivery_id, result2.delivery_id,
        "different idempotency keys should produce different delivery_ids"
    );
}

#[tokio::test]
async fn test_trigger_fan_out() {
    let cfg = match load_config() {
        Some(c) => c,
        None => return,
    };

    let client = build_client(&cfg.api_url, &cfg.api_key);
    let result = client
        .trigger(
            &cfg.event_type,
            TriggerOptions {
                payload: json!({"test": "trigger_fan_out", "ts": now_millis()}),
                metadata: None,
            },
        )
        .await
        .expect("trigger should succeed");

    assert_eq!(result.status, "accepted");
    assert!(
        result.event_type_id.starts_with("evt_"),
        "event_type_id should start with 'evt_', got: {}",
        result.event_type_id
    );
    assert!(
        !result.delivery_ids.is_empty(),
        "trigger fan-out should produce at least 1 delivery_id"
    );
}

#[tokio::test]
async fn test_trigger_unsubscribed() {
    let cfg = match load_config() {
        Some(c) => c,
        None => return,
    };

    let client = build_client(&cfg.api_url, &cfg.api_key);
    let result = client
        .trigger(
            "event.type.nobody.subscribed.to",
            TriggerOptions {
                payload: json!({"test": "trigger_unsubscribed"}),
                metadata: None,
            },
        )
        .await
        .expect("trigger should succeed even with no subscribers");

    assert!(
        result.delivery_ids.is_empty(),
        "unsubscribed event type should produce empty delivery_ids"
    );
}

#[tokio::test]
async fn test_send_batch() {
    let cfg = match load_config() {
        Some(c) => c,
        None => return,
    };

    let client = build_client(&cfg.api_url, &cfg.api_key);
    let result = client
        .send_batch(vec![
            SendBatchItem {
                endpoint_id: cfg.endpoint_id.clone(),
                payload: json!({"test": "batch_item_1"}),
                idempotency_key: Some(format!("batch-send-1-{}", uuid::Uuid::new_v4())),
            },
            SendBatchItem {
                endpoint_id: cfg.endpoint_id.clone(),
                payload: json!({"test": "batch_item_2"}),
                idempotency_key: Some(format!("batch-send-2-{}", uuid::Uuid::new_v4())),
            },
        ])
        .await
        .expect("send_batch should succeed");

    assert_eq!(result.items.len(), 2, "batch should return 2 items");
    for item in &result.items {
        assert_eq!(
            item.status.as_deref(),
            Some("accepted"),
            "batch item {} should be accepted",
            item.index
        );
        assert!(
            item.delivery_id
                .as_ref()
                .map(|id| id.starts_with("del_"))
                .unwrap_or(false),
            "batch item {} delivery_id should start with 'del_'",
            item.index
        );
    }
}

#[tokio::test]
async fn test_trigger_batch() {
    let cfg = match load_config() {
        Some(c) => c,
        None => return,
    };

    let client = build_client(&cfg.api_url, &cfg.api_key);
    let result = client
        .trigger_batch(vec![
            TriggerBatchItem {
                event_type: cfg.event_type.clone(),
                payload: json!({"test": "trigger_batch_1"}),
                metadata: None,
            },
            TriggerBatchItem {
                event_type: cfg.event_type.clone(),
                payload: json!({"test": "trigger_batch_2"}),
                metadata: None,
            },
        ])
        .await
        .expect("trigger_batch should succeed");

    assert_eq!(result.items.len(), 2, "batch should return 2 items");
    for item in &result.items {
        assert_eq!(
            item.status.as_deref(),
            Some("accepted"),
            "batch item {} should be accepted",
            item.index
        );
    }
}

// ── Error tests ──

#[tokio::test]
async fn test_error_401_invalid_key() {
    let cfg = match load_config() {
        Some(c) => c,
        None => return,
    };

    let client = build_client(&cfg.api_url, "nhk_us_invalid_key_that_does_not_exist");
    let err = client
        .send(
            &cfg.endpoint_id,
            SendOptions {
                payload: json!({"test": "error_401"}),
                idempotency_key: None,
            },
        )
        .await
        .expect_err("should fail with invalid API key");

    match &err {
        NahookError::Api(api_err) => {
            assert_eq!(api_err.status, 401, "expected 401 status");
            assert!(api_err.is_auth_error(), "should be an auth error");
        }
        other => panic!("expected NahookError::Api, got: {:?}", other),
    }
}

#[tokio::test]
async fn test_error_403_disabled_key() {
    let cfg = match load_config() {
        Some(c) => c,
        None => return,
    };

    let client = build_client(&cfg.api_url, &cfg.disabled_api_key);
    let err = client
        .send(
            &cfg.endpoint_id,
            SendOptions {
                payload: json!({"test": "error_403"}),
                idempotency_key: None,
            },
        )
        .await
        .expect_err("should fail with disabled API key");

    match &err {
        NahookError::Api(api_err) => {
            assert_eq!(api_err.status, 403, "expected 403 status");
            assert_eq!(api_err.code, "token_disabled", "expected token_disabled code");
            assert!(api_err.is_auth_error(), "disabled key should be an auth error");
        }
        other => panic!("expected NahookError::Api, got: {:?}", other),
    }
}

#[tokio::test]
async fn test_error_404_missing_endpoint() {
    let cfg = match load_config() {
        Some(c) => c,
        None => return,
    };

    let client = build_client(&cfg.api_url, &cfg.api_key);
    let err = client
        .send(
            "ep_nonexistent_endpoint_xyz",
            SendOptions {
                payload: json!({"test": "error_404"}),
                idempotency_key: None,
            },
        )
        .await
        .expect_err("should fail with non-existent endpoint");

    match &err {
        NahookError::Api(api_err) => {
            assert_eq!(api_err.status, 404, "expected 404 status");
            assert!(api_err.is_not_found(), "should be a not-found error");
        }
        other => panic!("expected NahookError::Api, got: {:?}", other),
    }
}

#[tokio::test]
async fn test_error_400_invalid_event_type() {
    let cfg = match load_config() {
        Some(c) => c,
        None => return,
    };

    let client = build_client(&cfg.api_url, &cfg.api_key);
    let err = client
        .trigger(
            "!!!invalid event type!!!",
            TriggerOptions {
                payload: json!({"test": "error_400"}),
                metadata: None,
            },
        )
        .await
        .expect_err("should fail with invalid event type");

    match &err {
        NahookError::Api(api_err) => {
            assert_eq!(api_err.status, 400, "expected 400 status");
            assert!(
                api_err.is_validation_error(),
                "should be a validation error"
            );
        }
        other => panic!("expected NahookError::Api, got: {:?}", other),
    }
}

// ── Helpers ──

/// Simple timestamp string for unique payloads.
fn now_millis() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .to_string()
}
