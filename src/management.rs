use std::time::Duration;

use crate::error::NahookError;
use crate::http_client::{HttpClient, HttpClientConfig};
use crate::resources::applications::ApplicationsResource;
use crate::resources::endpoints::EndpointsResource;
use crate::resources::event_types::EventTypesResource;
use crate::resources::portal_sessions::PortalSessionsResource;
use crate::resources::subscriptions::SubscriptionsResource;

/// Builder for constructing a [`NahookManagement`] client with custom configuration.
pub struct NahookManagementBuilder {
    token: String,
    base_url: Option<String>,
    timeout: Option<Duration>,
}

impl NahookManagementBuilder {
    /// Create a new builder with the given management token.
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
            base_url: None,
            timeout: None,
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

    /// Build the [`NahookManagement`] client.
    pub fn build(self) -> Result<NahookManagement, NahookError> {
        NahookManagement::new_internal(self.token, self.base_url, self.timeout)
    }
}

/// Client for the Nahook Management API.
///
/// Provides access to resources for managing endpoints, event types,
/// applications, subscriptions, and portal sessions.
///
/// # Example
/// ```no_run
/// use nahook::NahookManagement;
///
/// # async fn example() -> Result<(), nahook::NahookError> {
/// let mgmt = NahookManagement::new("nhm_your_token")?;
/// let endpoints = mgmt.endpoints().list("ws_abc123").await?;
/// for ep in &endpoints.data {
///     println!("{}: {}", ep.id, ep.url);
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct NahookManagement {
    http: HttpClient,
}

impl NahookManagement {
    /// Create a new management client with default settings.
    ///
    /// Returns an error if the token does not start with `nhm_`.
    pub fn new(token: impl Into<String>) -> Result<Self, NahookError> {
        Self::new_internal(token.into(), None, None)
    }

    /// Create a builder for more advanced configuration.
    pub fn builder(token: impl Into<String>) -> NahookManagementBuilder {
        NahookManagementBuilder::new(token)
    }

    fn new_internal(
        token: String,
        base_url: Option<String>,
        timeout: Option<Duration>,
    ) -> Result<Self, NahookError> {
        if !token.starts_with("nhm_") {
            return Err(NahookError::Api(crate::error::ApiError {
                status: 0,
                code: "invalid_token".to_string(),
                message: "Invalid management token: must start with 'nhm_'".to_string(),
                retry_after: None,
            }));
        }

        let config = HttpClientConfig {
            token,
            base_url: base_url.unwrap_or_else(|| "https://api.nahook.com".to_string()),
            timeout_ms: timeout.map(|d| d.as_millis() as u64).unwrap_or(30_000),
            retries: 0, // No retries for management
        };

        let http = HttpClient::new(config)?;
        Ok(Self { http })
    }

    /// Access the endpoints resource.
    pub fn endpoints(&self) -> EndpointsResource<'_> {
        EndpointsResource { http: &self.http }
    }

    /// Access the event types resource.
    pub fn event_types(&self) -> EventTypesResource<'_> {
        EventTypesResource { http: &self.http }
    }

    /// Access the applications resource.
    pub fn applications(&self) -> ApplicationsResource<'_> {
        ApplicationsResource { http: &self.http }
    }

    /// Access the subscriptions resource.
    pub fn subscriptions(&self) -> SubscriptionsResource<'_> {
        SubscriptionsResource { http: &self.http }
    }

    /// Access the portal sessions resource.
    pub fn portal_sessions(&self) -> PortalSessionsResource<'_> {
        PortalSessionsResource { http: &self.http }
    }
}
