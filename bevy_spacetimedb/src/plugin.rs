use crate::{
    AddMessageChannelAppExtensions, StdbConnectedMessage, StdbConnection,
    StdbConnectionErrorMessage, StdbDisconnectedMessage,
};
use bevy::app::{App, Plugin};
use spacetimedb_sdk::{Compression, DbConnectionBuilder, DbContext};
use std::{
    any::{Any, TypeId},
    collections::HashMap,
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
    compression: Option<Compression>,
    light_mode: bool,

    // Stores Senders for registered table messages.
    pub(crate) message_senders: Mutex<HashMap<TypeId, Box<dyn Any + Send + Sync>>>,
    #[allow(clippy::type_complexity)]
    pub(crate) table_registers: Vec<
        Box<dyn Fn(&StdbPlugin<C, M>, &mut App, &'static <C as DbContext>::DbView) + Send + Sync>,
    >,
    #[allow(clippy::type_complexity)]
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
            uri: None,
            token: None,
            run_fn: None,
            compression: Some(Compression::default()),
            light_mode: false,

            message_senders: Mutex::default(),
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

    /// Sets the compression used when a certain threshold in the message size has been reached.
    ///
    /// The current threshold used by the host is 1KiB for the entire server message
    /// and for individual query updates.
    /// Note however that this threshold is not guaranteed and may change without notice.
    pub fn with_compression(mut self, compression: Compression) -> Self {
        self.compression = Some(compression);
        self
    }

    /// Sets whether the "light" mode is used.
    ///
    /// The light mode is meant for clients which are network-bandwidth constrained
    /// and results in non-callers receiving only light incremental updates.
    /// These updates will not include information about the reducer that caused them,
    /// but will contain updates to subscribed-to tables.
    /// As a consequence, when light-mode is enabled,
    /// non-callers will not receive reducer callbacks,
    /// but will receive callbacks for row insertion/deletion/updates.
    pub fn with_light_mode(mut self, light_mode: bool) -> Self {
        self.light_mode = light_mode;
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

        let (send_connected, recv_connected) = channel::<StdbConnectedMessage>();
        let (send_disconnected, recv_disconnected) = channel::<StdbDisconnectedMessage>();
        let (send_connect_error, recv_connect_error) = channel::<StdbConnectionErrorMessage>();
        app.add_message_channel::<StdbConnectionErrorMessage>(recv_connect_error)
            .add_message_channel::<StdbConnectedMessage>(recv_connected)
            .add_message_channel::<StdbDisconnectedMessage>(recv_disconnected);

        // FIXME App should not crash if intial connection fails.
        let conn = DbConnectionBuilder::<M>::new()
            .with_module_name(self.module_name.clone().unwrap())
            .with_uri(self.uri.clone().unwrap())
            .with_token(self.token.clone())
            .with_compression(self.compression.unwrap_or_default())
            .with_light_mode(self.light_mode)
            .on_connect_error(move |_ctx, err| {
                send_connect_error
                    .send(StdbConnectionErrorMessage { err })
                    .unwrap();
            })
            .on_disconnect(move |_ctx, err| {
                send_disconnected
                    .send(StdbDisconnectedMessage { err })
                    .unwrap();
            })
            .on_connect(move |_ctx, id, token| {
                send_connected
                    .send(StdbConnectedMessage {
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
        run_fn(conn);

        app.insert_resource(StdbConnection::new(conn));
    }
}
