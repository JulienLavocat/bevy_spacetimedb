#![deny(missing_docs)]
#![no_std]

//! A bevy plugin for SpacetimeDB.

mod aliases;
mod channel_receiver;
mod events;
mod plugin;
mod stdb_connection;

pub use aliases::*;
pub use bevy_spacetimedb_macros::*;
pub use events::*;
pub use plugin::*;
pub use stdb_connection::*;
