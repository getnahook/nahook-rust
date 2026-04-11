use std::time::Duration;

use rand::Rng;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, CONTENT_TYPE, USER_AGENT};
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::error::{ApiError, NahookError, NetworkError, TimeoutError};

const SDK_VERSION: &str = "0.1.0";
const BASE_DELAY_MS: u64 = 500;
const MAX_DELAY_MS: u64 = 10_000;

/// HTTP method for requests.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Method {
    Get,
    Post,
    Patch,
    Delete,
}

/// Configuration for the underlying HTTP client.
#[derive(Debug, Clone)]
pub(crate) struct HttpClientConfig {
    pub token: String,
    pub base_url: String,
    pub timeout_ms: u64,
    pub retries: u32,
}

/// Internal HTTP client handling requests, retries, and error parsing.
#[derive(Debug, Clone)]
pub(crate) struct HttpClient {
    client: reqwest::Client,
    config: HttpClientConfig,
}

impl HttpClient {
    pub fn new(config: HttpClientConfig) -> Result<Self, NahookError> {
        let mut default_headers = HeaderMap::new();
        default_headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", config.token))
                .map_err(|e| NahookError::Api(ApiError {
                    status: 0,
                    code: "invalid_header".to_string(),
                    message: format!("Invalid authorization header: {e}"),
                    retry_after: None,
                }))?,
        );
        default_headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        default_headers.insert(
            USER_AGENT,
            HeaderValue::from_str(&format!("nahook-rust/{SDK_VERSION}")).unwrap(),
        );

        let client = reqwest::Client::builder()
            .default_headers(default_headers)
            .timeout(Duration::from_millis(config.timeout_ms))
            .build()
            .map_err(|e| NahookError::Network(NetworkError { cause: e }))?;

        Ok(Self { client, config })
    }

    /// Execute a request and deserialize the JSON response.
    pub async fn request<T: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        body: Option<&impl Serialize>,
    ) -> Result<T, NahookError> {
        let response = self.execute_with_retry(method, path, body).await?;

        if response.status().as_u16() == 204 {
            // For 204 No Content, return deserialized "null" which works for Option<T> types.
            // If the caller expects a non-optional type, this will produce a clear serde error.
            return serde_json::from_str("null").map_err(|e| {
                NahookError::Api(ApiError {
                    status: 204,
                    code: "no_content".to_string(),
                    message: format!("Cannot deserialize 204 No Content response: {e}"),
                    retry_after: None,
                })
            });
        }

        response
            .json::<T>()
            .await
            .map_err(|e| NahookError::Network(NetworkError { cause: e }))
    }

    /// Execute a request expecting no response body (e.g., DELETE).
    pub async fn request_empty(
        &self,
        method: Method,
        path: &str,
        body: Option<&impl Serialize>,
    ) -> Result<(), NahookError> {
        self.execute_with_retry(method, path, body).await?;
        Ok(())
    }

    /// Execute a request, returning the raw array for list endpoints
    /// that return arrays instead of objects.
    pub async fn request_list<T: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
    ) -> Result<Vec<T>, NahookError> {
        let response = self.execute_with_retry::<serde_json::Value>(method, path, None).await?;
        response
            .json::<Vec<T>>()
            .await
            .map_err(|e| NahookError::Network(NetworkError { cause: e }))
    }

    async fn execute_with_retry<B: Serialize>(
        &self,
        method: Method,
        path: &str,
        body: Option<&B>,
    ) -> Result<reqwest::Response, NahookError> {
        let url = format!(
            "{}/{}",
            self.config.base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        );

        let mut last_error: Option<NahookError> = None;

        for attempt in 0..=self.config.retries {
            if attempt > 0 {
                let retry_after_ms = match &last_error {
                    Some(NahookError::Api(api_err)) => {
                        api_err.retry_after.map(|s| s * 1000)
                    }
                    _ => None,
                };
                let delay = calculate_delay(attempt - 1, retry_after_ms);
                tokio::time::sleep(Duration::from_millis(delay)).await;
            }

            let mut request = match method {
                Method::Get => self.client.get(&url),
                Method::Post => self.client.post(&url),
                Method::Patch => self.client.patch(&url),
                Method::Delete => self.client.delete(&url),
            };

            if let Some(b) = body {
                request = request
                    .header(CONTENT_TYPE, "application/json")
                    .json(b);
            }

            match request.send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        return Ok(response);
                    }

                    let error = parse_error(response).await;
                    if attempt < self.config.retries && is_retryable(&error) {
                        last_error = Some(error);
                        continue;
                    }
                    return Err(error);
                }
                Err(e) => {
                    let error = if e.is_timeout() {
                        NahookError::Timeout(TimeoutError {
                            timeout_ms: self.config.timeout_ms,
                        })
                    } else {
                        NahookError::Network(NetworkError { cause: e })
                    };

                    if attempt < self.config.retries && is_retryable(&error) {
                        last_error = Some(error);
                        continue;
                    }
                    return Err(error);
                }
            }
        }

        Err(last_error.unwrap())
    }
}

