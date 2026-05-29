use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;

// ── Client (ingestion) types ──

/// Options for sending a payload to a specific endpoint.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SendOptions {
    pub payload: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
}

/// Result of a successful send operation.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SendResult {
    pub delivery_id: String,
    pub idempotency_key: String,
    pub status: String,
}

/// Options for triggering a fan-out by event type.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TriggerOptions {
    pub payload: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

/// Result of a successful trigger operation.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TriggerResult {
    pub event_type_id: String,
    pub delivery_ids: Vec<String>,
    pub status: String,
}

/// A single item in a send batch request.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SendBatchItem {
    pub endpoint_id: String,
    pub payload: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
}

/// A single item in a trigger batch request.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TriggerBatchItem {
    pub event_type: String,
    pub payload: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

/// A single item in a batch result.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchResultItem {
    pub index: u32,
    pub delivery_id: Option<String>,
    pub idempotency_key: Option<String>,
    pub event_type_id: Option<String>,
    pub delivery_ids: Option<Vec<String>>,
    pub status: Option<String>,
    pub error: Option<BatchItemError>,
}

/// Error details for a single batch item.
#[derive(Debug, Deserialize, Serialize)]
pub struct BatchItemError {
    pub code: String,
    pub message: String,
}

/// Result of a batch operation.
#[derive(Debug, Deserialize, Serialize)]
pub struct BatchResult {
    pub items: Vec<BatchResultItem>,
}

// ── Management types ──

/// An endpoint resource.
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Endpoint {
    pub id: String,
    pub url: String,
    pub description: Option<String>,
    pub is_active: bool,
    #[serde(rename = "type")]
    pub endpoint_type: String,
    pub config: serde_json::Value,
    pub secret: Option<String>,
    pub metadata: Option<HashMap<String, String>>,
    pub created_at: String,
    pub updated_at: String,
}

/// An event type resource.
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EventType {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
}

/// An application resource.
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Application {
    pub id: String,
    pub external_id: Option<String>,
    pub name: String,
    pub metadata: HashMap<String, String>,
    pub created_at: String,
    pub updated_at: String,
}

/// A subscription resource.
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Subscription {
    pub id: String,
    pub event_type_id: String,
    pub event_type_name: String,
    pub created_at: String,
}

/// Result of a subscription create operation.
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateSubscriptionResult {
    pub subscribed: u32,
}

/// A portal session resource.
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PortalSession {
    pub url: String,
    pub code: String,
    pub expires_at: String,
}

/// Paginated list result.
#[derive(Debug, Deserialize)]
pub struct ListResult<T> {
    pub data: Vec<T>,
}

/// Options for list pagination.
#[derive(Debug, Default, Serialize)]
pub struct ListOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,
}

/// Options for creating an endpoint.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEndpointOptions {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
    pub endpoint_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_password: Option<String>,
    /// Optional. Public id (e.g. `env_abc123`) of the environment to scope this
    /// endpoint. If omitted, the workspace's default environment is used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment_id: Option<String>,
}

/// Options for updating an endpoint.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateEndpointOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
}

/// Options for creating an event type.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEventTypeOptions {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Options for updating an event type.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateEventTypeOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Options for creating an application.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateApplicationOptions {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

/// Options for updating an application.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateApplicationOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

/// Options for creating subscriptions (subscribe to one or more event types).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSubscriptionOptions {
    pub event_type_ids: Vec<String>,
}

/// An environment resource.
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Environment {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub is_default: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Options for creating an environment.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEnvironmentOptions {
    pub name: String,
    pub slug: String,
}

/// Options for updating an environment.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateEnvironmentOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// An event type visibility entry within an environment.
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EventTypeVisibility {
    pub event_type_id: String,
    pub event_type_name: String,
    pub published: bool,
}

/// Options for setting event type visibility in an environment.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetVisibilityOptions {
    pub published: bool,
}

/// Options for creating a portal session.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CreatePortalSessionOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "expiresInMinutes")]
    pub expires_in_minutes: Option<i32>,
}

// ── Deliveries ──

