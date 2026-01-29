use std::{
    any::TypeId,
    sync::mpsc::{Sender, channel},
};

use crate::AddMessageChannelAppExtensions;
use bevy::app::App;
use spacetimedb_sdk::{__codegen as spacetime_codegen, Event, Table, TableWithPrimaryKey};
// Imports are marked as unused but they are useful for linking types in docs.
// #[allow(unused_imports)]
use crate::{DeleteMessage, InsertMessage, InsertUpdateMessage, StdbPlugin, UpdateMessage};

pub trait TableMessage where Self:Sized {
    type Row : Send + Sync + Clone + 'static;
    type Reducer: Send + Sync + Clone + 'static;
}

pub trait RegisterableTable<C, M>
where
    C: spacetime_codegen::DbConnection<Module = M> + spacetimedb_sdk::DbContext,
    M: spacetime_codegen::SpacetimeModule<DbConnection = C>,
    Self: Sized,
{
    type Row: Send + Sync + Clone + 'static;
    type Reducer: Send + Sync + Clone + 'static;
    type Table: Table<Row = Self::Row> + TableWithPrimaryKey<Row = Self::Row>;
    type Message: TableMessage<Row = Self::Row, Reducer = Self::Reducer>;

    fn table_accessor(db_context: &'static C::DbView) -> Self::Table;
    fn context_event_accessor(ctx: &<Self::Table as Table>::EventContext) -> Event<Self::Reducer>;
}

pub trait RegisterableTableWithoutPk<C, M>
where
    C: spacetime_codegen::DbConnection<Module = M> + spacetimedb_sdk::DbContext,
    M: spacetime_codegen::SpacetimeModule<DbConnection = C>,
    Self: Sized,
{
    type Row: Send + Sync + Clone + 'static;
    type Reducer: Send + Sync + Clone + 'static;
    type Table: Table<Row = Self::Row>;
    type Message: TableMessage<Row = Self::Row, Reducer = Self::Reducer>;

    fn table_accessor(db_context: &'static C::DbView) -> Self::Table;
    fn context_event_accessor(ctx: &<Self::Table as Table>::EventContext) -> Event<Self::Reducer>;
}

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

/// Passed into [`StdbPlugin::add_table_without_pk`] to determine which table messages to register.
/// Specifically for tables with no Primary keys
#[derive(Debug, Default, Clone, Copy)]
pub struct TableMessagesWithoutPrimaryKey {
    /// Same as [`TableMessages::insert`]
    pub insert: bool,
    /// Same as [`TableMessages::delete`]
    pub delete: bool,
}

impl TableMessagesWithoutPrimaryKey {
    /// Register all available table messages
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
    /// Registers a table for the bevy application with all messages enabled.
    pub fn add_table<T: RegisterableTable<C, M> + Send + Sync + 'static>(self) -> Self {
        self.add_partial_table::<T>(TableMessages::all())
    }

    ///Registers a table for the bevy application with the specified messages in the `messages` parameter.
    pub fn add_partial_table<T: RegisterableTable<C, M> + Send + Sync + 'static>(
        mut self,
        messages: TableMessages,
    ) -> Self {
        // A closure that sets up messages for the table
        let register = move |plugin: &Self, app: &mut App, db: &'static C::DbView| {
            if messages.insert {
                plugin.on_insert::<T>(app, db);
            }
            if messages.delete {
                plugin.on_delete::<T>(app, db);
            }
            if messages.update {
                plugin.on_update::<T>(app, db);
            }
            if messages.update && messages.insert {
                plugin.on_insert_update::<T>(app, db);
            }
        };

        // Store this table, and later when the plugin is built, call them on .
        self.table_registers
            .lock()
            .unwrap()
            .push(Box::new(register));

        self
    }

    /// Registers a table without primary key for the bevy application with all messages enabled.
    pub fn add_table_without_pk<T: RegisterableTableWithoutPk<C, M> + Send + Sync + 'static>(
        self,
    ) -> Self {
        self.add_partial_table_without_pk::<T>(TableMessagesWithoutPrimaryKey::all())
    }

    ///Registers a table without primary key for the bevy application with the specified messages in the `messages` parameter.
    pub fn add_partial_table_without_pk<T: RegisterableTableWithoutPk<C, M> + Send + Sync + 'static>(
        mut self,
        messages: TableMessagesWithoutPrimaryKey,
    ) -> Self {
        // A closure that sets up messages for the table
        let register = move |plugin: &Self, app: &mut App, db: &'static C::DbView| {
            if messages.insert {
                plugin.on_insert_without_pk::<T>(app, db);
            }
            if messages.delete {
                plugin.on_delete_without_pk::<T>(app, db);
            }
        };
        // Store this table, and later when the plugin is built, call them on .
        self.table_registers
            .lock()
            .unwrap()
            .push(Box::new(register));

        self
    }

    /// Register a Bevy message of type InsertMessage<TRow> for the `on_insert` message on the provided table.
    fn on_insert<T: RegisterableTable<C, M> + Send + Sync + 'static>(
        &self,
        app: &mut App,
        db: &'static C::DbView,
    ) -> &Self {
        let type_id = TypeId::of::<InsertMessage<T::Message>>();

        let mut map = self.message_senders.lock().unwrap();

        let sender = map
            .entry(type_id)
            .or_insert_with(|| {
                let (send, recv) = channel::<InsertMessage<T::Message>>();
                app.add_message_channel(recv);
                Box::new(send)
            })
            .downcast_ref::<Sender<InsertMessage<T::Message>>>()
            .expect("Sender type mismatch")
            .clone();

        T::table_accessor(db).on_insert(move |_ctx, row| {
            let message = InsertMessage {
                event: T::context_event_accessor(_ctx),
                row: row.clone(),
            };
            let _ = sender.send(message);
        });

        self
    }

    /// Register a Bevy message of type DeleteMessage<TRow> for the `on_delete` message on the provided table.
    fn on_delete<T: RegisterableTable<C, M> + Send + Sync + 'static>(
        &self,
        app: &mut App,
        db: &'static C::DbView,
    ) -> &Self {
        let type_id = TypeId::of::<DeleteMessage<T::Message>>();

        let mut map = self.message_senders.lock().unwrap();
        let sender = map
            .entry(type_id)
            .or_insert_with(|| {
                let (send, recv) = channel::<DeleteMessage<T::Message>>();
                app.add_message_channel(recv);
                Box::new(send)
            })
            .downcast_ref::<Sender<DeleteMessage<T::Message>>>()
            .expect("Sender type mismatch")
            .clone();

        T::table_accessor(db).on_delete(move |_ctx, row| {
            let message = DeleteMessage {
                event: T::context_event_accessor(_ctx),
                row: row.clone(),
            };
            let _ = sender.send(message);
        });

        self
    }

    /// Register a Bevy message of type UpdateMessage<TRow> for the `on_update` message on the provided table.
    fn on_update<T: RegisterableTable<C, M> + Send + Sync + 'static>(
        &self,
        app: &mut App,
        db: &'static C::DbView,
    ) -> &Self {
        let type_id = TypeId::of::<UpdateMessage<T::Message>>();

        let mut map = self.message_senders.lock().unwrap();
        let sender = map
            .entry(type_id)
            .or_insert_with(|| {
                let (send, recv) = channel::<UpdateMessage<T::Message>>();
                app.add_message_channel(recv);
                Box::new(send)
            })
            .downcast_ref::<Sender<UpdateMessage<T::Message>>>()
            .expect("Sender type mismatch")
            .clone();

        T::table_accessor(db).on_update(move |_ctx, old, new| {
            let message = UpdateMessage {
                event: T::context_event_accessor(_ctx),
                old: old.clone(),
                new: new.clone(),
            };
            let _ = sender.send(message);
        });

        self
    }

    /// Register a Bevy message of type InsertUpdateMessage<TRow> for the `on_insert` and `on_update` messages on the provided table.
    fn on_insert_update<T: RegisterableTable<C, M> + Send + Sync + 'static>(
        &self,
        app: &mut App,
        db: &'static C::DbView,
    ) -> &Self {
        let type_id = TypeId::of::<InsertUpdateMessage<T::Message>>();

        let mut map = self.message_senders.lock().unwrap();
        let send = map
            .entry(type_id)
            .or_insert_with(|| {
                let (send, recv) = channel::<InsertUpdateMessage<T::Message>>();
                app.add_message_channel(recv);
                Box::new(send)
            })
            .downcast_ref::<Sender<InsertUpdateMessage<T::Message>>>()
            .expect("Sender type mismatch")
            .clone();

        let send_update = send.clone();
        T::table_accessor(db).on_update(move |_ctx, old, new| {
            let message = InsertUpdateMessage {
                event: T::context_event_accessor(_ctx),
                old: Some(old.clone()),
                new: new.clone(),
            };
            let _ = send_update.send(message);
        });

        T::table_accessor(db).on_insert(move |_ctx, row| {
            let message = InsertUpdateMessage {
                event: T::context_event_accessor(_ctx),
                old: None,
                new: row.clone(),
            };
            let _ = send.send(message);
        });

        self
    }

    /// Register a Bevy message of type InsertMessage<TRow> for the `on_insert` message on a table without primary key.
    fn on_insert_without_pk<T: RegisterableTableWithoutPk<C, M> + Send + Sync + 'static>(
        &self,
        app: &mut App,
        db: &'static C::DbView,
    ) -> &Self {
        let type_id = TypeId::of::<InsertMessage<T::Message>>();

        let mut map = self.message_senders.lock().unwrap();

        let sender = map
            .entry(type_id)
            .or_insert_with(|| {
                let (send, recv) = channel::<InsertMessage<T::Message>>();
                app.add_message_channel(recv);
                Box::new(send)
            })
            .downcast_ref::<Sender<InsertMessage<T::Message>>>()
            .expect("Sender type mismatch")
            .clone();

        T::table_accessor(db).on_insert(move |_ctx, row| {
            let message = InsertMessage {
                event: T::context_event_accessor(_ctx),
                row: row.clone(),
            };
            let _ = sender.send(message);
        });

        self
    }

    /// Register a Bevy message of type DeleteMessage<TRow> for the `on_delete` message on a table without primary key.
    fn on_delete_without_pk<T: RegisterableTableWithoutPk<C, M> + Send + Sync + 'static>(
        &self,
        app: &mut App,
        db: &'static C::DbView,
    ) -> &Self {
        let type_id = TypeId::of::<DeleteMessage<T::Message>>();

        let mut map = self.message_senders.lock().unwrap();
        let sender = map
            .entry(type_id)
            .or_insert_with(|| {
                let (send, recv) = channel::<DeleteMessage<T::Message>>();
                app.add_message_channel(recv);
                Box::new(send)
            })
            .downcast_ref::<Sender<DeleteMessage<T::Message>>>()
            .expect("Sender type mismatch")
            .clone();

        T::table_accessor(db).on_delete(move |_ctx, row| {
            let message = DeleteMessage {
                event: T::context_event_accessor(_ctx),
                row: row.clone(),
            };
            let _ = sender.send(message);
        });

        self
    }
}
