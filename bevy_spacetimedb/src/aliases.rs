use bevy::prelude::MessageReader;

use crate::{
    DeleteEvent, InsertEvent, InsertUpdateEvent, ReducerResultEvent, StdbConnectedEvent,
    StdbConnectionErrorEvent, StdbDisconnectedEvent, UpdateEvent,
};

/// A type alias for a Bevy event reader for InsertEvent<T>.
pub type ReadInsertEvent<'w, 's, T> = MessageReader<'w, 's, InsertEvent<T>>;

/// A type alias for a Bevy event reader for UpdateEvent<T>.
pub type ReadUpdateEvent<'w, 's, T> = MessageReader<'w, 's, UpdateEvent<T>>;

/// A type alias for a Bevy event reader for DeleteEvent<T>.
pub type ReadDeleteEvent<'w, 's, T> = MessageReader<'w, 's, DeleteEvent<T>>;

/// A type alias for a Bevy event reader for InsertUpdateEvent<T>.
pub type ReadInsertUpdateEvent<'w, 's, T> = MessageReader<'w, 's, InsertUpdateEvent<T>>;

/// A type alias for a Bevy event reader for ReducerResultEvent<T>.
pub type ReadReducerEvent<'w, 's, T> = MessageReader<'w, 's, ReducerResultEvent<T>>;

/// A type alias for a Bevy event reader for StdbConnectedEvent.
pub type ReadStdbConnectedEvent<'w, 's> = MessageReader<'w, 's, StdbConnectedEvent>;

/// A type alias for a Bevy event reader for StdbDisconnectedEvent.
pub type ReadStdbDisconnectedEvent<'w, 's> = MessageReader<'w, 's, StdbDisconnectedEvent>;

/// A type alias for a Bevy event reader for StdbConnectionErrorEvent.
pub type ReadStdbConnectionErrorEvent<'w, 's> = MessageReader<'w, 's, StdbConnectionErrorEvent>;
