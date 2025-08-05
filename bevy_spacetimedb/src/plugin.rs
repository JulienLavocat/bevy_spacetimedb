use crate::{
    AddEventChannelAppExtensions, StdbConnectedEvent, StdbConnection, StdbConnectionErrorEvent,
    StdbDisconnectedEvent,
};
use bevy::{
    app::{App, Plugin},
    platform::collections::HashMap,
};
use spacetimedb_sdk::{DbConnectionBuilder, DbContext};
use std::{
    any::{Any, TypeId},
    sync::{Mutex, mpsc::channel},
    thread::JoinHandle,
};

/// The plugin for connecting SpacetimeDB with your bevy application.
pub struct StdbPlugin<
    C: spacetimedb_sdk::__codegen::DbConnection<Module = M> + DbContext,
    M: spacetimedb_sdk::__codegen::SpacetimeModule<DbConnection = C>,
> {
    module_name: Option<String>,
    uri: Option<String>,
    token: Option<String>,
    run_fn: Option<fn(&C) -> JoinHandle<()>>,
    // Stores Senders for registered table events.
    pub(crate) event_senders: Mutex<HashMap<TypeId, Box<dyn Any + Send + Sync>>>,
    pub(crate) table_registers: Vec<
        Box<dyn Fn(&StdbPlugin<C, M>, &mut App, &'static <C as DbContext>::DbView) + Send + Sync>,
    >,
    pub(crate) reducer_registers:
        Vec<Box<dyn Fn(&mut App, &<C as DbContext>::Reducers) + Send + Sync>>,
}

impl<
    C: spacetimedb_sdk::__codegen::DbConnection<Module = M> + DbContext,
    M: spacetimedb_sdk::__codegen::SpacetimeModule<DbConnection = C>,
> Default for StdbPlugin<C, M>
{
    fn default() -> Self {
        Self {
            module_name: Default::default(),
            uri: Default::default(),
            token: Default::default(),
            run_fn: Option::default(),
            event_senders: Mutex::default(),
            table_registers: Vec::default(),
            reducer_registers: Vec::default(),
        }
    }
}

impl<
    C: spacetimedb_sdk::__codegen::DbConnection<Module = M> + DbContext + Send + Sync,
    M: spacetimedb_sdk::__codegen::SpacetimeModule<DbConnection = C>,
> StdbPlugin<C, M>
{
    /// The function that the connection will run with. The recommended function is `DbConnection::run_threaded`.
    ///
    /// Other function are not tested, they may not work.
    pub fn with_run_fn(mut self, run_fn: fn(&C) -> JoinHandle<()>) -> Self {
        self.run_fn = Some(run_fn);
        self
    }

    /// Set the name or identity of the remote module.
    pub fn with_module_name(mut self, name: impl Into<String>) -> Self {
        self.module_name = Some(name.into());
        self
    }

    /// Set the URI of the SpacetimeDB host which is running the remote module.
    ///
    /// The URI must have either no scheme or one of the schemes `http`, `https`, `ws` or `wss`.
    pub fn with_uri(mut self, uri: impl Into<String>) -> Self {
        self.uri = Some(uri.into());
        self
    }

    /// Supply a token with which to authenticate with the remote database.
    ///
    /// `token` should be an OpenID Connect compliant JSON Web Token.
    ///
    /// If this method is not invoked, or `None` is supplied,
    /// the SpacetimeDB host will generate a new anonymous `Identity`.
    ///
    /// If the passed token is invalid or rejected by the host,
    /// the connection will fail asynchrnonously.
    pub fn with_token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }
}

impl<
    C: spacetimedb_sdk::__codegen::DbConnection<Module = M> + DbContext + Sync,
    M: spacetimedb_sdk::__codegen::SpacetimeModule<DbConnection = C>,
> Plugin for StdbPlugin<C, M>
{
    fn build(&self, app: &mut App) {
        self.uri
            .clone()
            .expect("No uri set for StdbPlugin. Set it with the with_uri() function");
        self.module_name.clone().expect(
            "No module name set for StdbPlugin. Set it with the with_module_name() function",
        );

        let (send_connected, recv_connected) = channel::<StdbConnectedEvent>();
        let (send_disconnected, recv_disconnected) = channel::<StdbDisconnectedEvent>();
        let (send_connect_error, recv_connect_error) = channel::<StdbConnectionErrorEvent>();
        app.add_event_channel::<StdbConnectionErrorEvent>(recv_connect_error)
            .add_event_channel::<StdbConnectedEvent>(recv_connected)
            .add_event_channel::<StdbDisconnectedEvent>(recv_disconnected);

        // FIXME App should not crash if intial connection fails.
        let conn = DbConnectionBuilder::<M>::new()
            .with_module_name(self.module_name.clone().unwrap())
            .with_uri(self.uri.clone().unwrap())
            .with_token(self.token.clone())
            .on_connect_error(move |_ctx, err| {
                send_connect_error
                    .send(StdbConnectionErrorEvent { err })
                    .unwrap();
            })
            .on_disconnect(move |_ctx, err| {
                send_disconnected
                    .send(StdbDisconnectedEvent { err })
                    .unwrap();
            })
            .on_connect(move |_ctx, id, token| {
                send_connected
                    .send(StdbConnectedEvent {
                        identity: id,
                        access_token: token.to_string(),
                    })
                    .unwrap();
            })
            .build()
            .expect("Failed to build connection");

        // A 'static ref is needed for the connection the register tables and reducers
        // This is fine because only a small and fixed amount of memory will be leaked
        // Because conn has to live until the end of the program anyways, not using it would not make for any performance improvements.
        let conn = Box::<C>::leak(Box::new(conn));

        for table_register in self.table_registers.iter() {
            table_register(self, app, conn.db());
        }
        for reducer_register in self.reducer_registers.iter() {
            reducer_register(app, conn.reducers());
        }

        let run_fn = self.run_fn.expect("No run function specified!");
        run_fn(&conn);

        app.insert_resource(StdbConnection::new(conn));
    }
}
