use crate::{
    bridge::get_bridge, AddEventChannelAppExtensions, StdbConnectedEvent,
    StdbConnectionErrorEvent, StdbDisconnectedEvent, StdbConnection,
    tables::TableConfig,
};
use bevy::app::{App, Plugin};
use wasm_bindgen::prelude::*;

/// The main plugin for connecting SpacetimeDB to your Bevy application
///
/// This plugin handles:
/// - Initializing the connection to SpacetimeDB via the TypeScript SDK bridge
/// - Setting up table event subscriptions
/// - Providing connection lifecycle events
///
/// # Example
/// ```ignore
/// use bevy::prelude::*;
/// use bevy_spacetimedb_wasm::*;
///
/// fn main() {
///     App::new()
///         .add_plugins(DefaultPlugins)
///         .add_plugins(
///             StdbPlugin::default()
///                 .with_uri("ws://localhost:3000")
///                 .with_module_name("my_game")
///                 .add_table::<Player>()
///                 .add_table::<Lobby>()
///         )
///         .add_systems(Update, handle_players)
///         .run();
/// }
///
/// fn handle_players(mut events: EventReader<InsertEvent<Player>>) {
///     for event in events.read() {
///         println!("New player: {:?}", event.row);
///     }
/// }
/// ```
pub struct StdbPlugin {
    /// The WebSocket URI of the SpacetimeDB server
    uri: Option<String>,
    /// The name of the SpacetimeDB module
    module_name: Option<String>,
    /// Optional authentication token
    auth_token: Option<String>,
    /// Table configurations
    pub(crate) table_configs: Vec<TableConfig>,
}

impl Default for StdbPlugin {
    fn default() -> Self {
        Self {
            uri: None,
            module_name: None,
            auth_token: None,
            table_configs: Vec::new(),
        }
    }
}

impl StdbPlugin {
    /// Set the URI of the SpacetimeDB host
    ///
    /// The URI should be a WebSocket URL, e.g., `"ws://localhost:3000"` or
    /// `"wss://myserver.com"`.
    ///
    /// # Example
    /// ```ignore
    /// StdbPlugin::default()
    ///     .with_uri("ws://localhost:3000")
    /// ```
    pub fn with_uri(mut self, uri: impl Into<String>) -> Self {
        self.uri = Some(uri.into());
        self
    }

    /// Set the name of the SpacetimeDB module
    ///
    /// This should match the module name defined in your SpacetimeDB server.
    ///
    /// # Example
    /// ```ignore
    /// StdbPlugin::default()
    ///     .with_module_name("my_game")
    /// ```
    pub fn with_module_name(mut self, name: impl Into<String>) -> Self {
        self.module_name = Some(name.into());
        self
    }

    /// Supply an authentication token
    ///
    /// The token should be an OpenID Connect compliant JSON Web Token.
    /// If not provided, the server will generate a new anonymous identity.
    ///
    /// # Example
    /// ```ignore
    /// StdbPlugin::default()
    ///     .with_auth_token("eyJ0eXAiOiJKV1QiLCJhbGciOiJFUzI1NiJ9...")
    /// ```
    pub fn with_auth_token(mut self, token: impl Into<String>) -> Self {
        self.auth_token = Some(token.into());
        self
    }
}

impl Plugin for StdbPlugin {
    fn build(&self, app: &mut App) {
        // Validate configuration
        let uri = self
            .uri
            .as_ref()
            .expect("No URI set for StdbPlugin. Set it with .with_uri()");
        let module_name = self
            .module_name
            .as_ref()
            .expect("No module name set for StdbPlugin. Set it with .with_module_name()");

        // Get the JavaScript bridge
        let bridge = get_bridge();

        // Create the connection
        let connection_id = bridge.create_connection(uri, module_name, self.auth_token.clone());

        web_sys::console::log_1(
            &format!(
                "Created SpacetimeDB connection {} to {}/{}",
                connection_id, uri, module_name
            )
            .into(),
        );

        // Setup connection lifecycle event channels
        let (connected_send, connected_recv) = std::sync::mpsc::channel::<StdbConnectedEvent>();
        let (disconnected_send, disconnected_recv) =
            std::sync::mpsc::channel::<StdbDisconnectedEvent>();
        let (error_send, error_recv) = std::sync::mpsc::channel::<StdbConnectionErrorEvent>();

        app.add_event_channel(connected_recv)
            .add_event_channel(disconnected_recv)
            .add_event_channel(error_recv);

        // Register connection lifecycle callbacks
        let connected_cb = Closure::wrap(Box::new(move || {
            let _ = connected_send.send(StdbConnectedEvent {});
        }) as Box<dyn Fn()>);

        let disconnected_cb = Closure::wrap(Box::new(move |err: JsValue| {
            let err_msg = err.as_string();
            let _ = disconnected_send.send(StdbDisconnectedEvent { err: err_msg });
        }) as Box<dyn Fn(JsValue)>);

        let error_cb = Closure::wrap(Box::new(move |err: JsValue| {
            let err_msg = err.as_string().unwrap_or_else(|| "Unknown error".to_string());
            let _ = error_send.send(StdbConnectionErrorEvent { err: err_msg });
        }) as Box<dyn Fn(JsValue)>);

        // Register the callbacks with the bridge
        let connected_id = bridge.register_callback(connected_cb.as_ref().unchecked_ref());
        let disconnected_id = bridge.register_callback(disconnected_cb.as_ref().unchecked_ref());
        let error_id = bridge.register_callback(error_cb.as_ref().unchecked_ref());

        bridge.on_connect(connection_id, connected_id);
        bridge.on_disconnect(connection_id, disconnected_id);
        bridge.on_connection_error(connection_id, error_id);

        // Keep the closures alive for the lifetime of the application
        connected_cb.forget();
        disconnected_cb.forget();
        error_cb.forget();

        // Setup table subscriptions
        for table_config in &self.table_configs {
            (table_config.setup_fn)(&bridge, connection_id, &table_config.events, app);
        }

        // Create and insert the connection resource
        let connection = StdbConnection::new(bridge.clone(), connection_id);
        app.insert_resource(connection);

        // Connect to the server asynchronously
        wasm_bindgen_futures::spawn_local(async move {
            web_sys::console::log_1(
                &format!("Connecting to SpacetimeDB connection {}...", connection_id).into(),
            );

            match wasm_bindgen_futures::JsFuture::from(bridge.connect(connection_id)).await {
                Ok(_) => {
                    web_sys::console::log_1(
                        &format!("Successfully connected to SpacetimeDB (connection {})", connection_id)
                            .into(),
                    );
                }
                Err(e) => {
                    web_sys::console::error_1(
                        &format!("Failed to connect to SpacetimeDB: {:?}", e).into(),
                    );
                }
            }
        });
    }
}
