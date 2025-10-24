use crate::{
    bridge::SpacetimeDBBridge, AddEventChannelAppExtensions, DeleteEvent, InsertEvent,
    InsertUpdateEvent, StdbPlugin, UpdateEvent,
};
use bevy::app::App;
use std::sync::mpsc::Sender;
use wasm_bindgen::prelude::*;

/// Trait for table rows that can be synchronized from SpacetimeDB
///
/// Implement this for your table types (usually auto-generated).
///
/// # Example
/// ```ignore
/// use bevy_spacetimedb_wasm::TableRow;
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Debug, Clone, Deserialize, Serialize)]
/// pub struct Player {
///     pub id: u64,
///     pub name: String,
///     pub x: f32,
///     pub y: f32,
/// }
///
/// impl TableRow for Player {
///     const TABLE_NAME: &'static str = "players";
/// }
/// ```
pub trait TableRow: serde::de::DeserializeOwned + Send + Sync + Clone + 'static {
    /// The name of the table in the SpacetimeDB module
    const TABLE_NAME: &'static str;
}

/// Configuration for which table events to subscribe to
#[derive(Debug, Default, Clone, Copy)]
pub struct TableEvents {
    /// Whether to subscribe to row insertions (sends `InsertEvent<T>`)
    pub insert: bool,
    /// Whether to subscribe to row updates (sends `UpdateEvent<T>`)
    pub update: bool,
    /// Whether to subscribe to row deletions (sends `DeleteEvent<T>`)
    pub delete: bool,
}

impl TableEvents {
    /// Subscribe to all table events (insert, update, delete)
    pub fn all() -> Self {
        Self {
            insert: true,
            update: true,
            delete: true,
        }
    }

    /// Subscribe to insert and delete events, but not updates
    pub fn no_update() -> Self {
        Self {
            insert: true,
            update: false,
            delete: true,
        }
    }

    /// Subscribe only to insert events
    pub fn insert_only() -> Self {
        Self {
            insert: true,
            update: false,
            delete: false,
        }
    }

    /// Subscribe only to update events
    pub fn update_only() -> Self {
        Self {
            insert: false,
            update: true,
            delete: false,
        }
    }

    /// Subscribe only to delete events
    pub fn delete_only() -> Self {
        Self {
            insert: false,
            update: false,
            delete: true,
        }
    }
}

impl StdbPlugin {
    /// Register a table with all events enabled
    ///
    /// # Example
    /// ```ignore
    /// StdbPlugin::default()
    ///     .add_table::<Player>()
    ///     .add_table::<Lobby>()
    /// ```
    pub fn add_table<T: TableRow>(self) -> Self {
        self.add_partial_table::<T>(TableEvents::all())
    }

    /// Register a table with specific events enabled
    ///
    /// # Example
    /// ```ignore
    /// StdbPlugin::default()
    ///     .add_partial_table::<Player>(TableEvents::all())
    ///     .add_partial_table::<Message>(TableEvents::no_update())
    /// ```
    pub fn add_partial_table<T: TableRow>(mut self, events: TableEvents) -> Self {
        self.table_configs.push(TableConfig {
            table_name: T::TABLE_NAME.to_string(),
            events,
            setup_fn: Box::new(setup_table_events::<T>),
        });
        self
    }
}

/// Internal table configuration
pub(crate) struct TableConfig {
    pub table_name: String,
    pub events: TableEvents,
    pub setup_fn: Box<dyn Fn(&SpacetimeDBBridge, u32, &TableEvents, &mut App) + Send + Sync>,
}

