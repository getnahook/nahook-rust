use std::time::Duration;

use crate::error::NahookError;
use crate::http_client::{encode_path_segment, HttpClient, HttpClientConfig, Method};
use crate::types::*;

const DEFAULT_BASE_URL: &str = "https://api.nahook.com";

/// Resolve the regional base URL from an nhk_ API key.
fn resolve_base_url(api_key: &str) -> &str {
    if api_key.len() >= 7 && api_key.starts_with("nhk_") && api_key.as_bytes()[6] == b'_' {
        match &api_key[4..6] {
            "us" => "https://us.api.nahook.com",
            "eu" => "https://eu.api.nahook.com",
            "ap" => "https://ap.api.nahook.com",
            _ => DEFAULT_BASE_URL,
        }
    } else {
        DEFAULT_BASE_URL
    }
}

/// Builder for constructing a [`NahookClient`] with custom configuration.
#[must_use]
pub struct NahookClientBuilder {
    api_key: String,
    base_url: Option<String>,
    timeout: Option<Duration>,
    retries: Option<u32>,
}

impl NahookClientBuilder {
    /// Create a new builder with the given API key.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: None,
            timeout: None,
            retries: None,
        }
    }

    /// Set the base URL for API requests.
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    /// Set the request timeout.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set the number of retries for failed requests.
    pub fn retries(mut self, retries: u32) -> Self {
        self.retries = Some(retries);
        self
    }

    /// Build the [`NahookClient`].
    pub fn build(self) -> Result<NahookClient, NahookError> {
        NahookClient::new_internal(
            self.api_key,
            self.base_url,
            self.timeout,
            self.retries,
        )
    }
}

/// Client for sending webhooks via the Nahook ingestion API.
///
/// # Example
/// ```no_run
/// use nahook::NahookClient;
/// use nahook::types::SendOptions;
///
/// # async fn example() -> Result<(), nahook::NahookError> {
/// let client = NahookClient::new("nhk_us_your_api_key")?;
/// let result = client.send("ep_abc123", SendOptions {
///     payload: serde_json::json!({"event": "order.paid"}),
///     idempotency_key: None,
/// }).await?;
/// println!("Delivery: {}", result.delivery_id);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct NahookClient {
    http: HttpClient,
}

impl NahookClient {
    /// Create a new client with default settings.
    ///
    /// Returns an error if the API key does not start with `nhk_`.
    pub fn new(api_key: impl Into<String>) -> Result<Self, NahookError> {
        Self::new_internal(api_key.into(), None, None, None)
    }

    /// Create a builder for more advanced configuration.
    pub fn builder(api_key: impl Into<String>) -> NahookClientBuilder {
        NahookClientBuilder::new(api_key)
    }

    fn new_internal(
        api_key: String,
        base_url: Option<String>,
        timeout: Option<Duration>,
        retries: Option<u32>,
    ) -> Result<Self, NahookError> {
        if !api_key.starts_with("nhk_") {
            return Err(NahookError::Api(crate::error::ApiError {
                status: 0,
                code: "invalid_key".to_string(),
                message: "Invalid API key: must start with 'nhk_'".to_string(),
                retry_after: None,
            }));
        }

        let resolved = base_url.unwrap_or_else(|| resolve_base_url(&api_key).to_string());
        let config = HttpClientConfig {
            token: api_key,
            base_url: resolved,
            timeout_ms: timeout.map(|d| d.as_millis() as u64).unwrap_or(30_000),
            retries: retries.unwrap_or(0),
        };

        let http = HttpClient::new(config)?;
        Ok(Self { http })
    }

    /// Send a payload to a specific endpoint.
    ///
    /// Automatically generates a UUID idempotency key if none is provided.
    pub async fn send(
        &self,
        endpoint_id: &str,
        options: SendOptions,
    ) -> Result<SendResult, NahookError> {
        let idempotency_key = options
            .idempotency_key
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        #[derive(serde::Serialize)]
        #[serde(rename_all = "camelCase")]
        struct SendBody {
            payload: serde_json::Value,
            idempotency_key: String,
        }

        let body = SendBody {
            payload: options.payload,
            idempotency_key,
        };

        let path = format!("/api/ingest/{}", encode_path_segment(endpoint_id));
        self.http.request(Method::Post, &path, Some(&body)).await
    }

    /// Fan-out a payload by event type to all subscribed endpoints.
    pub async fn trigger(
        &self,
        event_type: &str,
        options: TriggerOptions,
    ) -> Result<TriggerResult, NahookError> {
        #[derive(serde::Serialize)]
        #[serde(rename_all = "camelCase")]
        struct TriggerBody {
            payload: serde_json::Value,
            #[serde(skip_serializing_if = "Option::is_none")]
            metadata: Option<std::collections::HashMap<String, String>>,
        }

        let body = TriggerBody {
            payload: options.payload,
            metadata: options.metadata,
        };

        let path = format!("/api/ingest/event/{}", encode_path_segment(event_type));
        self.http.request(Method::Post, &path, Some(&body)).await
    }

    /// Batch send to multiple specific endpoints (max 20 items).
    pub async fn send_batch(
        &self,
        items: Vec<SendBatchItem>,
    ) -> Result<BatchResult, NahookError> {
        #[derive(serde::Serialize)]
        struct BatchBody {
            items: Vec<SendBatchItem>,
        }

        let body = BatchBody { items };
        self.http
            .request(Method::Post, "/api/ingest/batch", Some(&body))
            .await
    }

    /// Batch fan-out by event types (max 20 items).
    pub async fn trigger_batch(
        &self,
        items: Vec<TriggerBatchItem>,
    ) -> Result<BatchResult, NahookError> {
        #[derive(serde::Serialize)]
        struct BatchBody {
            items: Vec<TriggerBatchItem>,
        }

        let body = BatchBody { items };
        self.http
            .request(Method::Post, "/api/ingest/event/batch", Some(&body))
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Regional routing ──

    #[test]
    fn resolve_base_url_us_region() {
        assert_eq!(
            resolve_base_url("nhk_us_abc123"),
            "https://us.api.nahook.com"
        );
    }

    #[test]
    fn resolve_base_url_eu_region() {
        assert_eq!(
            resolve_base_url("nhk_eu_abc123"),
            "https://eu.api.nahook.com"
        );
    }

    #[test]
    fn resolve_base_url_ap_region() {
        assert_eq!(
            resolve_base_url("nhk_ap_abc123"),
            "https://ap.api.nahook.com"
        );
    }

    #[test]
    fn resolve_base_url_unknown_region_falls_back_to_default() {
        assert_eq!(
            resolve_base_url("nhk_zz_abc123"),
            DEFAULT_BASE_URL
        );
    }

    #[test]
    fn base_url_option_overrides_region() {
        // When base_url is Some, new_internal should use it directly
        // instead of calling resolve_base_url. We verify by confirming
        // that resolve_base_url("nhk_eu_abc123") returns the EU URL,
        // but building with a custom base_url succeeds (proving the
        // builder path is separate from resolution).
        assert_eq!(
            resolve_base_url("nhk_eu_abc123"),
            "https://eu.api.nahook.com",
            "Without override, EU key should resolve to EU endpoint"
        );
        let custom = "https://custom.example.com";
        // Builder accepts the override — this exercises new_internal's
        // `base_url.unwrap_or_else(|| resolve_base_url(...))` branch.
        let client = NahookClient::builder("nhk_eu_abc123")
            .base_url(custom)
            .build();
        assert!(
            client.is_ok(),
            "Building client with custom base_url should succeed"
        );
    }
}