/// A webhook delivery's status and metadata. Read-only — deliveries are
/// created by the ingestion API, never directly through the management API.
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Delivery {
    pub id: String,
    pub idempotency_key: String,
    pub endpoint_id: String,
    pub status: String,
    pub total_attempts: u32,
    pub first_attempt_at: Option<String>,
    pub delivered_at: Option<String>,
    pub next_retry_at: Option<String>,
    pub has_payload: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// A single HTTP attempt against a delivery's endpoint. Returned in
/// chronological order (oldest first).
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeliveryAttempt {
    pub id: String,
    pub attempt_number: u32,
    /// Opaque outcome string emitted by the delivery worker (e.g.
    /// `"failed"`, `"success"`). Treat as a free-form string — the set is
    /// not enumerated and may evolve.
    pub status: String,
    pub response_status_code: Option<u16>,
    pub response_time_ms: Option<u64>,
    pub error_message: Option<String>,
    pub created_at: String,
}

/// Envelope wrapping the optional decrypted payload returned by
/// [`DeliveriesResource::get`] when `include_payload` is requested.
///
/// The envelope's variant describes whether the payload was retrievable;
/// non-`Available` variants are not errors — they reflect plan gating,
/// processing state, or absent payloads. The endpoint stays `200` for
/// all variants.
#[derive(Debug, Clone)]
pub enum PayloadEnvelope {
    /// Payload retrieved and decrypted.
    Available {
        data: serde_json::Value,
        content_type: String,
    },
    /// Workspace plan does not include payload storage.
    Forbidden,
    /// Delivery still in flight; payload write may be racing the read.
    Processing,
    /// Terminal delivery without a stored payload (older row or plan was
    /// lower at ingest time).
    NotFound,
    /// Transient infrastructure failure retrieving the payload.
    Error,
    /// Forward-compatibility variant: the server returned a `status` the SDK
    /// does not recognise. The original string is preserved so callers can
    /// log it or branch on it. A future SDK upgrade may add a stronger
    /// variant for any commonly-seen value.
    ///
    /// Note: any extra fields the server may send alongside `status` (e.g.
    /// `reason`, `details`) are dropped on deserialize and not re-emitted
    /// on serialize. If/when we need lossless preservation we'd extend this
    /// variant with a `raw: serde_json::Value` field; until then, consumers
    /// should treat Unknown as "log and move on".
    Unknown { status: String },
}

// Manual Deserialize impl — instead of `#[serde(tag = "status")]` which would
// hard-fail on a new server-side status, we read the JSON object, dispatch
// the known variants by name, and fall back to `Unknown { status }` for
// anything else. Keeps the SDK from breaking the next time the server adds
// an envelope status (e.g. `quarantined`).
impl<'de> Deserialize<'de> for PayloadEnvelope {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v = serde_json::Value::deserialize(deserializer)?;
        let obj = v
            .as_object()
            .ok_or_else(|| serde::de::Error::custom("envelope must be a JSON object"))?;
        let status = obj
            .get("status")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| serde::de::Error::missing_field("status"))?
            .to_string();

        match status.as_str() {
            "available" => {
                let data = obj
                    .get("data")
                    .cloned()
                    .ok_or_else(|| serde::de::Error::missing_field("data"))?;
                let content_type = obj
                    .get("contentType")
                    .and_then(serde_json::Value::as_str)
                    .ok_or_else(|| serde::de::Error::missing_field("contentType"))?
                    .to_string();
                Ok(PayloadEnvelope::Available { data, content_type })
            }
            "forbidden" => Ok(PayloadEnvelope::Forbidden),
            "processing" => Ok(PayloadEnvelope::Processing),
            "not_found" => Ok(PayloadEnvelope::NotFound),
            "error" => Ok(PayloadEnvelope::Error),
            _ => Ok(PayloadEnvelope::Unknown { status }),
        }
    }
}

// Manual Serialize impl — mirrors the previous tagged behaviour for the known
// variants. `Unknown { status }` emits the original status string verbatim
// so callers that capture and re-emit envelopes don't lose information.
impl Serialize for PayloadEnvelope {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeMap;
        match self {
            PayloadEnvelope::Available { data, content_type } => {
                let mut map = serializer.serialize_map(Some(3))?;
                map.serialize_entry("status", "available")?;
                map.serialize_entry("data", data)?;
                map.serialize_entry("contentType", content_type)?;
                map.end()
            }
            PayloadEnvelope::Forbidden => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("status", "forbidden")?;
                map.end()
            }
            PayloadEnvelope::Processing => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("status", "processing")?;
                map.end()
            }
            PayloadEnvelope::NotFound => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("status", "not_found")?;
                map.end()
            }
            PayloadEnvelope::Error => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("status", "error")?;
                map.end()
            }
            PayloadEnvelope::Unknown { status } => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("status", status)?;
                map.end()
            }
        }
    }
}

/// A [`Delivery`] optionally enriched with its decrypted payload envelope.
///
/// `payload` is `None` when the caller did not request `include_payload`,
/// and `Some(envelope)` otherwise — see [`PayloadEnvelope`] for the
/// possible states.
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeliveryWithPayload {
    pub id: String,
    pub idempotency_key: String,
    pub endpoint_id: String,
    pub status: String,
    pub total_attempts: u32,
    pub first_attempt_at: Option<String>,
    pub delivered_at: Option<String>,
    pub next_retry_at: Option<String>,
    pub has_payload: bool,
    pub created_at: String,
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<PayloadEnvelope>,
}

/// Cursor-paginated result. The cursor is an opaque token — pass
/// [`Self::next_cursor`] verbatim into the next call's
/// `cursor` option; do not decode or modify it. `next_cursor` is
/// `None` when there are no more pages.
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedResult<T> {
    pub data: Vec<T>,
    pub next_cursor: Option<String>,
}

/// Options for [`crate::resources::deliveries::DeliveriesResource::list`].
#[derive(Debug, Clone, Default)]
pub struct ListDeliveriesOptions {
    /// Server-side default is 50, max 100. Omit to use the default.
    pub limit: Option<u32>,
    /// Opaque cursor from a previous [`PaginatedResult::next_cursor`].
    pub cursor: Option<String>,
    /// Filter by status: `pending`, `delivering`, `delivered`,
    /// `scheduled_retry`, `failed`, or `dead_letter`.
    pub status: Option<String>,
}

/// Options for [`crate::resources::deliveries::DeliveriesResource::get`].
#[derive(Debug, Clone, Default)]
pub struct GetDeliveryOptions {
    /// When `true`, request the decrypted payload via the
    /// `?include=payload` query param. The response will populate
    /// [`DeliveryWithPayload::payload`] with a [`PayloadEnvelope`].
    pub include_payload: bool,
}
