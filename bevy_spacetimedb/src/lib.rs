// #![deny(missing_docs)]

//! A bevy plugin for SpacetimeDB.

mod aliases;
mod channel_receiver;
mod messages;
mod plugin;
mod reducers;
mod stdb_connection;
mod tables;

pub use aliases::*;
#[cfg(feature = "macros")]
pub use bevy_spacetimedb_macros::*;

// Primary message-based API
pub use channel_receiver::AddMessageChannelAppExtensions;
pub use messages::{
    DeleteMessage, InsertMessage, InsertUpdateMessage, ReducerResultMessage, StdbConnectedMessage,
    StdbConnectionErrorMessage, StdbDisconnectedMessage, UpdateMessage,
};
pub use plugin::*;
pub use reducers::RegisterableReducerMessage;
pub use stdb_connection::*;
pub use tables::TableMessages;
