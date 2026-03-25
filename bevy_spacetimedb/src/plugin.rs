use crate::{
    AddMessageChannelAppExtensions, StdbConnectedMessage, StdbConnection,
    StdbConnectionErrorMessage, StdbDisconnectedMessage,
};
use bevy::{
    app::{App, Plugin},
    platform::collections::HashMap,
    prelude::Resource,
};
use std::marker::PhantomData;
use spacetimedb_sdk::{Compression, DbConnectionBuilder, DbContext};
use std::{
    any::{Any, TypeId},
    sync::{Arc, Mutex, mpsc::{channel, Sender}},
};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;

/// Trait for starting the connection's event loop.
///
/// Use the [`impl_connection_runner!`] macro to implement this for your
/// generated `DbConnection` type. The macro handles platform differences
/// internally, so your code stays `#[cfg]`-free.
pub trait ConnectionRunner: Send + Sync + 'static {
    fn start_event_loop(&self);
}

/// Configuration for delayed SpacetimeDB connection
pub struct StdbPluginConfig<
    C: spacetimedb_sdk::__codegen::DbConnection<Module = M> + DbContext + Send + Sync,
    M: spacetimedb_sdk::__codegen::SpacetimeModule<DbConnection = C>,
> {
    pub database_name: String,
    pub uri: String,
    pub compression: Compression,
    pub send_connected: Sender<StdbConnectedMessage>,
    pub send_disconnected: Sender<StdbDisconnectedMessage>,
    pub send_connect_error: Sender<StdbConnectionErrorMessage>,
    _phantom: PhantomData<(C, M)>,
}

// Manually implement Resource since we can't derive it with PhantomData
impl<
    C: spacetimedb_sdk::__codegen::DbConnection<Module = M> + DbContext + Send + Sync + 'static,
    M: spacetimedb_sdk::__codegen::SpacetimeModule<DbConnection = C> + 'static,
> Resource for StdbPluginConfig<C, M> {}

/// Stores plugin data (table/reducer registrations) for delayed connection
struct DelayedPluginData<
    C: spacetimedb_sdk::__codegen::DbConnection<Module = M> + DbContext + Send + Sync,
    M: spacetimedb_sdk::__codegen::SpacetimeModule<DbConnection = C>,