/// Parse an error response into a `NahookError`.
async fn parse_error(response: reqwest::Response) -> NahookError {
    let status = response.status().as_u16();
    let retry_after = response
        .headers()
        .get("retry-after")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u64>().ok());

    #[derive(serde::Deserialize)]
    struct ErrorBody {
        error: Option<ErrorDetail>,
    }

    #[derive(serde::Deserialize)]
    struct ErrorDetail {
        code: Option<String>,
        message: Option<String>,
    }

    let (code, message) = match response.json::<ErrorBody>().await {
        Ok(body) => {
            let detail = body.error.unwrap_or(ErrorDetail {
                code: None,
                message: None,
            });
            (
                detail.code.unwrap_or_else(|| "unknown".to_string()),
                detail.message.unwrap_or_else(|| "Unknown error".to_string()),
            )
        }
        Err(_) => ("unknown".to_string(), "Unknown error".to_string()),
    };

    NahookError::Api(ApiError {
        status,
        code,
        message,
        retry_after,
    })
}

/// Whether an error is retryable.
fn is_retryable(error: &NahookError) -> bool {
    match error {
        NahookError::Api(api_err) => api_err.is_retryable(),
        NahookError::Network(_) | NahookError::Timeout(_) => true,
    }
}

/// Calculate retry delay with exponential backoff and full jitter.
fn calculate_delay(attempt: u32, retry_after_ms: Option<u64>) -> u64 {
    if let Some(ms) = retry_after_ms {
        if ms > 0 {
            return ms;
        }
    }
    let exponential = std::cmp::min(MAX_DELAY_MS, BASE_DELAY_MS * 2u64.pow(attempt));
    let jitter: f64 = rand::thread_rng().gen();
    (exponential as f64 * jitter) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculate_delay_within_exponential_cap() {
        // attempt 0 → exponential cap = min(10_000, 500 * 2^0) = 500
        // jitter ∈ [0, 1) so result ∈ [0, 500)
        for _ in 0..50 {
            let delay = calculate_delay(0, None);
            assert!(delay < 500, "Expected delay < 500, got {delay}");
        }
    }

    #[test]
    fn calculate_delay_caps_at_max_delay() {
        // attempt 10 → 500 * 2^10 = 512_000, capped to MAX_DELAY_MS = 10_000
        // jitter ∈ [0, 1) so result ∈ [0, 10_000)
        for _ in 0..50 {
            let delay = calculate_delay(10, None);
            assert!(
                delay < MAX_DELAY_MS,
                "Expected delay < {MAX_DELAY_MS}, got {delay}"
            );
        }
    }

    #[test]
    fn calculate_delay_uses_retry_after() {
        let delay = calculate_delay(0, Some(3000));
        assert_eq!(delay, 3000, "Expected retry_after value to be used directly");
    }
}

/// Percent-encode a path segment (mirrors JavaScript's `encodeURIComponent`).
///
/// Encodes all characters except unreserved characters: A-Z a-z 0-9 - _ . ~ ! ' ( ) *
/// This matches the behavior of JavaScript's `encodeURIComponent`.
pub(crate) fn encode_path_segment(segment: &str) -> String {
    /// Characters NOT encoded by JavaScript's `encodeURIComponent`:
    /// A-Z a-z 0-9 - _ . ~ ! ' ( ) *
    const ENCODE_URI_COMPONENT_SET: &percent_encoding::AsciiSet = &percent_encoding::NON_ALPHANUMERIC
        .remove(b'-')
        .remove(b'_')
        .remove(b'.')
        .remove(b'~')
        .remove(b'!')
        .remove(b'\'')
        .remove(b'(')
        .remove(b')')
        .remove(b'*');

    percent_encoding::utf8_percent_encode(segment, ENCODE_URI_COMPONENT_SET).to_string()
}
