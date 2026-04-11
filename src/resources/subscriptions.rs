use crate::error::NahookError;
use crate::http_client::{encode_path_segment, HttpClient, Method};
use crate::types::*;

/// Resource for managing subscriptions via the Management API.
pub struct SubscriptionsResource<'a> {
    pub(crate) http: &'a HttpClient,
}

impl<'a> SubscriptionsResource<'a> {
    /// List all subscriptions for an endpoint.
    pub async fn list(
        &self,
        workspace_id: &str,
        endpoint_id: &str,
    ) -> Result<ListResult<Subscription>, NahookError> {
        let path = format!(
            "/management/v1/workspaces/{}/endpoints/{}/subscriptions",
            encode_path_segment(workspace_id),
            encode_path_segment(endpoint_id)
        );
        let data: Vec<Subscription> = self.http.request_list(Method::Get, &path).await?;
        Ok(ListResult { data })
    }

    /// Subscribe an endpoint to one or more event types.
    pub async fn create(
        &self,
        workspace_id: &str,
        endpoint_id: &str,
        options: CreateSubscriptionOptions,
    ) -> Result<CreateSubscriptionResult, NahookError> {
        let path = format!(
            "/management/v1/workspaces/{}/endpoints/{}/subscriptions",
            encode_path_segment(workspace_id),
            encode_path_segment(endpoint_id)
        );
        self.http.request(Method::Post, &path, Some(&options)).await
    }

    /// Delete a subscription by event type ID.
    pub async fn delete(
        &self,
        workspace_id: &str,
        endpoint_id: &str,
        event_type_id: &str,
    ) -> Result<(), NahookError> {
        let path = format!(
            "/management/v1/workspaces/{}/endpoints/{}/subscriptions/{}",
            encode_path_segment(workspace_id),
            encode_path_segment(endpoint_id),
            encode_path_segment(event_type_id)
        );
        self.http
            .request_empty(Method::Delete, &path, None::<&serde_json::Value>)
            .await
    }
}
