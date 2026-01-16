use crate::{
    AddMessageChannelAppExtensions, StdbConnectedMessage, StdbConnection,
    StdbConnectionErrorMessage, StdbDisconnectedMessage,
};
use bevy::{
    app::{App, Plugin},
    platform::collections::HashMap,
    prelude::Resource,
};
use spacetimedb_sdk::{Compression, DbConnectionBuilder, DbContext};
use std::marker::PhantomData;
use std::{
    any::{Any, TypeId},
    sync::{
        Arc, Mutex,
        mpsc::{Sender, channel},
    },
    thread::JoinHandle,
};

/// Configuration for delayed SpacetimeDB connection
pub struct StdbPluginConfig<
    C: spacetimedb_sdk::__codegen::DbConnection<Module = M> + DbContext + Send + Sync,
    M: spacetimedb_sdk::__codegen::SpacetimeModule<DbConnection = C>,
> {
    pub module_name: String,
    pub uri: String,
    pub run_fn: fn(&C) -> JoinHandle<()>,
    pub compression: Compression,
    pub light_mode: bool,
    pub send_connected: Sender<StdbConnectedMessage>,
    pub send_disconnected: Sender<StdbDisconnectedMessage>,
    pub send_connect_error: Sender<StdbConnectionErrorMessage>,
    _phantom: PhantomData<(C, M)>,
}

// Manually implement Resource since we can't derive it with PhantomData
impl<
    C: spacetimedb_sdk::__codegen::DbConnection<Module = M> + DbContext + Send + Sync + 'static,
    M: spacetimedb_sdk::__codegen::SpacetimeModule<DbConnection = C> + 'static,
> Resource for StdbPluginConfig<C, M>
{
}

/// Stores plugin data (table/reducer registrations) for delayed connection
struct DelayedPluginData<
    C: spacetimedb_sdk::__codegen::DbConnection<Module = M> + DbContext + Send + Sync,
    M: spacetimedb_sdk::__codegen::SpacetimeModule<DbConnection = C>,
