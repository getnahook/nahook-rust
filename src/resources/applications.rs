use crate::error::NahookError;
use crate::http_client::{encode_path_segment, HttpClient, Method};
use crate::types::*;

/// Resource for managing applications via the Management API.
pub struct ApplicationsResource<'a> {
    pub(crate) http: &'a HttpClient,
}

impl<'a> ApplicationsResource<'a> {
    /// List all applications in a workspace.
    pub async fn list(
        &self,
        workspace_id: &str,
        options: Option<ListOptions>,
    ) -> Result<ListResult<Application>, NahookError> {
        let opts = options.unwrap_or_default();
        let mut path = format!(
            "/management/v1/workspaces/{}/applications",
            encode_path_segment(workspace_id)
        );

        let mut query_parts = Vec::new();
        if let Some(limit) = opts.limit {
            query_parts.push(format!("limit={}", limit));
        }
        if let Some(offset) = opts.offset {
            query_parts.push(format!("offset={}", offset));
        }
        if !query_parts.is_empty() {
            path.push('?');
            path.push_str(&query_parts.join("&"));
        }

        let data: Vec<Application> = self.http.request_list(Method::Get, &path).await?;
        Ok(ListResult { data })
    }

    /// Create a new application.
    pub async fn create(
        &self,
        workspace_id: &str,
        options: CreateApplicationOptions,
    ) -> Result<Application, NahookError> {
        let path = format!(
            "/management/v1/workspaces/{}/applications",
            encode_path_segment(workspace_id)
        );
        self.http.request(Method::Post, &path, Some(&options)).await
    }

    /// Get an application by ID.
    pub async fn get(&self, workspace_id: &str, id: &str) -> Result<Application, NahookError> {
        let path = format!(
            "/management/v1/workspaces/{}/applications/{}",
            encode_path_segment(workspace_id),
            encode_path_segment(id)
        );
        self.http
            .request::<Application>(Method::Get, &path, None::<&serde_json::Value>)
            .await
    }

    /// Update an application.
    pub async fn update(
        &self,
        workspace_id: &str,
        id: &str,
        options: UpdateApplicationOptions,
    ) -> Result<Application, NahookError> {
        let path = format!(
            "/management/v1/workspaces/{}/applications/{}",
            encode_path_segment(workspace_id),
            encode_path_segment(id)
        );
        self.http
            .request(Method::Patch, &path, Some(&options))
            .await
    }

    /// Delete an application.
    pub async fn delete(&self, workspace_id: &str, id: &str) -> Result<(), NahookError> {
        let path = format!(
            "/management/v1/workspaces/{}/applications/{}",
            encode_path_segment(workspace_id),
            encode_path_segment(id)
        );
        self.http
            .request_empty(Method::Delete, &path, None::<&serde_json::Value>)
            .await
    }

    /// List endpoints for an application.
    pub async fn list_endpoints(
        &self,
        workspace_id: &str,
        app_id: &str,
    ) -> Result<ListResult<Endpoint>, NahookError> {
        let path = format!(
            "/management/v1/workspaces/{}/applications/{}/endpoints",
            encode_path_segment(workspace_id),
            encode_path_segment(app_id)
        );
        let data: Vec<Endpoint> = self.http.request_list(Method::Get, &path).await?;
        Ok(ListResult { data })
    }

    /// Create an endpoint under an application.
    pub async fn create_endpoint(
        &self,
        workspace_id: &str,
        app_id: &str,
        options: CreateEndpointOptions,
    ) -> Result<Endpoint, NahookError> {
        let path = format!(
            "/management/v1/workspaces/{}/applications/{}/endpoints",
            encode_path_segment(workspace_id),
            encode_path_segment(app_id)
        );
        self.http.request(Method::Post, &path, Some(&options)).await
    }
}
