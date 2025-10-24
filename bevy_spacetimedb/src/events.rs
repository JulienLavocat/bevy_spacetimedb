use bevy::prelude::Event;

/// An event that is triggered when a connection to SpacetimeDB is established.
#[derive(Event, Debug, Clone)]
pub struct StdbConnectedEvent {
    // Note: In WASM mode, we don't have direct access to Identity/tokens
    // These would need to be queried from the JS side if needed
}

/// An event that is triggered when a connection to SpacetimeDB is lost.
#[derive(Event, Debug, Clone)]
pub struct StdbDisconnectedEvent {
    /// The error message that caused the disconnection, if any.
    pub err: Option<String>,
}

/// An event that is triggered when a connection to SpacetimeDB encounters an error.
#[derive(Event, Debug, Clone)]
pub struct StdbConnectionErrorEvent {
    /// The error message that occurred.
    pub err: String,
}

/// An event that is triggered when a row is inserted into a table.
#[derive(Event, Debug, Clone)]
pub struct InsertEvent<T> {
    /// The row that was inserted.
    pub row: T,
}

/// An event that is triggered when a row is deleted from a table.
#[derive(Event, Debug, Clone)]
pub struct DeleteEvent<T> {
    /// The row that was deleted.
    pub row: T,
}

/// An event that is triggered when a row is updated in a table.
#[derive(Event, Debug, Clone)]
pub struct UpdateEvent<T> {
    /// The old row.
    pub old: T,
    /// The new row.
    pub new: T,
}

/// An event that is triggered when a row is inserted or updated in a table.
#[derive(Event, Debug, Clone)]
pub struct InsertUpdateEvent<T> {
    /// The previous value of the row if it was updated.
    pub old: Option<T>,
    /// The new value of the row or the inserted value.
    pub new: T,
}