/// Setup event subscriptions for a table
fn setup_table_events<T: TableRow>(
    bridge: &SpacetimeDBBridge,
    connection_id: u32,
    events: &TableEvents,
    app: &mut App,
) {
    let mut insert_callback_id = None;
    let mut update_callback_id = None;
    let mut delete_callback_id = None;

    // Setup insert events
    if events.insert {
        let (send, recv) = std::sync::mpsc::channel::<InsertEvent<T>>();
        app.add_event_channel(recv);

        let callback = Closure::wrap(Box::new(move |data: JsValue| {
            if let Some(json) = data.as_string() {
                match serde_json::from_str::<serde_json::Value>(&json) {
                    Ok(value) => {
                        if let Ok(row) = serde_json::from_value::<T>(value["row"].clone()) {
                            let _ = send.send(InsertEvent { row });
                        } else {
                            web_sys::console::error_1(
                                &format!(
                                    "Failed to deserialize row for table {}",
                                    T::TABLE_NAME
                                )
                                .into(),
                            );
                        }
                    }
                    Err(e) => {
                        web_sys::console::error_1(
                            &format!("Failed to parse JSON for insert event: {}", e).into(),
                        );
                    }
                }
            }
        }) as Box<dyn Fn(JsValue)>);

        let id = bridge.register_callback(callback.as_ref().unchecked_ref());
        callback.forget(); // Keep the closure alive
        insert_callback_id = Some(id);
    }

    // Setup update events
    if events.update {
        let (send, recv) = std::sync::mpsc::channel::<UpdateEvent<T>>();
        app.add_event_channel(recv);

        let callback = Closure::wrap(Box::new(move |data: JsValue| {
            if let Some(json) = data.as_string() {
                match serde_json::from_str::<serde_json::Value>(&json) {
                    Ok(value) => {
                        let old_result = serde_json::from_value::<T>(value["oldRow"].clone());
                        let new_result = serde_json::from_value::<T>(value["newRow"].clone());

                        if let (Ok(old), Ok(new)) = (old_result, new_result) {
                            let _ = send.send(UpdateEvent { old, new });
                        } else {
                            web_sys::console::error_1(
                                &format!(
                                    "Failed to deserialize rows for table {}",
                                    T::TABLE_NAME
                                )
                                .into(),
                            );
                        }
                    }
                    Err(e) => {
                        web_sys::console::error_1(
                            &format!("Failed to parse JSON for update event: {}", e).into(),
                        );
                    }
                }
            }
        }) as Box<dyn Fn(JsValue)>);

        let id = bridge.register_callback(callback.as_ref().unchecked_ref());
        callback.forget();
        update_callback_id = Some(id);
    }

    // Setup delete events
    if events.delete {
        let (send, recv) = std::sync::mpsc::channel::<DeleteEvent<T>>();
        app.add_event_channel(recv);

        let callback = Closure::wrap(Box::new(move |data: JsValue| {
            if let Some(json) = data.as_string() {
                match serde_json::from_str::<serde_json::Value>(&json) {
                    Ok(value) => {
                        if let Ok(row) = serde_json::from_value::<T>(value["row"].clone()) {
                            let _ = send.send(DeleteEvent { row });
                        } else {
                            web_sys::console::error_1(
                                &format!(
                                    "Failed to deserialize row for table {}",
                                    T::TABLE_NAME
                                )
                                .into(),
                            );
                        }
                    }
                    Err(e) => {
                        web_sys::console::error_1(
                            &format!("Failed to parse JSON for delete event: {}", e).into(),
                        );
                    }
                }
            }
        }) as Box<dyn Fn(JsValue)>);

        let id = bridge.register_callback(callback.as_ref().unchecked_ref());
        callback.forget();
        delete_callback_id = Some(id);
    }

    // Setup InsertUpdate events if both insert and update are enabled
    if events.insert && events.update {
        let (send, recv) = std::sync::mpsc::channel::<InsertUpdateEvent<T>>();
        app.add_event_channel(recv);

        // We'll create duplicate callbacks that send to the InsertUpdateEvent channel
        // This is simpler than trying to share the same callback

        let send_insert = send.clone();
        let insert_update_callback = Closure::wrap(Box::new(move |data: JsValue| {
            if let Some(json) = data.as_string() {
                match serde_json::from_str::<serde_json::Value>(&json) {
                    Ok(value) => {
                        if let Ok(row) = serde_json::from_value::<T>(value["row"].clone()) {
                            let _ = send_insert.send(InsertUpdateEvent { old: None, new: row });
                        }
                    }
                    Err(_) => {}
                }
            }
        }) as Box<dyn Fn(JsValue)>);

        let send_update = send;
        let update_update_callback = Closure::wrap(Box::new(move |data: JsValue| {
            if let Some(json) = data.as_string() {
                match serde_json::from_str::<serde_json::Value>(&json) {
                    Ok(value) => {
                        let old_result = serde_json::from_value::<T>(value["oldRow"].clone());
                        let new_result = serde_json::from_value::<T>(value["newRow"].clone());

                        if let (Ok(old), Ok(new)) = (old_result, new_result) {
                            let _ =
                                send_update.send(InsertUpdateEvent { old: Some(old), new });
                        }
                    }
                    Err(_) => {}
                }
            }
        }) as Box<dyn Fn(JsValue)>);

        // We don't register these with the bridge directly since they're duplicates
        // of the insert/update callbacks. Just keep them alive.
        insert_update_callback.forget();
        update_update_callback.forget();
    }

    // Subscribe to the table with the registered callbacks
    bridge.subscribe_table(
        connection_id,
        T::TABLE_NAME,
        insert_callback_id,
        update_callback_id,
        delete_callback_id,
    );

    web_sys::console::log_1(
        &format!(
            "Subscribed to table {} (insert: {}, update: {}, delete: {})",
            T::TABLE_NAME,
            events.insert,
            events.update,
            events.delete
        )
        .into(),
    );
}
