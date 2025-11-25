use crate::bridge::SpacetimeDBBridge;

/// Trait for reducers that can be called on the SpacetimeDB server
///
/// Implement this trait for your reducer types. The easiest way is to use the
/// `define_reducer!` macro provided by generated code.
///
/// # Example
/// ```ignore
/// use bevy_spacetimedb_wasm::Reducer;
/// use serde::Serialize;
///
/// pub struct SpawnPlayer;
///
/// impl Reducer for SpawnPlayer {
///     const NAME: &'static str = "spawn_player";
///     type Args = (String, f32, f32); // (name, x, y)
/// }
/// ```
pub trait Reducer: Send + Sync + 'static {
    /// The name of the reducer as defined in your SpacetimeDB module
    const NAME: &'static str;

    /// The argument types for this reducer
    ///
    /// Use a tuple for multiple arguments: `(String, u32, f32)`
    /// Use a single type for one argument: `String`
    /// Use `()` for no arguments
    type Args: serde::Serialize;
}

/// Helper for calling reducers on the SpacetimeDB server
///
/// Obtained via `StdbConnection::reducers()`.
pub struct ReducerCaller<'a> {
    pub(crate) bridge: &'a SpacetimeDBBridge,
    pub(crate) connection_id: u32,
}

impl<'a> ReducerCaller<'a> {
    /// Call a reducer on the SpacetimeDB server
    ///
    /// This is fire-and-forget - the call happens asynchronously and any errors
    /// will be logged to the console.
    ///
    /// # Example
    /// ```ignore
    /// fn spawn_system(stdb: Res<StdbConnection>) {
    ///     stdb.reducers()
    ///         .call::<SpawnPlayer>(("Alice".to_string(), 10.0, 20.0))
    ///         .expect("Failed to serialize reducer args");
    /// }
    /// ```
    pub fn call<R: Reducer>(&self, args: R::Args) -> Result<(), serde_wasm_bindgen::Error> {
        // Serialize the arguments to JsValue
        let args_value = serde_wasm_bindgen::to_value(&args)?;

        let connection_id = self.connection_id;
        let reducer_name = R::NAME.to_string();

        // Call the reducer and get the promise
        let promise = self.bridge.call_reducer(connection_id, &reducer_name, args_value);

        // Call the reducer asynchronously
        wasm_bindgen_futures::spawn_local(async move {
            match wasm_bindgen_futures::JsFuture::from(promise).await
            {
                Ok(_) => {
                    web_sys::console::log_1(
                        &format!("Called reducer: {}", reducer_name).into(),
                    );
                }
                Err(e) => {
                    web_sys::console::error_1(
                        &format!("Failed to call reducer {}: {:?}", reducer_name, e).into(),
                    );
                }
            }
        });

        Ok(())
    }
}

/// Macro for defining reducers
///
/// # Example
/// ```ignore
/// define_reducer!(SpawnPlayer(name: String, x: f32, y: f32));
/// define_reducer!(MovePlayer(id: u64, x: f32, y: f32));
/// define_reducer!(DeletePlayer(id: u64));
/// ```
#[macro_export]
macro_rules! define_reducer {
    ($name:ident($($arg:ident: $ty:ty),*)) => {
        pub struct $name;

        impl $crate::Reducer for $name {
            const NAME: &'static str = stringify!($name);
            type Args = ($($ty,)*);
        }
    };
    ($name:ident()) => {
        pub struct $name;

        impl $crate::Reducer for $name {
            const NAME: &'static str = stringify!($name);
            type Args = ();
        }
    };
}
