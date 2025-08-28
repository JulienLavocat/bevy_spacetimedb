use std::{
    any::TypeId,
    sync::mpsc::{Sender, channel},
};

use bevy::app::App;
use spacetimedb_sdk::{__codegen as spacetime_codegen, Table, TableWithPrimaryKey};

use crate::AddEventChannelAppExtensions;
// Imports are marked as unused but they are useful for linking types in docs.
// #[allow(unused_imports)]
use crate::{DeleteEvent, InsertEvent, InsertUpdateEvent, StdbPlugin, UpdateEvent};

/// Passed into [`StdbPlugin::add_table`] to determine which table events to register.
#[derive(Debug, Default, Clone, Copy)]
pub struct TableEvents {
    /// Whether to register to a row insertion. Registers the [`InsertEvent`] event for the table.
    ///
    /// Use along with update to register the [`InsertUpdateEvent`] event as well.
    pub insert: bool,

    /// Whether to register to a row update. Registers the [`UpdateEvent`] event for the table.
    ///
    /// Use along with insert to register the [`InsertUpdateEvent`] event as well.
    pub update: bool,

    /// Whether to register to a row deletion. Registers the [`DeleteEvent`] event for the table.
    pub delete: bool,
}

impl TableEvents {
    /// Register all table events
    pub fn all() -> Self {
        Self {
            insert: true,
            update: true,
            delete: true,
        }
    }

    pub fn no_update() -> Self {
        Self {
            insert: true,
            update: false,
            delete: true,
        }
    }
}

/// Passed into [`StdbPlugin::add_table_without_pk`] to determine which table events to register.
/// Specifically for tables with no Primary keys
#[derive(Debug, Default, Clone, Copy)]
pub struct TableEventsWithoutPrimaryKey {
    /// Same as [`TableEvents::insert`]
    pub insert: bool,
    /// Same as [`TableEvents::delete`]
    pub delete: bool,
}

impl TableEventsWithoutPrimaryKey {
    /// Register all available table events
    pub fn all() -> Self {
        Self {
            insert: true,
            delete: true,
        }
    }
}

impl<
    C: spacetime_codegen::DbConnection<Module = M> + spacetimedb_sdk::DbContext,
    M: spacetime_codegen::SpacetimeModule<DbConnection = C>,
