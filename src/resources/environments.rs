use crate::error::NahookError;
use crate::http_client::{encode_path_segment, HttpClient, Method};
use crate::types::*;

/// Resource for managing environments via the Management API.
pub struct EnvironmentsResource<'a> {
    pub(crate) http: &'a HttpClient,
}

impl<'a> EnvironmentsResource<'a> {
    /// List all environments in a workspace.
    pub async fn list(&self, workspace_id: &str) -> Result<ListResult<Environment>, NahookError> {
        let path = format!(
            "/management/v1/workspaces/{}/environments",
            encode_path_segment(workspace_id)
        );
        let data: Vec<Environment> = self.http.request_list(Method::Get, &path).await?;
        Ok(ListResult { data })
    }

    /// Create a new environment.
    pub async fn create(
        &self,
        workspace_id: &str,
        options: CreateEnvironmentOptions,
    ) -> Result<Environment, NahookError> {
        let path = format!(
            "/management/v1/workspaces/{}/environments",
            encode_path_segment(workspace_id)
        );
        self.http.request(Method::Post, &path, Some(&options)).await
    }

    /// Get an environment by ID.
    pub async fn get(
        &self,
        workspace_id: &str,
        id: &str,
    ) -> Result<Environment, NahookError> {
        let path = format!(
            "/management/v1/workspaces/{}/environments/{}",
            encode_path_segment(workspace_id),
            encode_path_segment(id)
        );
        self.http
            .request::<Environment>(Method::Get, &path, None::<&serde_json::Value>)
            .await
    }

    /// Update an environment.
    pub async fn update(
        &self,
        workspace_id: &str,
        id: &str,
        options: UpdateEnvironmentOptions,
    ) -> Result<Environment, NahookError> {
        let path = format!(
            "/management/v1/workspaces/{}/environments/{}",
            encode_path_segment(workspace_id),
            encode_path_segment(id)
        );
        self.http
            .request(Method::Patch, &path, Some(&options))
            .await
    }

    /// Delete an environment.
    pub async fn delete(
        &self,
        workspace_id: &str,
        id: &str,
    ) -> Result<(), NahookError> {
        let path = format!(
            "/management/v1/workspaces/{}/environments/{}",
            encode_path_segment(workspace_id),
            encode_path_segment(id)
        );
        self.http
            .request_empty(Method::Delete, &path, None::<&serde_json::Value>)
            .await
    }

    /// List event type visibility for an environment.
    pub async fn list_event_type_visibility(
        &self,
        workspace_id: &str,
        environment_id: &str,
    ) -> Result<ListResult<EventTypeVisibility>, NahookError> {
        let path = format!(
            "/management/v1/workspaces/{}/environments/{}/event-types",
            encode_path_segment(workspace_id),
            encode_path_segment(environment_id)
        );
        let data: Vec<EventTypeVisibility> = self.http.request_list(Method::Get, &path).await?;
        Ok(ListResult { data })
    }

    /// Set event type visibility in an environment.
    pub async fn set_event_type_visibility(
        &self,
        workspace_id: &str,
        environment_id: &str,
        event_type_id: &str,
        options: SetVisibilityOptions,
    ) -> Result<EventTypeVisibility, NahookError> {
        let path = format!(
            "/management/v1/workspaces/{}/environments/{}/event-types/{}/visibility",
            encode_path_segment(workspace_id),
            encode_path_segment(environment_id),
            encode_path_segment(event_type_id)
        );
        self.http
            .request(Method::Put, &path, Some(&options))
            .await
    }
}
