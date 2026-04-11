use serde::{Deserialize, Serialize};
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
#[derive(Debug, Deserialize)]
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
#[derive(Debug, Deserialize)]
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
#[derive(Debug, Deserialize)]
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
#[derive(Debug, Deserialize)]
pub struct BatchItemError {
    pub code: String,
    pub message: String,
}

/// Result of a batch operation.
#[derive(Debug, Deserialize)]
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
#[derive(Debug, Serialize)]
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
}

/// Options for updating an endpoint.
#[derive(Debug, Serialize)]
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
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEventTypeOptions {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Options for updating an event type.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateEventTypeOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Options for creating an application.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateApplicationOptions {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

/// Options for updating an application.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateApplicationOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

/// Options for creating subscriptions (subscribe to one or more event types).
#[derive(Debug, Serialize)]
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
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEnvironmentOptions {
    pub name: String,
    pub slug: String,
}

/// Options for updating an environment.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateEnvironmentOptions {
    pub name: String,
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
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePortalSessionOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}
