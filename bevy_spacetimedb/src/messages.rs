use bevy::prelude::Message;
use spacetimedb_sdk::{Error, Identity};

/// A message that is emitted when a connection to SpacetimeDB is established.
#[derive(Message)]
pub struct StdbConnectedMessage {
    /// The `Identity`` of the successful connection.
    pub identity: Identity,
    /// The private access token which can be used to later re-authenticate as the same `Identity`.
    pub access_token: String,
}

/// A message that is emitted when a connection to SpacetimeDB is lost.
#[derive(Message)]
pub struct StdbDisconnectedMessage {
    /// The error that caused the disconnection, if any.
    pub err: Option<Error>,
}

/// A message that is emitted when a connection to SpacetimeDB encounters an error.
#[derive(Message)]
pub struct StdbConnectionErrorMessage {
    /// The error that occurred.
    pub err: Error,
}

/// A message that is emitted when a row is inserted into a table.
#[derive(Message)]
pub struct InsertMessage<T> {
    /// The row that was inserted.
    pub row: T,
}

/// A message that is emitted when a row is deleted from a table.
#[derive(Message)]
pub struct DeleteMessage<T> {
    /// The row that was deleted.
    pub row: T,
}

/// A message that is emitted when a row is updated in a table.
#[derive(Message)]
pub struct UpdateMessage<T> {
    /// The old row.
    pub old: T,
    /// The new row.
    pub new: T,
}

/// A message that is emitted when a row is inserted or updated in a table.
#[derive(Message)]
pub struct InsertUpdateMessage<T> {
    /// The previous value of the row if it was updated.
    pub old: Option<T>,
    /// The new value of the row or the inserted value.
    pub new: T,
}

/// A message that is emitted when a reducer is invoked.
#[derive(Message, Debug)]
pub struct ReducerResultMessage<T> {
    /// The result of the reducer invocation.
    pub result: T,
}

impl<T> ReducerResultMessage<T> {
    /// Creates a new reducer result message.
    pub fn new(result: T) -> Self {
        Self { result }
    }
}

#[derive(Message, Debug)]
pub struct ProcedureResultMessage<T> {
    /// The result of the reducer invocation.
    pub result: T,
}

impl<T> ProcedureResultMessage<T> {
    /// Creates a new reducer result message.
    pub fn new(result: T) -> Self {
        Self { result }
    }
}
