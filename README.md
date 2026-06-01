# Nahook Rust SDK

Official Rust SDK for the [Nahook](https://nahook.com) webhook platform.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
nahook = "0.1"
tokio = { version = "1", features = ["full"] }
serde_json = "1"
```

### Feature Flags

| Feature      | Default | Description                          |
|-------------|---------|--------------------------------------|
| `client`    | Yes     | Ingestion client (`NahookClient`)    |
| `management`| Yes     | Management client (`NahookManagement`) |

To use only the ingestion client:

```toml
[dependencies]
nahook = { version = "0.1", default-features = false, features = ["client"] }
```

## Quick Start

### Sending Webhooks

```rust
use nahook::NahookClient;
use nahook::types::SendOptions;

#[tokio::main]
async fn main() -> Result<(), nahook::NahookError> {
    let client = NahookClient::new("nhk_us_your_api_key")?;

    // Send to a specific endpoint
    let result = client.send("ep_abc123", SendOptions {
        payload: serde_json::json!({
            "event": "order.paid",
            "data": { "order_id": "ord_123", "amount": 4999 }
        }),
        idempotency_key: None, // Auto-generates a UUID
    }).await?;

    println!("Delivery ID: {}", result.delivery_id);
    Ok(())
}
```

### Fan-out by Event Type

```rust
use nahook::NahookClient;
use nahook::types::TriggerOptions;

#[tokio::main]
async fn main() -> Result<(), nahook::NahookError> {
    let client = NahookClient::new("nhk_us_your_api_key")?;

    let result = client.trigger("order.paid", TriggerOptions {
        payload: serde_json::json!({"order_id": "ord_123"}),
        metadata: None,
    }).await?;

    println!("Delivered to {} endpoints", result.delivery_ids.len());
    Ok(())
}
```

### Batch Operations

```rust
use nahook::NahookClient;
use nahook::types::{SendBatchItem, TriggerBatchItem};

#[tokio::main]
async fn main() -> Result<(), nahook::NahookError> {
    let client = NahookClient::new("nhk_us_your_api_key")?;

    // Batch send to specific endpoints
    let result = client.send_batch(vec![
        SendBatchItem {
            endpoint_id: "ep_abc".to_string(),
            payload: serde_json::json!({"event": "a"}),
            idempotency_key: None,
        },
        SendBatchItem {
            endpoint_id: "ep_def".to_string(),
            payload: serde_json::json!({"event": "b"}),
            idempotency_key: None,
        },
    ]).await?;

    println!("Batch sent: {} items", result.items.len());

    // Batch trigger by event types
    let result = client.trigger_batch(vec![
        TriggerBatchItem {
            event_type: "order.paid".to_string(),
            payload: serde_json::json!({"id": "1"}),
            metadata: None,
        },
    ]).await?;

    println!("Batch triggered: {} items", result.items.len());
    Ok(())
}
```

### Builder Configuration

```rust
use std::time::Duration;
use nahook::NahookClient;

let client = NahookClient::builder("nhk_us_your_api_key")
    .timeout(Duration::from_secs(15))
    .retries(3)
    .build()?;
```

### Configuration

The SDK automatically routes requests to the correct regional API based on your API key prefix (`nhk_us_...` -> US, `nhk_eu_...` -> EU, `nhk_ap_...` -> Asia Pacific). No configuration needed.

To override the base URL (for testing or local development):

```rust
let client = NahookClient::builder("nhk_us_your_api_key")
    .base_url("http://localhost:3001")
    .build()?;
```

For unit tests, mock the SDK client at the dependency injection boundary. For integration tests, override the base URL to point at a local server.

## Management API

The management client provides full CRUD for workspace resources.

```rust
use nahook::NahookManagement;
use nahook::types::*;

#[tokio::main]
async fn main() -> Result<(), nahook::NahookError> {
    let mgmt = NahookManagement::new("nhm_your_token")?;

    // List endpoints
    let endpoints = mgmt.endpoints().list("ws_abc123").await?;
    for ep in &endpoints.data {
        println!("{}: {} (active: {})", ep.id, ep.url, ep.is_active);
    }

    // Create an endpoint
    let new_ep = mgmt.endpoints().create("ws_abc123", CreateEndpointOptions {
        url: "https://example.com/webhook".to_string(),
        description: Some("Order notifications".to_string()),
        ..Default::default()
    }).await?;

    // Create an event type
    let evt = mgmt.event_types().create("ws_abc123", CreateEventTypeOptions {
        name: "order.paid".to_string(),
        description: Some("Fired when an order is paid".to_string()),
    }).await?;

    // Subscribe endpoint to event type
    let sub = mgmt.subscriptions().create("ws_abc123", &new_ep.id, CreateSubscriptionOptions {
        event_type_ids: vec![evt.id.clone()],
    }).await?;

    // Create application and portal session
    let app = mgmt.applications().create("ws_abc123", CreateApplicationOptions {
        name: "Acme Corp".to_string(),
        external_id: Some("acme-123".to_string()),
        metadata: None,
    }).await?;

    let portal = mgmt.portal_sessions().create("ws_abc123", &app.id, None).await?;
    println!("Portal URL: {}", portal.url);

    // Environments
    let envs = mgmt.environments().list("ws_abc123").await?;
    for env in &envs.data {
        println!("Env: {} ({}, default: {})", env.name, env.slug, env.is_default);
    }

    let new_env = mgmt.environments().create("ws_abc123", CreateEnvironmentOptions {
        name: "Staging".to_string(),
        slug: "staging".to_string(),
    }).await?;

    mgmt.environments().update("ws_abc123", &new_env.id, UpdateEnvironmentOptions {
        name: Some("Pre-production".to_string()),
    }).await?;

    mgmt.environments().delete("ws_abc123", &new_env.id).await?;

    // Event type visibility
    let vis = mgmt.environments().list_event_type_visibility("ws_abc123", &new_env.id).await?;
    mgmt.environments().set_event_type_visibility("ws_abc123", &new_env.id, &evt.id, SetVisibilityOptions {
        published: true,
    }).await?;

    Ok(())
}
```

### Management Builder

```rust
use std::time::Duration;
use nahook::NahookManagement;

let mgmt = NahookManagement::builder("nhm_your_token")
    .timeout(Duration::from_secs(60))
    .build()?;
```

### Deliveries

Read access to webhook delivery state, attempts, and (on Pro and above) the original decrypted payload.

```rust
use nahook::NahookManagement;
use nahook::types::{GetDeliveryOptions, ListDeliveriesOptions, PayloadEnvelope};

#[tokio::main]
async fn main() -> Result<(), nahook::NahookError> {
    let mgmt = NahookManagement::new("nhm_your_token")?;

    // Paginated list, newest-first. `next_cursor` is an opaque encrypted
    // token — pass it back verbatim, do not decode or modify it.
    let page = mgmt.deliveries().list("ws_abc", "ep_123", Some(ListDeliveriesOptions {
        limit: Some(50),
        cursor: None,
        status: None,
    })).await?;
    // page.data        -> Vec<Delivery>
    // page.next_cursor -> Option<String>

    if let Some(cursor) = page.next_cursor.clone() {
        let _next = mgmt.deliveries().list("ws_abc", "ep_123", Some(ListDeliveriesOptions {
            cursor: Some(cursor),
            ..Default::default()
        })).await?;
    }

    // Filter by status
    let _failed = mgmt.deliveries().list("ws_abc", "ep_123", Some(ListDeliveriesOptions {
        status: Some("failed".to_string()),
        ..Default::default()
    })).await?;

    // Get a single delivery's status + metadata (no payload envelope)
    let _delivery = mgmt.deliveries().get("ws_abc", "del_xyz", None).await?;

    // Get a delivery with its decrypted payload. The response wraps the body in
    // an envelope whose variant describes whether the payload is available,
    // gated by plan (`Forbidden`), still in flight (`Processing`), or absent.
    let with_payload = mgmt.deliveries().get("ws_abc", "del_xyz", Some(GetDeliveryOptions {
        include_payload: true,
    })).await?;
    match with_payload.payload {
        Some(PayloadEnvelope::Available { data, .. }) => {
            println!("Payload: {data}"); // the original webhook body
        }
        Some(PayloadEnvelope::Forbidden) => println!("Plan does not include payload storage"),
        Some(PayloadEnvelope::Processing) => println!("Payload write still in flight"),
        Some(PayloadEnvelope::NotFound) => println!("No stored payload for this delivery"),
        Some(PayloadEnvelope::Error) => println!("Transient error retrieving payload"),
        // Forward-compat: handles future server-side envelope statuses the SDK
        // doesn't model yet. Treat as "log and move on" — when this fires, an
        // SDK upgrade is likely available with a stronger variant.
        Some(PayloadEnvelope::Unknown { status }) => println!("Unrecognised envelope status: {status}"),
        None => {}
    }

    // List the attempt history for a delivery
    let _attempts = mgmt.deliveries().get_attempts("ws_abc", "del_xyz").await?;

    Ok(())
}
```

## Error Handling

```rust
use nahook::{NahookClient, NahookError};
use nahook::types::SendOptions;

async fn send_webhook(client: &NahookClient) {
    let result = client.send("ep_abc", SendOptions {
        payload: serde_json::json!({"test": true}),
        idempotency_key: None,
    }).await;

    match result {
        Ok(send_result) => {
            println!("Success: {}", send_result.delivery_id);
        }
        Err(NahookError::Api(api_err)) => {
            if api_err.is_rate_limited() {
                println!("Rate limited, retry after {:?}s", api_err.retry_after);
            } else if api_err.is_not_found() {
                println!("Endpoint not found");
            } else if api_err.is_auth_error() {
                println!("Authentication failed");
            } else if api_err.is_validation_error() {
                println!("Validation error: {}", api_err.message);
            } else {
                println!("API error {}: {}", api_err.status, api_err.message);
            }
        }
        Err(NahookError::Network(net_err)) => {
            println!("Network error: {}", net_err.cause);
        }
        Err(NahookError::Timeout(timeout_err)) => {
            println!("Timed out after {}ms", timeout_err.timeout_ms);
        }
    }
}
```

## Auth

- **Client API keys** must start with `nhk_`
- **Management tokens** must start with `nhm_`

Construction returns `Err` immediately if the prefix is invalid.

## Retry Behavior

The `NahookClient` supports automatic retries with exponential backoff and jitter:

- **Default retries**: 0 (no retries)
- **Base delay**: 500ms
- **Max delay**: 10s
- **Formula**: `min(10s, 500ms * 2^attempt) * rand()`
- **Retry-After**: Respected when present in 429 responses
- **Retryable**: 5xx, 429, connection errors, timeouts
- **Non-retryable**: 400, 401, 403, 404, 409, 413

The `NahookManagement` client does **not** retry.

## License

MIT
