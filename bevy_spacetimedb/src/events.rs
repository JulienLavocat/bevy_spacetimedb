use bevy::prelude::Event;
use spacetimedb_sdk::Error;

/// An event that is triggered when a connection to SpacetimeDB is established.
#[derive(Event)]
pub struct StdbConnectedEvent;

/// An event that is triggered when a connection to SpacetimeDB is lost.
#[derive(Event)]
pub struct StdbDisconnectedEvent {
    /// The error that caused the disconnection, if any.
    pub err: Option<Error>,
}

/// An event that is triggered when a connection to SpacetimeDB encounters an error.
#[derive(Event)]
pub struct StdbConnectionErrorEvent {
    /// The error that occurred.
    pub err: Error,
}

/// An event that is triggered when a row is inserted into a table.
#[derive(Event)]
pub struct InsertEvent<T> {
    /// The row that was inserted.
    pub row: T,
}

/// An event that is triggered when a row is deleted from a table.
#[derive(Event)]
pub struct DeleteEvent<T> {
    /// The row that was deleted.
    pub row: T,
}

/// An event that is triggered when a row is updated in a table.
#[derive(Event)]
pub struct UpdateEvent<T> {
    /// The old row.
    pub old: T,
    /// The new row.
    pub new: T,
}

/// An event that is triggered when a row is inserted or updated in a table.
#[derive(Event)]
pub struct InsertUpdateEvent<T> {
    /// The previous value of the row if it was updated.
    pub old: Option<T>,
    /// The new value of the row or the inserted value.
    pub new: T,
}

/// An event that is triggered when a reducer is invoked.
#[derive(Event, Debug)]
pub struct ReducerResultEvent<T> {
    /// The result of the reducer invocation.
    pub result: T,
}

impl<T> ReducerResultEvent<T> {
    /// Creates a new reducer result event.
    pub fn new(result: T) -> Self {
        Self { result }
    }
}
