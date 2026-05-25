//! # Nahook SDK
//!
//! Official Rust SDK for the [Nahook](https://nahook.com) webhook platform.
//!
//! This crate provides two main clients:
//!
//! - [`NahookClient`] - For sending webhooks via the ingestion API (requires `client` feature)
//! - [`NahookManagement`] - For managing resources via the management API (requires `management` feature)
//!
//! Both features are enabled by default.
//!
//! # Quick Start
//!
//! ```no_run
//! use nahook::NahookClient;
//! use nahook::types::SendOptions;
//!
//! # async fn example() -> Result<(), nahook::NahookError> {
//! let client = NahookClient::new("nhk_us_your_api_key")?;
//! let result = client.send("ep_abc123", SendOptions {
//!     payload: serde_json::json!({"event": "order.paid"}),
//!     idempotency_key: None,
//! }).await?;
//! println!("Delivery ID: {}", result.delivery_id);
//! # Ok(())
//! # }
//! ```

pub mod error;
pub(crate) mod http_client;
pub mod types;

#[cfg(feature = "client")]
mod client;

#[cfg(feature = "management")]
mod management;

#[cfg(feature = "management")]
pub mod resources;

// Re-exports
pub use error::NahookError;
pub use types::{SendOptions, TriggerOptions};

#[cfg(feature = "client")]
pub use client::{NahookClient, NahookClientBuilder};

#[cfg(feature = "management")]
pub use management::{NahookManagement, NahookManagementBuilder};
