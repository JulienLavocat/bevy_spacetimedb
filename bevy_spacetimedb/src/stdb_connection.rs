use bevy::prelude::Resource;
use crate::bridge::SpacetimeDBBridge;
use crate::reducers::ReducerCaller;

/// A connection to the SpacetimeDB server via the TypeScript SDK bridge
#[derive(Resource, Clone)]
pub struct StdbConnection {
    /// The JavaScript bridge instance (wrapped in SendSyncWrapper for WASM single-threaded context)
    pub(crate) bridge: SendSyncWrapper<SpacetimeDBBridge>,
    /// The connection ID
    pub(crate) connection_id: u32,
}

/// Wrapper to make JS types Send + Sync in WASM single-threaded context
/// SAFETY: WASM is single-threaded, so Send + Sync are safe
#[derive(Clone)]
pub(crate) struct SendSyncWrapper<T>(T);

unsafe impl<T> Send for SendSyncWrapper<T> {}
unsafe impl<T> Sync for SendSyncWrapper<T> {}

impl<T> std::ops::Deref for SendSyncWrapper<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl StdbConnection {
    /// Create a new connection resource
    pub(crate) fn new(bridge: SpacetimeDBBridge, connection_id: u32) -> Self {
        Self {
            bridge: SendSyncWrapper(bridge),
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
    pub fn reducers(&self) -> ReducerCaller<'_> {
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
        let connection_id = self.connection_id;

        // Call disconnect directly - no async needed since it returns a Promise
        let promise = self.bridge.disconnect(connection_id);

        wasm_bindgen_futures::spawn_local(async move {
            match wasm_bindgen_futures::JsFuture::from(promise).await {
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
