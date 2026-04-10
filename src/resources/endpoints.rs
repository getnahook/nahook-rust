use crate::error::NahookError;
use crate::http_client::{encode_path_segment, HttpClient, Method};
use crate::types::*;

/// Resource for managing endpoints via the Management API.
pub struct EndpointsResource<'a> {
    pub(crate) http: &'a HttpClient,
}

impl<'a> EndpointsResource<'a> {
    /// List all endpoints in a workspace.
    pub async fn list(&self, workspace_id: &str) -> Result<ListResult<Endpoint>, NahookError> {
        let path = format!(
            "/management/v1/workspaces/{}/endpoints",
            encode_path_segment(workspace_id)
        );
        let data: Vec<Endpoint> = self.http.request_list(Method::Get, &path).await?;
        Ok(ListResult { data })
    }

    /// Create a new endpoint.
    pub async fn create(
        &self,
        workspace_id: &str,
        options: CreateEndpointOptions,
    ) -> Result<Endpoint, NahookError> {
        let path = format!(
            "/management/v1/workspaces/{}/endpoints",
            encode_path_segment(workspace_id)
        );
        self.http.request(Method::Post, &path, Some(&options)).await
    }

    /// Get an endpoint by ID.
    pub async fn get(
        &self,
        workspace_id: &str,
        id: &str,
    ) -> Result<Endpoint, NahookError> {
        let path = format!(
            "/management/v1/workspaces/{}/endpoints/{}",
            encode_path_segment(workspace_id),
            encode_path_segment(id)
        );
        self.http
            .request::<Endpoint>(Method::Get, &path, None::<&serde_json::Value>)
            .await
    }

    /// Update an endpoint.
    pub async fn update(
        &self,
        workspace_id: &str,
        id: &str,
        options: UpdateEndpointOptions,
    ) -> Result<Endpoint, NahookError> {
        let path = format!(
            "/management/v1/workspaces/{}/endpoints/{}",
            encode_path_segment(workspace_id),
            encode_path_segment(id)
        );
        self.http
            .request(Method::Patch, &path, Some(&options))
            .await
    }

    /// Delete an endpoint.
    pub async fn delete(
        &self,
        workspace_id: &str,
        id: &str,
    ) -> Result<(), NahookError> {
        let path = format!(
            "/management/v1/workspaces/{}/endpoints/{}",
            encode_path_segment(workspace_id),
            encode_path_segment(id)
        );
        self.http
            .request_empty(Method::Delete, &path, None::<&serde_json::Value>)
            .await
    }
}
