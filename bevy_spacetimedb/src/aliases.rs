use bevy::prelude::MessageReader;

use crate::{
    DeleteMessage, InsertMessage, InsertUpdateMessage, ReducerResultMessage, StdbConnectedMessage,
    StdbConnectionErrorMessage, StdbDisconnectedMessage, UpdateMessage,
};

/// A type alias for a Bevy event reader for InsertEvent<T>.
pub type ReadInsertMessage<'w, 's, T> = MessageReader<'w, 's, InsertMessage<T>>;

/// A type alias for a Bevy event reader for UpdateEvent<T>.
pub type ReadUpdateMessage<'w, 's, T> = MessageReader<'w, 's, UpdateMessage<T>>;

/// A type alias for a Bevy event reader for DeleteEvent<T>.
pub type ReadDeleteMessage<'w, 's, T> = MessageReader<'w, 's, DeleteMessage<T>>;

/// A type alias for a Bevy event reader for InsertUpdateEvent<T>.
pub type ReadInsertUpdateMessage<'w, 's, T> = MessageReader<'w, 's, InsertUpdateMessage<T>>;

/// A type alias for a Bevy event reader for ReducerResultEvent<T>.
pub type ReadReducerMessage<'w, 's, T> = MessageReader<'w, 's, ReducerResultMessage<T>>;

/// A type alias for a Bevy message reader for StdbConnectedMessage.
pub type ReadStdbConnectedMessage<'w, 's> = MessageReader<'w, 's, StdbConnectedMessage>;

/// A type alias for a Bevy message reader for StdbDisconnectedMessage.
pub type ReadStdbDisconnectedMessage<'w, 's> = MessageReader<'w, 's, StdbDisconnectedMessage>;

/// A type alias for a Bevy message reader for StdbConnectionErrorMessage.
pub type ReadStdbConnectionErrorMessage<'w, 's> = MessageReader<'w, 's, StdbConnectionErrorMessage>;
