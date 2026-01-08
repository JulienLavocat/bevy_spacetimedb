use bevy::prelude::Resource;
use spacetimedb_sdk::{ConnectionId, DbContext, Identity, Result};

#[derive(Resource)]
/// A connection to the SpacetimeDB server, as a Bevy resource.
/// This struct is a wrapper around a concrete-typed `DbContext` in your auto-generated.
pub struct StdbConnection<T: DbContext + 'static> {
    /// The underlying connection.
    conn: &'static T,
}

impl<T: DbContext> StdbConnection<T> {
    /// Create a new connection to the SpacetimeDB server.
    pub fn new(conn: &'static T) -> Self {
        Self { conn }
    }
}

impl<T: DbContext> StdbConnection<T> {
    /// Access to tables in the client cache, which stores a read-only replica of the remote database state.
    pub fn db(&self) -> &T::DbView {
        self.conn.db()
    }

    /// Access to reducers defined by the module.
    pub fn reducers(&self) -> &T::Reducers {
        self.conn.reducers()
    }

    pub fn procedures(&self) -> &T::Procedures {
        self.conn.procedures()
    }

    /// Get a builder-pattern constructor for subscribing to queries,
    /// causing matching rows to be replicated into the client cache.
    pub fn subscription_builder(&self) -> T::SubscriptionBuilder {
        self.conn.subscription_builder()
    }

    /// Get the [`Identity`] of this connection.
    pub fn identity(&self) -> Identity {
        self.conn.identity()
    }

    /// Get the [`Identity`] of this connection.
    pub fn try_identity(&self) -> Option<Identity> {
        self.conn.try_identity()
    }

    /// Returns `true` if the connection is active, i.e. has not yet disconnected.
    pub fn is_active(&self) -> bool {
        self.conn.is_active()
    }

    /// Close the connection.
    pub fn disconnect(&self) -> Result<()> {
        self.conn.disconnect()
    }

    /// Access to setters for per-reducer flags.
    pub fn set_reducer_flags(&self) -> &T::SetReducerFlags {
        self.conn.set_reducer_flags()
    }

    /// Get the connection ID.
    pub fn connection_id(&self) -> ConnectionId {
        self.conn.connection_id()
    }

    /// Access to the underlying connection, it's not recommended to use this method directly.
    pub fn conn(&self) -> &T {
        self.conn
    }
}
