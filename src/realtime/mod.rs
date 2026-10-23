//! Real-time synchronization support for AnyList.
//!
//! This module provides WebSocket-based real-time updates, allowing your
//! application to receive notifications when data changes on the server.
//!
//! # Overview
//!
//! AnyList uses WebSockets to notify clients of changes in real-time. When
//! another user (or another device) makes changes to shared lists, recipes, or
//! other data, the server sends a notification message indicating what type of
//! data changed.
//!
//! This library **does not** retain any state. Instead, it notifies your
//! application via a callback, and you decide what action to take (re-fetch
//! data, update a cache, notify the user, etc.).
//!
//! # Examples
//!
//! ## Simple
//!
//! ```no_run
//! use anylist_rs::{AnyListClient, SyncEvent};
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = Arc::new(
//!         AnyListClient::login("user@example.com", "password").await?
//!     );
//!
//!     // Connects immediately
//!     let mut sync = client.start_realtime_sync(|event| {
//!         match event {
//!             SyncEvent::ShoppingListsChanged => {
//!                 println!("Lists changed - consider re-fetching");
//!             }
//!             SyncEvent::RecipeDataChanged => {
//!                 println!("Recipes changed");
//!             }
//!             _ => {}
//!         }
//!     }).await?;
//!
//!     // Your application logic...
//!
//!     sync.disconnect().await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Advanced
//!
//! For cases where you need more control (parallel connections, testing, etc.):
//!
//! ```no_run
//! use anylist_rs::{AnyListClient, SyncEvent};
//! use anylist_rs::realtime::RealtimeSync;
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = Arc::new(
//!         AnyListClient::login("user@example.com", "password").await?
//!     );
//!
//!     // Create without connecting
//!     let mut sync = RealtimeSync::new(client, |event| {
//!         println!("Event: {:?}", event);
//!     });
//!
//!     // Connect when ready
//!     sync.connect().await?;
//!
//!     // Later...
//!     sync.disconnect().await?;
//!     Ok(())
//! }
//! ```
//!
//! # Connection Management
//!
//! The connection is managed automatically by one background task that owns
//! the WebSocket:
//! - **Heartbeats**: `--heartbeat--` and a WebSocket Ping are sent every
//!   5 seconds to keep the connection alive
//! - **Liveness**: if no frame of any kind arrives for 15 seconds, the
//!   connection is treated as dead and replaced
//! - **Auto-reconnect**: any interruption the consumer did not ask for
//!   (server close with any code, stream error or end, liveness timeout)
//!   moves to [`ConnectionState::Reconnecting`] and reconnects with
//!   exponential backoff; [`SyncEvent::Reconnected`] tells you when it is
//!   back, so you can refetch what changed during the gap
//! - **Token refresh**: if the access token expires (close code 4010, or an
//!   HTTP 401 on the handshake), tokens are refreshed before reconnecting.
//!   Ordinary outages do not refresh tokens.
//! - **Shutdown**: `disconnect()` returns within about
//!   [`RealtimeConfig::close_timeout`] in every state, and dropping the
//!   instance stops it without blocking
//!
//! All timings are configurable through [`RealtimeConfig`] and
//! [`RealtimeSync::with_config`].
//!
//! # Callbacks
//!
//! The callback runs on the connection's background task, one event at a
//! time and in order. A callback that blocks also delays heartbeats and
//! liveness checks, so spawn a task for real work.
//!
//! # Event Types
//!
//! See [`SyncEvent`] for the full list of events you can receive.
//!
//! # Thread Safety
//!
//! All types in this module are thread-safe and can be used across async tasks.

pub mod config;
pub mod events;
mod reconnect;
pub mod sync;

pub use config::RealtimeConfig;
pub use events::SyncEvent;
pub use sync::{ConnectionState, RealtimeSync};

/// Callback type for sync events
pub type SyncCallback = std::sync::Arc<dyn Fn(SyncEvent) + Send + Sync>;
