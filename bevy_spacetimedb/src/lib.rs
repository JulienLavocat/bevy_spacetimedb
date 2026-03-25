// #![deny(missing_docs)]

//! A bevy plugin for SpacetimeDB.

mod aliases;
mod channel_receiver;
mod messages;
mod plugin;
mod reducers;
mod stdb_connection;
mod tables;
mod procedures;

pub use aliases::*;
#[cfg(feature = "macros")]
pub use bevy_spacetimedb_macros::*;

pub use channel_receiver::AddMessageChannelAppExtensions;
pub use messages::*;
pub use plugin::{StdbPlugin, StdbPluginConfig, ConnectionRunner, connect_with_token};
pub use stdb_connection::*;
pub use tables::{TableMessages, TableMessagesWithoutPrimaryKey};

/// Implements [`ConnectionRunner`] for a generated `DbConnection` type.
///
/// This handles the platform-specific event loop dispatch so your code
/// doesn't need any `#[cfg]` attributes:
///
/// ```ignore
/// bevy_spacetimedb::impl_connection_runner!(DbConnection);
/// ```
#[macro_export]
macro_rules! impl_connection_runner {
    ($conn_ty:ty) => {
        impl $crate::ConnectionRunner for $conn_ty {
            fn start_event_loop(&self) {
                #[cfg(not(target_arch = "wasm32"))]
                { self.run_threaded(); }
                #[cfg(target_arch = "wasm32")]
                { self.run_background_task(); }
            }
        }
    };
}
