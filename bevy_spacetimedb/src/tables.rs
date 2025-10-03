use std::{
    any::TypeId,
    sync::mpsc::{Sender, channel},
};

use bevy::app::App;
use spacetimedb_sdk::{__codegen as spacetime_codegen, Table, TableWithPrimaryKey};

use crate::AddMessageChannelAppExtensions;
// Imports are marked as unused but they are useful for linking types in docs.
// #[allow(unused_imports)]
use crate::{DeleteMessage, InsertMessage, InsertUpdateMessage, StdbPlugin, UpdateMessage};

/// Passed into [`StdbPlugin::add_table`] to determine which table messages to register.
#[derive(Debug, Default, Clone, Copy)]
pub struct TableMessages {
    /// Whether to register to a row insertion. Registers the [`InsertMessage`] message for the table.
    ///
    /// Use along with update to register the [`InsertUpdateMessage`] message as well.
    pub insert: bool,

    /// Whether to register to a row update. Registers the [`UpdateMessage`] message for the table.
    ///
    /// Use along with insert to register the [`InsertUpdateMessage`] message as well.
    pub update: bool,

    /// Whether to register to a row deletion. Registers the [`DeleteMessage`] message for the table.
    pub delete: bool,
}

impl TableMessages {
    /// Register all table messages
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

impl<
    C: spacetime_codegen::DbConnection<Module = M> + spacetimedb_sdk::DbContext,
    M: spacetime_codegen::SpacetimeModule<DbConnection = C>,
> StdbPlugin<C, M>
{
    /// Registers a table for the bevy application with all messages enabled.
    pub fn add_table<TRow, TTable, F>(self, accessor: F) -> Self
    where
        TRow: Send + Sync + Clone + 'static,
        TTable: Table<Row = TRow> + TableWithPrimaryKey<Row = TRow>,
        F: 'static + Send + Sync + Fn(&'static C::DbView) -> TTable,
    {
        self.add_partial_table(accessor, TableMessages::all())
    }

    ///Registers a table for the bevy application with the specified messages in the `messages` parameter.
    pub fn add_partial_table<TRow, TTable, F>(
        mut self,
        accessor: F,
        messages: TableMessages,
    ) -> Self
    where
        TRow: Send + Sync + Clone + 'static,
        TTable: Table<Row = TRow> + TableWithPrimaryKey<Row = TRow>,
        F: 'static + Send + Sync + Fn(&'static C::DbView) -> TTable,
    {
        // A closure that sets up messages for the table
        let register = move |plugin: &Self, app: &mut App, db: &'static C::DbView| {
            let table = accessor(db);
            if messages.insert {
                plugin.on_insert(app, &table);
            }
            if messages.delete {
                plugin.on_delete(app, &table);
            }
            if messages.update {
                plugin.on_update(app, &table);
            }
            if messages.update && messages.insert {
                plugin.on_insert_update(app, &table);
            }
        };

        // Store this table, and later when the plugin is built, call them on .
        self.table_registers.push(Box::new(register));

        self
    }

    /// Register a Bevy message of type InsertMessage<TRow> for the `on_insert` message on the provided table.
    fn on_insert<TRow>(&self, app: &mut App, table: &impl Table<Row = TRow>) -> &Self
    where
        TRow: Send + Sync + Clone + 'static,
    {
        let type_id = TypeId::of::<InsertMessage<TRow>>();

        let mut map = self.message_senders.lock().unwrap();

        let sender = map
            .entry(type_id)
            .or_insert_with(|| {
                let (send, recv) = channel::<InsertMessage<TRow>>();
                app.add_message_channel(recv);
                Box::new(send)
            })
            .downcast_ref::<Sender<InsertMessage<TRow>>>()
            .expect("Sender type mismatch")
            .clone();

        table.on_insert(move |_ctx, row| {
            let message = InsertMessage { row: row.clone() };
            let _ = sender.send(message);
        });

        self
    }

    /// Register a Bevy message of type DeleteMessage<TRow> for the `on_delete` message on the provided table.
    fn on_delete<TRow>(&self, app: &mut App, table: &impl Table<Row = TRow>) -> &Self
    where
        TRow: Send + Sync + Clone + 'static,
    {
        let type_id = TypeId::of::<DeleteMessage<TRow>>();

        let mut map = self.message_senders.lock().unwrap();
        let sender = map
            .entry(type_id)
            .or_insert_with(|| {
                let (send, recv) = channel::<DeleteMessage<TRow>>();
                app.add_message_channel(recv);
                Box::new(send)
            })
            .downcast_ref::<Sender<DeleteMessage<TRow>>>()
            .expect("Sender type mismatch")
            .clone();

        table.on_delete(move |_ctx, row| {
            let message = DeleteMessage { row: row.clone() };
            let _ = sender.send(message);
        });

        self
    }

    /// Register a Bevy message of type UpdateMessage<TRow> for the `on_update` message on the provided table.
    fn on_update<TRow, TTable>(&self, app: &mut App, table: &TTable) -> &Self
    where
        TRow: Send + Sync + Clone + 'static,
        TTable: Table<Row = TRow> + TableWithPrimaryKey<Row = TRow>,
    {
        let type_id = TypeId::of::<UpdateMessage<TRow>>();

        let mut map = self.message_senders.lock().unwrap();
        let sender = map
            .entry(type_id)
            .or_insert_with(|| {
                let (send, recv) = channel::<UpdateMessage<TRow>>();
                app.add_message_channel(recv);
                Box::new(send)
            })
            .downcast_ref::<Sender<UpdateMessage<TRow>>>()
            .expect("Sender type mismatch")
            .clone();

        table.on_update(move |_ctx, old, new| {
            let message = UpdateMessage {
                old: old.clone(),
                new: new.clone(),
            };
            let _ = sender.send(message);
        });

        self
    }

    /// Register a Bevy message of type InsertUpdateMessage<TRow> for the `on_insert` and `on_update` messages on the provided table.
    fn on_insert_update<TRow, TTable>(&self, app: &mut App, table: &TTable) -> &Self
    where
        TRow: Send + Sync + Clone + 'static,
        TTable: Table<Row = TRow> + TableWithPrimaryKey<Row = TRow>,
    {
        let type_id = TypeId::of::<InsertUpdateMessage<TRow>>();

        let mut map = self.message_senders.lock().unwrap();
        let send = map
            .entry(type_id)
            .or_insert_with(|| {
                let (send, recv) = channel::<InsertUpdateMessage<TRow>>();
                app.add_message_channel(recv);
                Box::new(send)
            })
            .downcast_ref::<Sender<InsertUpdateMessage<TRow>>>()
            .expect("Sender type mismatch")
            .clone();

        let send_update = send.clone();
        table.on_update(move |_ctx, old, new| {
            let message = InsertUpdateMessage {
                old: Some(old.clone()),
                new: new.clone(),
            };
            let _ = send_update.send(message);
        });

        table.on_insert(move |_ctx, row| {
            let message = InsertUpdateMessage {
                old: None,
                new: row.clone(),
            };
            let _ = send.send(message);
        });

        self
    }
}