> {
    message_senders: Arc<Mutex<HashMap<TypeId, Box<dyn Any + Send + Sync>>>>,
    #[allow(clippy::type_complexity)]
    table_registers: Arc<Mutex<Vec<
        Box<dyn Fn(&StdbPlugin<C, M>, &mut App, &'static <C as DbContext>::DbView) + Send + Sync>,
    >>>,
}

/// Connect to SpacetimeDB with the given token (for delayed connection mode)
/// 
/// Call this from an exclusive system (system with `world: &mut World` parameter)
/// after OAuth completes to establish the connection with the token.
pub fn connect_with_token<
    C: spacetimedb_sdk::__codegen::DbConnection<Module = M> + DbContext + ConnectionRunner,
    M: spacetimedb_sdk::__codegen::SpacetimeModule<DbConnection = C>,
>(
    world: &mut bevy::prelude::World,
    token: Option<String>,
) {
    let config = world.remove_resource::<StdbPluginConfig<C, M>>()
        .expect("StdbPluginConfig not found - did you call with_delayed_connect()?");
    
    let plugin_data = world.remove_non_send_resource::<DelayedPluginData<C, M>>()
        .expect("DelayedPluginData not found");

    #[cfg(target_arch = "wasm32")]
    {
        let world_ptr = world as *mut bevy::prelude::World;
        spawn_local(async move {
            let send_connected = config.send_connected.clone();
            let send_disconnected = config.send_disconnected.clone();
            let send_connect_error = config.send_connect_error.clone();

            let conn = DbConnectionBuilder::<M>::new()
                .with_database_name(config.database_name)
                .with_uri(config.uri)
                .with_token(token)
                .with_compression(config.compression)
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
                .await
                .expect("Failed to build delayed connection");

            let conn = Box::<C>::leak(Box::new(conn));

            // SAFETY: We're accessing world pointer from async context.
            // Safe because connect_with_token is called from an exclusive system.
            let world = unsafe { &mut *world_ptr };
            let temp_plugin = StdbPlugin::<C, M> {
                database_name: None,
                uri: None,
                token: None,
                compression: None,
                delayed_connect: false,
                message_senders: Arc::clone(&plugin_data.message_senders),
                table_registers: Arc::new(Mutex::new(Vec::new())),
                procedure_registers: Arc::new(Mutex::new(Vec::new())),
            };

            let table_regs = plugin_data.table_registers.lock().unwrap();
            for table_register in table_regs.iter() {
                table_register(&temp_plugin, unsafe { &mut *(world as *mut _ as *mut App) }, conn.db());
            }
            drop(table_regs);

            conn.start_event_loop();
            world.insert_resource(StdbConnection::new(conn));
        });
        return;
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let send_connected = config.send_connected.clone();
        let send_disconnected = config.send_disconnected.clone();
        let send_connect_error = config.send_connect_error.clone();

        let conn = DbConnectionBuilder::<M>::new()
            .with_database_name(config.database_name)
            .with_uri(config.uri)
            .with_token(token)
            .with_compression(config.compression)
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

        let temp_plugin = StdbPlugin::<C, M> {
            database_name: None,
            uri: None,
            token: None,
            compression: None,
            delayed_connect: false,
            message_senders: Arc::clone(&plugin_data.message_senders),
            table_registers: Arc::new(Mutex::new(Vec::new())),
            procedure_registers: Arc::new(Mutex::new(Vec::new())),
        };

        let table_regs = plugin_data.table_registers.lock().unwrap();
        for table_register in table_regs.iter() {
            table_register(&temp_plugin, unsafe { &mut *(world as *mut _ as *mut App) }, conn.db());
        }
        drop(table_regs);

        conn.start_event_loop();
        world.insert_resource(StdbConnection::new(conn));
    }
}

/// The plugin for connecting SpacetimeDB with your bevy application.
pub struct StdbPlugin<
    C: spacetimedb_sdk::__codegen::DbConnection<Module = M> + DbContext,
    M: spacetimedb_sdk::__codegen::SpacetimeModule<DbConnection = C>,
> {
    database_name: Option<String>,
    uri: Option<String>,
    token: Option<String>,
    compression: Option<Compression>,
    delayed_connect: bool,

    // Stores Senders for registered table messages.
    pub(crate) message_senders: Arc<Mutex<HashMap<TypeId, Box<dyn Any + Send + Sync>>>>,
    #[allow(clippy::type_complexity)]
    pub(crate) table_registers: Arc<Mutex<Vec<
        Box<dyn Fn(&StdbPlugin<C, M>, &mut App, &'static <C as DbContext>::DbView) + Send + Sync>,
    >>>,
    #[allow(clippy::type_complexity)]
    pub(crate) procedure_registers:
        Arc<Mutex<Vec<Box<dyn Fn(&mut App, &<C as DbContext>::Procedures) + Send + Sync>>>>,
}

impl<
    C: spacetimedb_sdk::__codegen::DbConnection<Module = M> + DbContext,
    M: spacetimedb_sdk::__codegen::SpacetimeModule<DbConnection = C>,
> Default for StdbPlugin<C, M>
{
    fn default() -> Self {
        Self {
            database_name: Default::default(),
            uri: None,
            token: None,
            compression: Some(Compression::default()),
            delayed_connect: false,

            message_senders: Arc::new(Mutex::default()),
            table_registers: Arc::new(Mutex::new(Vec::default())),
            procedure_registers: Arc::new(Mutex::new(Vec::default())),
        }
    }
}

impl<
    C: spacetimedb_sdk::__codegen::DbConnection<Module = M> + DbContext + Send + Sync,
    M: spacetimedb_sdk::__codegen::SpacetimeModule<DbConnection = C>,
> StdbPlugin<C, M>
{
    /// Set the name of the remote database.
    pub fn with_database_name(mut self, name: impl Into<String>) -> Self {
        self.database_name = Some(name.into());
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
    C: spacetimedb_sdk::__codegen::DbConnection<Module = M> + DbContext + ConnectionRunner,
    M: spacetimedb_sdk::__codegen::SpacetimeModule<DbConnection = C>,
> Plugin for StdbPlugin<C, M>
{
    fn build(&self, app: &mut App) {
        self.uri
            .clone()
            .expect("No uri set for StdbPlugin. Set it with the with_uri() function");
        self.database_name.clone().expect(
            "No database name set for StdbPlugin. Set it with the with_database_name() function",
        );

        let (send_connected, recv_connected) = channel::<StdbConnectedMessage>();
        let (send_disconnected, recv_disconnected) = channel::<StdbDisconnectedMessage>();
        let (send_connect_error, recv_connect_error) = channel::<StdbConnectionErrorMessage>();
        app.add_message_channel::<StdbConnectionErrorMessage>(recv_connect_error)
            .add_message_channel::<StdbConnectedMessage>(recv_connected)
            .add_message_channel::<StdbDisconnectedMessage>(recv_disconnected);

        // On wasm, always use delayed connect since build() is sync but connection is async
        #[cfg(target_arch = "wasm32")]
        let delayed_connect = true;
        #[cfg(not(target_arch = "wasm32"))]
        let delayed_connect = self.delayed_connect;

        if delayed_connect {
            // Store configuration AND table/reducer registrations for later connection
            app.insert_resource(StdbPluginConfig::<C, M> {
                database_name: self.database_name.clone().unwrap(),
                uri: self.uri.clone().unwrap(),
                compression: self.compression.unwrap_or_default(),
                send_connected,
                send_disconnected,
                send_connect_error,
                _phantom: PhantomData,
            });
            
            // Clone the Arc pointers to share the data with connect_with_token
            let plugin_for_later = DelayedPluginData::<C, M> {
                table_registers: Arc::clone(&self.table_registers),
                message_senders: Arc::clone(&self.message_senders),
            };
            app.insert_non_send_resource(plugin_for_later);
            
            return; // Skip connection - it will be created later via connect_with_token
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let conn = DbConnectionBuilder::<M>::new()
                .with_database_name(self.database_name.clone().unwrap())
                .with_uri(self.uri.clone().unwrap())
                .with_token(self.token.clone())
                .with_compression(self.compression.unwrap_or_default())
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

            // A 'static ref is needed for the connection to register tables and reducers.
            // This is fine because only a small and fixed amount of memory will be leaked;
            // conn has to live until the end of the program anyways.
            let conn = Box::<C>::leak(Box::new(conn));

            {
                let table_regs = self.table_registers.lock().unwrap();
                for table_register in table_regs.iter() {
                    table_register(self, app, conn.db());
                }
            }

            conn.start_event_loop();

            app.insert_resource(StdbConnection::new(conn));
        }
    }
}
