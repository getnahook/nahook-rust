use serde::Deserialize;

use crate::error::NahookError;
use crate::http_client::{encode_path_segment, HttpClient, Method};
use crate::types::*;

/// Read access to a workspace's webhook deliveries.
///
/// All methods are paginated or single-resource reads — there is no
/// create/update/delete on this resource. Deliveries are scoped to an
/// endpoint for [`Self::list`] because the deliveries table is indexed
/// by endpoint; single [`Self::get`] and [`Self::get_attempts`] accept a
/// delivery public id directly.
pub struct DeliveriesResource<'a> {
    pub(crate) http: &'a HttpClient,
}

/// Wire shape returned by the list endpoint. The resource maps this
/// into the generic [`PaginatedResult`] (renaming `deliveries` to `data`)
/// for cross-SDK consistency.
#[derive(Debug, Deserialize)]
struct ListDeliveriesResponse {
    deliveries: Vec<Delivery>,
    #[serde(rename = "nextCursor")]
    next_cursor: Option<String>,
}

impl<'a> DeliveriesResource<'a> {
    /// List deliveries for an endpoint, newest-first.
    ///
    /// `next_cursor` on the returned [`PaginatedResult`] is an opaque
    /// encrypted token — pass it back into [`ListDeliveriesOptions::cursor`]
    /// verbatim. Do not decode or modify it.
    pub async fn list(
        &self,
        workspace_id: &str,
        endpoint_id: &str,
        options: Option<ListDeliveriesOptions>,
    ) -> Result<PaginatedResult<Delivery>, NahookError> {
        let opts = options.unwrap_or_default();
        let mut path = format!(
            "/management/v1/workspaces/{}/endpoints/{}/deliveries",
            encode_path_segment(workspace_id),
            encode_path_segment(endpoint_id)
        );

        let mut query_parts: Vec<String> = Vec::new();
        if let Some(limit) = opts.limit {
            query_parts.push(format!("limit={}", limit));
        }
        if let Some(cursor) = opts.cursor.as_deref() {
            query_parts.push(format!("cursor={}", encode_path_segment(cursor)));
        }
        if let Some(status) = opts.status.as_deref() {
            query_parts.push(format!("status={}", encode_path_segment(status)));
        }
        if !query_parts.is_empty() {
            path.push('?');
            path.push_str(&query_parts.join("&"));
        }

        let raw: ListDeliveriesResponse = self
            .http
            .request(Method::Get, &path, None::<&serde_json::Value>)
            .await?;
        Ok(PaginatedResult {
            data: raw.deliveries,
            next_cursor: raw.next_cursor,
        })
    }

    /// Get a single delivery's metadata. When [`GetDeliveryOptions::include_payload`]
    /// is `true`, the response also carries a [`PayloadEnvelope`] in
    /// [`DeliveryWithPayload::payload`].
    pub async fn get(
        &self,
        workspace_id: &str,
        delivery_id: &str,
        options: Option<GetDeliveryOptions>,
    ) -> Result<DeliveryWithPayload, NahookError> {
        let opts = options.unwrap_or_default();
        let mut path = format!(
            "/management/v1/workspaces/{}/deliveries/{}",
            encode_path_segment(workspace_id),
            encode_path_segment(delivery_id)
        );
        if opts.include_payload {
            path.push_str("?include=payload");
        }
        self.http
            .request::<DeliveryWithPayload>(Method::Get, &path, None::<&serde_json::Value>)
            .await
    }

    /// List the chronological attempt history for a delivery.
    pub async fn get_attempts(
        &self,
        workspace_id: &str,
        delivery_id: &str,
    ) -> Result<Vec<DeliveryAttempt>, NahookError> {
        let path = format!(
            "/management/v1/workspaces/{}/deliveries/{}/attempts",
            encode_path_segment(workspace_id),
            encode_path_segment(delivery_id)
        );
        self.http.request_list(Method::Get, &path).await
    }
}
