//! Bridge to the SpacetimeDB TypeScript SDK via wasm-bindgen
//!
//! This module provides Rust bindings to the JavaScript bridge layer that wraps
//! the SpacetimeDB TypeScript SDK. The bridge must be initialized in JavaScript
//! before the WASM module is loaded.

use wasm_bindgen::prelude::*;

/// External bindings to the JavaScript SpacetimeDB bridge
///
/// These functions are implemented in `js/spacetimedb-bridge.js`
#[wasm_bindgen]
extern "C" {
    /// The SpacetimeDB bridge singleton
    #[wasm_bindgen(js_name = SpacetimeDBBridge)]
    #[derive(Clone)]
    pub type SpacetimeDBBridge;

    /// Create a new connection to SpacetimeDB
    #[wasm_bindgen(method, js_name = createConnection)]
    pub fn create_connection(
        this: &SpacetimeDBBridge,
        uri: &str,
        module_name: &str,
        auth_token: Option<String>,
    ) -> u32;

    /// Connect to the SpacetimeDB server
    #[wasm_bindgen(method)]
    pub fn connect(this: &SpacetimeDBBridge, connection_id: u32) -> js_sys::Promise;

    /// Disconnect from the SpacetimeDB server
    #[wasm_bindgen(method)]
    pub fn disconnect(this: &SpacetimeDBBridge, connection_id: u32) -> js_sys::Promise;

    /// Register a callback for connection events
    #[wasm_bindgen(method, js_name = onConnect)]
    pub fn on_connect(this: &SpacetimeDBBridge, connection_id: u32, callback_id: u32);

    /// Register a callback for disconnection events
    #[wasm_bindgen(method, js_name = onDisconnect)]
    pub fn on_disconnect(this: &SpacetimeDBBridge, connection_id: u32, callback_id: u32);

    /// Register a callback for connection error events
    #[wasm_bindgen(method, js_name = onConnectionError)]
    pub fn on_connection_error(this: &SpacetimeDBBridge, connection_id: u32, callback_id: u32);

    /// Call a reducer on the SpacetimeDB server
    #[wasm_bindgen(method, js_name = callReducer)]
    pub fn call_reducer(
        this: &SpacetimeDBBridge,
        connection_id: u32,
        reducer_name: &str,
        args: JsValue,
    ) -> js_sys::Promise;

    /// Subscribe to a SQL query
    #[wasm_bindgen(method)]
    pub fn subscribe(
        this: &SpacetimeDBBridge,
        connection_id: u32,
        query: &str,
    ) -> js_sys::Promise;

    /// Subscribe to table events
    #[wasm_bindgen(method, js_name = subscribeTable)]
    pub fn subscribe_table(
        this: &SpacetimeDBBridge,
        connection_id: u32,
        table_name: &str,
        on_insert_id: Option<u32>,
        on_update_id: Option<u32>,
        on_delete_id: Option<u32>,
    );

    /// Register a JavaScript callback
    #[wasm_bindgen(method, js_name = registerCallback)]
    pub fn register_callback(this: &SpacetimeDBBridge, callback: &js_sys::Function) -> u32;

    /// Unregister a callback
    #[wasm_bindgen(method, js_name = unregisterCallback)]
    pub fn unregister_callback(this: &SpacetimeDBBridge, callback_id: u32);
}

/// Get the global SpacetimeDB bridge instance
///
/// # Panics
///
/// Panics with a helpful error message if the bridge is not initialized.
/// The bridge must be initialized in JavaScript before loading the WASM module:
///
/// ```html
/// <script type="module">
///   import { SpacetimeDBBridge } from './js/spacetimedb-bridge.js';
///   window.__SPACETIMEDB_BRIDGE__ = new SpacetimeDBBridge();
///
///   // Then load your WASM module
///   import init from './pkg/my_game.js';
///   await init();
/// </script>
/// ```
pub fn get_bridge() -> SpacetimeDBBridge {
    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = global, js_name = __SPACETIMEDB_BRIDGE__)]
        static BRIDGE_GLOBAL: JsValue;
    }

    // Try browser window first
    if let Some(window) = web_sys::window() {
        if let Ok(bridge) = js_sys::Reflect::get(&window, &JsValue::from_str("__SPACETIMEDB_BRIDGE__")) {
            if !bridge.is_undefined() && !bridge.is_null() {
                return bridge.unchecked_into();
            }
        }
    }

    // Try Node.js global
    if !BRIDGE_GLOBAL.is_undefined() && !BRIDGE_GLOBAL.is_null() {
        return BRIDGE_GLOBAL.clone().unchecked_into();
    }

    panic!(
        "\n\n\
        ╔══════════════════════════════════════════════════════════════════════════════╗\n\
        ║ SpacetimeDB TypeScript SDK Bridge Not Found!                                ║\n\
        ╠══════════════════════════════════════════════════════════════════════════════╣\n\
        ║                                                                              ║\n\
        ║ The bevy_spacetimedb_wasm plugin requires the SpacetimeDB TypeScript SDK    ║\n\
        ║ bridge to be loaded and initialized BEFORE the WASM module loads.           ║\n\
        ║                                                                              ║\n\
        ║ For browser environments, add this to your HTML file:                       ║\n\
        ║                                                                              ║\n\
        ║   <script type=\"module\">                                                    ║\n\
        ║     import {{ SpacetimeDBBridge }} from './js/spacetimedb-bridge.js';        ║\n\
        ║     window.__SPACETIMEDB_BRIDGE__ = new SpacetimeDBBridge();                ║\n\
        ║                                                                              ║\n\
        ║     // THEN load your WASM module                                           ║\n\
        ║     import init from './pkg/my_game.js';                                    ║\n\
        ║     await init();                                                           ║\n\
        ║   </script>                                                                 ║\n\
        ║                                                                              ║\n\
        ║ For Node.js/test environments:                                              ║\n\
        ║   global.__SPACETIMEDB_BRIDGE__ = new SpacetimeDBBridge();                 ║\n\
        ║                                                                              ║\n\
        ║ The bridge file should be at: bevy_spacetimedb/js/spacetimedb-bridge.js    ║\n\
        ║                                                                              ║\n\
        ╚══════════════════════════════════════════════════════════════════════════════╝\n\n"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_bridge_available() {
        // This test will fail if run without the bridge loaded
        // In a real test environment, you'd need to load the bridge first
        // For now, this serves as documentation
        let _bridge = get_bridge();
    }
}