> StdbPlugin<C, M>
{
    /// Registers a table for the bevy application with all events enabled.
    pub fn add_table<TRow, TTable, F>(self, accessor: F) -> Self
    where
        TRow: Send + Sync + Clone + 'static,
        TTable: Table<Row = TRow> + TableWithPrimaryKey<Row = TRow>,
        F: 'static + Send + Sync + Fn(&'static C::DbView) -> TTable,
    {
        self.add_partial_table(accessor, TableEvents::all())
    }

    ///Registers a table for the bevy application with the specified events in the `events` parameter.
    pub fn add_partial_table<TRow, TTable, F>(mut self, accessor: F, events: TableEvents) -> Self
    where
        TRow: Send + Sync + Clone + 'static,
        TTable: Table<Row = TRow> + TableWithPrimaryKey<Row = TRow>,
        F: 'static + Send + Sync + Fn(&'static C::DbView) -> TTable,
    {
        // A closure that sets up events for the table
        let register = move |plugin: &Self, app: &mut App, db: &'static C::DbView| {
            let table = accessor(db);
            if events.insert {
                plugin.on_insert(app, &table);
            }
            if events.delete {
                plugin.on_delete(app, &table);
            }
            if events.update {
                plugin.on_update(app, &table);
            }
            if events.update && events.insert {
                plugin.on_insert_update(app, &table);
            }
        };

        // Store this table, and later when the plugin is built, call them on .
        self.table_registers.push(Box::new(register));

        self
    }

    ///Registers a table without a private key for the bevy application with the specified events in the `events` parameter.
    pub fn add_table_without_pk<TRow, TTable, F>(mut self, accessor: F, events: TableEventsWithoutPrimaryKey) -> Self
    where
        TRow: Send + Sync + Clone + 'static,
        TTable: Table<Row = TRow>,
        F: 'static + Send + Sync + Fn(&'static C::DbView) -> TTable,
    {
        // A closure that sets up events for the table
        let register = move |plugin: &Self, app: &mut App, db: &'static C::DbView| {
            let table = accessor(db);
            if events.insert {
                plugin.on_insert(app, &table);
            }
            if events.delete {
                plugin.on_delete(app, &table);
            }
        };

        // Store this table, and later when the plugin is built, call them on .
        self.table_registers.push(Box::new(register));

        self
    }

    /// Register a Bevy event of type InsertEvent<TRow> for the `on_insert` event on the provided table.
    fn on_insert<TRow>(&self, app: &mut App, table: &impl Table<Row = TRow>) -> &Self
    where
        TRow: Send + Sync + Clone + 'static,
    {
        let type_id = TypeId::of::<InsertEvent<TRow>>();

        let mut map = self.event_senders.lock().unwrap();

        let sender = map
            .entry(type_id)
            .or_insert_with(|| {
                let (send, recv) = channel::<InsertEvent<TRow>>();
                app.add_event_channel(recv);
                Box::new(send)
            })
            .downcast_ref::<Sender<InsertEvent<TRow>>>()
            .expect("Sender type mismatch")
            .clone();

        table.on_insert(move |_ctx, row| {
            let event = InsertEvent { row: row.clone() };
            let _ = sender.send(event);
        });

        self
    }

    /// Register a Bevy event of type DeleteEvent<TRow> for the `on_delete` event on the provided table.
    fn on_delete<TRow>(&self, app: &mut App, table: &impl Table<Row = TRow>) -> &Self
    where
        TRow: Send + Sync + Clone + 'static,
    {
        let type_id = TypeId::of::<DeleteEvent<TRow>>();

        let mut map = self.event_senders.lock().unwrap();
        let sender = map
            .entry(type_id)
            .or_insert_with(|| {
                let (send, recv) = channel::<DeleteEvent<TRow>>();
                app.add_event_channel(recv);
                Box::new(send)
            })
            .downcast_ref::<Sender<DeleteEvent<TRow>>>()
            .expect("Sender type mismatch")
            .clone();

        table.on_delete(move |_ctx, row| {
            let event = DeleteEvent { row: row.clone() };
            let _ = sender.send(event);
        });

        self
    }

    /// Register a Bevy event of type UpdateEvent<TRow> for the `on_update` event on the provided table.
    fn on_update<TRow, TTable>(&self, app: &mut App, table: &TTable) -> &Self
    where
        TRow: Send + Sync + Clone + 'static,
        TTable: Table<Row = TRow> + TableWithPrimaryKey<Row = TRow>,
    {
        let type_id = TypeId::of::<UpdateEvent<TRow>>();

        let mut map = self.event_senders.lock().unwrap();
        let sender = map
            .entry(type_id)
            .or_insert_with(|| {
                let (send, recv) = channel::<UpdateEvent<TRow>>();
                app.add_event_channel(recv);
                Box::new(send)
            })
            .downcast_ref::<Sender<UpdateEvent<TRow>>>()
            .expect("Sender type mismatch")
            .clone();

        table.on_update(move |_ctx, old, new| {
            let event = UpdateEvent {
                old: old.clone(),
                new: new.clone(),
            };
            let _ = sender.send(event);
        });

        self
    }

    /// Register a Bevy event of type InsertUpdateEvent<TRow> for the `on_insert` and `on_update` events on the provided table.
    fn on_insert_update<TRow, TTable>(&self, app: &mut App, table: &TTable) -> &Self
    where
        TRow: Send + Sync + Clone + 'static,
        TTable: Table<Row = TRow> + TableWithPrimaryKey<Row = TRow>,
    {
        let type_id = TypeId::of::<InsertUpdateEvent<TRow>>();

        let mut map = self.event_senders.lock().unwrap();
        let send = map
            .entry(type_id)
            .or_insert_with(|| {
                let (send, recv) = channel::<InsertUpdateEvent<TRow>>();
                app.add_event_channel(recv);
                Box::new(send)
            })
            .downcast_ref::<Sender<InsertUpdateEvent<TRow>>>()
            .expect("Sender type mismatch")
            .clone();

        let send_update = send.clone();
        table.on_update(move |_ctx, old, new| {
            let event = InsertUpdateEvent {
                old: Some(old.clone()),
                new: new.clone(),
            };
            let _ = send_update.send(event);
        });

        table.on_insert(move |_ctx, row| {
            let event = InsertUpdateEvent {
                old: None,
                new: row.clone(),
            };
            let _ = send.send(event);
        });

        self
    }
}
