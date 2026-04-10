use crate::error::NahookError;
use crate::http_client::{encode_path_segment, HttpClient, Method};
use crate::types::*;

/// Resource for creating portal sessions via the Management API.
pub struct PortalSessionsResource<'a> {
    pub(crate) http: &'a HttpClient,
}

impl<'a> PortalSessionsResource<'a> {
    /// Create a new portal session for an application.
    pub async fn create(
        &self,
        workspace_id: &str,
        app_id: &str,
        options: Option<CreatePortalSessionOptions>,
    ) -> Result<PortalSession, NahookError> {
        let path = format!(
            "/management/v1/workspaces/{}/applications/{}/portal",
            encode_path_segment(workspace_id),
            encode_path_segment(app_id)
        );
        let body = options.unwrap_or_default();
        self.http.request(Method::Post, &path, Some(&body)).await
    }
}
