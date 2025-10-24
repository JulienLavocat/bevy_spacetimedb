use bevy::prelude::Resource;
use crate::bridge::SpacetimeDBBridge;
use crate::reducers::ReducerCaller;

/// A connection to the SpacetimeDB server via the TypeScript SDK bridge
#[derive(Resource, Clone)]
pub struct StdbConnection {
    /// The JavaScript bridge instance
    pub(crate) bridge: SpacetimeDBBridge,
    /// The connection ID
    pub(crate) connection_id: u32,
}

impl StdbConnection {
    /// Create a new connection resource
    pub(crate) fn new(bridge: SpacetimeDBBridge, connection_id: u32) -> Self {
        Self {
            bridge,
            connection_id,
        }
    }

    /// Get a reducer caller for invoking reducers on the SpacetimeDB server
    ///
    /// # Example
    /// ```ignore
    /// fn my_system(stdb: Res<StdbConnection>) {
    ///     stdb.reducers()
    ///         .call::<SpawnPlayer>(("Alice".to_string(), 0.0, 0.0))
    ///         .expect("Failed to call reducer");
    /// }
    /// ```
    pub fn reducers(&self) -> ReducerCaller {
        ReducerCaller {
            bridge: &self.bridge,
            connection_id: self.connection_id,
        }
    }

    /// Get the connection ID
    pub fn connection_id(&self) -> u32 {
        self.connection_id
    }

    /// Disconnect from the SpacetimeDB server
    ///
    /// This returns immediately and the disconnection happens asynchronously.
    pub fn disconnect(&self) {
        let bridge = self.bridge.clone();
        let connection_id = self.connection_id;

        wasm_bindgen_futures::spawn_local(async move {
            match wasm_bindgen_futures::JsFuture::from(bridge.disconnect(connection_id)).await {
                Ok(_) => {
                    web_sys::console::log_1(&format!("Disconnected from SpacetimeDB").into());
                }
                Err(e) => {
                    web_sys::console::error_1(
                        &format!("Failed to disconnect: {:?}", e).into(),
                    );
                }
            }
        });
    }
}