> {
    message_senders: Arc<Mutex<HashMap<TypeId, Box<dyn Any + Send + Sync>>>>,
    #[allow(clippy::type_complexity)]
    table_registers: Arc<
        Mutex<
            Vec<
                Box<
                    dyn Fn(&StdbPlugin<C, M>, &mut App, &'static <C as DbContext>::DbView)
                        + Send
                        + Sync,
                >,
            >,
        >,
    >,
    #[allow(clippy::type_complexity)]
    reducer_registers:
        Arc<Mutex<Vec<Box<dyn Fn(&mut App, &<C as DbContext>::Reducers) + Send + Sync>>>>,
}

/// Connect to SpacetimeDB with the given token (for delayed connection mode)
///
/// Call this from an exclusive system (system with `world: &mut World` parameter)
/// after OAuth completes to establish the connection with the token.
pub fn connect_with_token<
    C: spacetimedb_sdk::__codegen::DbConnection<Module = M> + DbContext + Send + Sync,
    M: spacetimedb_sdk::__codegen::SpacetimeModule<DbConnection = C>,
>(
    world: &mut bevy::prelude::World,
    token: Option<String>,
) {
    let config = world
        .remove_resource::<StdbPluginConfig<C, M>>()
        .expect("StdbPluginConfig not found - did you call with_delayed_connect()?");

    let plugin_data = world
        .remove_non_send_resource::<DelayedPluginData<C, M>>()
        .expect("DelayedPluginData not found");

    let send_connected = config.send_connected.clone();
    let send_disconnected = config.send_disconnected.clone();
    let send_connect_error = config.send_connect_error.clone();

    let conn = DbConnectionBuilder::<M>::new()
        .with_module_name(config.module_name)
        .with_uri(config.uri)
        .with_token(token)
        .with_compression(config.compression)
        .with_light_mode(config.light_mode)
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
        .expect("Failed to build delayed connection");

    let conn = Box::<C>::leak(Box::new(conn));

    // NOW register tables and reducers with the actual connection!
    // Create a temporary plugin with the stored message senders
    let temp_plugin = StdbPlugin::<C, M> {
        module_name: None,
        uri: None,
        token: None,
        run_fn: None,
        compression: None,
        light_mode: false,
        delayed_connect: false,
        message_senders: plugin_data.message_senders,
        table_registers: Arc::new(Mutex::new(Vec::new())),
        reducer_registers: Arc::new(Mutex::new(Vec::new())),
    };

    // Register tables with the real connection
    let table_regs = plugin_data.table_registers.lock().unwrap();
    for table_register in table_regs.iter() {
        table_register(
            &temp_plugin,
            unsafe { &mut *(world as *mut _ as *mut App) },
            conn.db(),
        );
    }
    drop(table_regs);

    // Register reducers
    let reducer_regs = plugin_data.reducer_registers.lock().unwrap();
    for reducer_register in reducer_regs.iter() {
        reducer_register(
            unsafe { &mut *(world as *mut _ as *mut App) },
            conn.reducers(),
        );
    }
    drop(reducer_regs);

    (config.run_fn)(conn);
    world.insert_resource(StdbConnection::new(conn));
}

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
    delayed_connect: bool, // NEW: Skip immediate connection

    // Stores Senders for registered table messages.
    pub(crate) message_senders: Arc<Mutex<HashMap<TypeId, Box<dyn Any + Send + Sync>>>>,
    #[allow(clippy::type_complexity)]
    pub(crate) table_registers: Arc<
        Mutex<
            Vec<
                Box<
                    dyn Fn(&StdbPlugin<C, M>, &mut App, &'static <C as DbContext>::DbView)
                        + Send
                        + Sync,
                >,
            >,
        >,
    >,
    #[allow(clippy::type_complexity)]
    pub(crate) reducer_registers:
        Arc<Mutex<Vec<Box<dyn Fn(&mut App, &<C as DbContext>::Reducers) + Send + Sync>>>>,
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
            delayed_connect: false, // NEW: Default to immediate connection

            message_senders: Arc::new(Mutex::new(HashMap::new())),
            table_registers: Arc::new(Mutex::new(Vec::new())),
            reducer_registers: Arc::new(Mutex::new(Vec::new())),
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

    /// Enable delayed connection mode. The connection will not be started
    /// during plugin build. You must manually call `connect_with_token()` later.
    ///
    /// This is useful for OAuth flows where the token is not available at app startup.
    pub fn with_delayed_connect(mut self, delayed: bool) -> Self {
        self.delayed_connect = delayed;
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

        // NEW: Check if we should delay the connection
        if self.delayed_connect {
            // Store configuration AND table/reducer registrations for later connection
            app.insert_resource(StdbPluginConfig::<C, M> {
                module_name: self.module_name.clone().unwrap(),
                uri: self.uri.clone().unwrap(),
                run_fn: self.run_fn.expect("No run function specified!"),
                compression: self.compression.unwrap_or_default(),
                light_mode: self.light_mode,
                send_connected,
                send_disconnected,
                send_connect_error,
                _phantom: PhantomData,
            });

            // Clone the Arc pointers to share the data with connect_with_token
            let plugin_for_later = DelayedPluginData::<C, M> {
                table_registers: Arc::clone(&self.table_registers),
                reducer_registers: Arc::clone(&self.reducer_registers),
                message_senders: Arc::clone(&self.message_senders),
            };
            app.insert_non_send_resource(plugin_for_later);

            return; // Skip connection - it will be created later via connect_with_token
        }

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

        {
            let table_regs = self.table_registers.lock().unwrap();
            for table_register in table_regs.iter() {
                table_register(self, app, conn.db());
            }
        }
        {
            let reducer_regs = self.reducer_registers.lock().unwrap();
            for reducer_register in reducer_regs.iter() {
                reducer_register(app, conn.reducers());
            }
        }

        let run_fn = self.run_fn.expect("No run function specified!");
        run_fn(conn);

        app.insert_resource(StdbConnection::new(conn));
    }
}
