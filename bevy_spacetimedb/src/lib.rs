// #![deny(missing_docs)]

//! A bevy plugin for SpacetimeDB.

mod aliases;
mod channel_receiver;
mod events;
mod plugin;
mod reducers;
mod stdb_connection;
mod tables;

pub use aliases::*;
#[cfg(feature = "macros")]
pub use bevy_spacetimedb_macros::*;
pub use channel_receiver::AddEventChannelAppExtensions;
pub use events::*;
pub use plugin::*;
pub use reducers::RegisterableReducerEvent;
pub use stdb_connection::*;
pub use tables::TableEvents;
pub use tables::TableEventsWithoutPrimaryKey;