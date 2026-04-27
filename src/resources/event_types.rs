use crate::error::NahookError;
use crate::http_client::{encode_path_segment, HttpClient, Method};
use crate::types::*;

/// Resource for managing event types via the Management API.
pub struct EventTypesResource<'a> {
    pub(crate) http: &'a HttpClient,
}

impl<'a> EventTypesResource<'a> {
    /// List all event types in a workspace.
    pub async fn list(&self, workspace_id: &str) -> Result<ListResult<EventType>, NahookError> {
        let path = format!(
            "/management/v1/workspaces/{}/event-types",
            encode_path_segment(workspace_id)
        );
        let data: Vec<EventType> = self.http.request_list(Method::Get, &path).await?;
        Ok(ListResult { data })
    }

    /// Create a new event type.
    pub async fn create(
        &self,
        workspace_id: &str,
        options: CreateEventTypeOptions,
    ) -> Result<EventType, NahookError> {
        let path = format!(
            "/management/v1/workspaces/{}/event-types",
            encode_path_segment(workspace_id)
        );
        self.http.request(Method::Post, &path, Some(&options)).await
    }

    /// Get an event type by ID.
    pub async fn get(&self, workspace_id: &str, id: &str) -> Result<EventType, NahookError> {
        let path = format!(
            "/management/v1/workspaces/{}/event-types/{}",
            encode_path_segment(workspace_id),
            encode_path_segment(id)
        );
        self.http
            .request::<EventType>(Method::Get, &path, None::<&serde_json::Value>)
            .await
    }

    /// Update an event type.
    pub async fn update(
        &self,
        workspace_id: &str,
        id: &str,
        options: UpdateEventTypeOptions,
    ) -> Result<EventType, NahookError> {
        let path = format!(
            "/management/v1/workspaces/{}/event-types/{}",
            encode_path_segment(workspace_id),
            encode_path_segment(id)
        );
        self.http
            .request(Method::Patch, &path, Some(&options))
            .await
    }

    /// Delete an event type.
    pub async fn delete(&self, workspace_id: &str, id: &str) -> Result<(), NahookError> {
        let path = format!(
            "/management/v1/workspaces/{}/event-types/{}",
            encode_path_segment(workspace_id),
            encode_path_segment(id)
        );
        self.http
            .request_empty(Method::Delete, &path, None::<&serde_json::Value>)
            .await
    }
}
