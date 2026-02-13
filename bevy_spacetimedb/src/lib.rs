// #![deny(missing_docs)]

//! A bevy plugin for SpacetimeDB.

#[cfg(all(feature = "client", feature = "server"))]
compile_error!("Features `client` and `server` are mutually exclusive. Enable only one.");

#[cfg(all(feature = "macros", feature = "server"))]
compile_error!("Feature `macros` cannot be used with `server`.");

mod aliases;
mod channel_receiver;
#[cfg(feature = "server")]
mod input;
mod messages;
mod plugin;
mod procedures;
mod reducers;
mod stdb_connection;
mod tables;

pub use aliases::*;
#[cfg(feature = "macros")]
pub use bevy_spacetimedb_macros::*;
#[cfg(feature = "server")]
pub use input::*;

pub use channel_receiver::AddMessageChannelAppExtensions;
pub use messages::*;
pub use plugin::{StdbPlugin, StdbPluginConfig, connect_with_token};
pub use reducers::RegisterableReducerMessage;
pub use stdb_connection::*;
pub use tables::{TableMessages, TableMessagesWithoutPrimaryKey};
