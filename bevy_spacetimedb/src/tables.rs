use std::{
    any::{Any, TypeId},
    hash::Hash,
    sync::mpsc::{Sender, channel},
};

use bevy::{
    app::{App, Update},
    platform::collections::HashMap,
    prelude::{Message, MessageReader, MessageWriter},
};
use spacetimedb_sdk::{__codegen as spacetime_codegen, Table, TableWithPrimaryKey};

use crate::AddMessageChannelAppExtensions;
// Imports are marked as unused but they are useful for linking types in docs.
// #[allow(unused_imports)]
use crate::{DeleteMessage, InsertMessage, InsertUpdateMessage, StdbPlugin, UpdateMessage};

/// Ensure that a message channel exists for the given type.
fn ensure_message_channel<T: Message>(
    app: &mut App,
    map: &mut HashMap<TypeId, Box<dyn Any + Send + Sync>>,
) {
    let type_id = TypeId::of::<T>();
    map.entry(type_id).or_insert_with(|| {
        let (send, recv) = channel::<T>();
        app.add_message_channel(recv);
        Box::new(send)
    });
}

/// Internal message used by `add_view_with_pk` to buffer view inserts/deletes so we can
/// coalesce them into `InsertMessage<TRow>`, `UpdateMessage<TRow>`, and `DeleteMessage<TRow>`
/// without causing Bevy message access conflicts.
#[derive(bevy::prelude::Message)]
enum ViewPkBufferedMessage<TRow> {
    Insert(TRow),
    Delete(TRow),
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

/// Per-frame reconciliation state for `add_view_with_pk`.
///
/// Stored as a Bevy `Local`, so it is reset per system instance and persists only across frames.
struct ViewPkReconcileState<TRow, TPk> {
    /// Rows that were deleted from the view this frame, keyed by primary key.
    deleted: HashMap<TPk, TRow>,
    /// Rows that were inserted into the view this frame, keyed by primary key.
    inserted: HashMap<TPk, TRow>,
}

impl<TRow, TPk> Default for ViewPkReconcileState<TRow, TPk> {
    fn default() -> Self {
        Self {
            deleted: HashMap::default(),
            inserted: HashMap::default(),
        }
    }
}

/// System that reconciles buffered view membership changes into `InsertMessage<TRow>`,
/// `UpdateMessage<TRow>`, and `DeleteMessage<TRow>`.
///
/// We intentionally buffer via `ViewPkBufferedMessage` so the system does **not** read and write
/// `Messages<InsertMessage<TRow>>` in the same system, which would trigger Bevy's `B0002` conflict.
fn reconcile_view_pk_frame<TRow, TPk>(
    mut state: bevy::prelude::Local<ViewPkReconcileState<TRow, TPk>>,
    mut buffered: MessageReader<ViewPkBufferedMessage<TRow>>,
    mut out_inserts: MessageWriter<InsertMessage<TRow>>,
    mut out_updates: MessageWriter<UpdateMessage<TRow>>,
    mut out_deletes: MessageWriter<DeleteMessage<TRow>>,
    pk: bevy::prelude::Res<ViewPkFn<TRow, TPk>>,
) where
    TRow: Send + Sync + Clone + 'static,
    TPk: Send + Sync + Clone + Eq + Hash + 'static,
{
    // Collect all buffered view changes for this frame, keyed by pk.
    for msg in buffered.read() {
        match msg {
            ViewPkBufferedMessage::Insert(row) => {
                let row = row.clone();
                let k = (pk.0)(&row);
                state.inserted.insert(k, row);
            }
            ViewPkBufferedMessage::Delete(row) => {
                let row = row.clone();
                let k = (pk.0)(&row);
                state.deleted.insert(k, row);
            }
        }
    }

    // Move inserted rows out to avoid double-borrowing `state` mutably while coalescing.
    let inserted = std::mem::take(&mut state.inserted);

    // Coalesce delete+insert of same pk into UpdateMessage, otherwise forward as Insert/Delete.
    for (k, new_row) in inserted {
        if let Some(old_row) = state.deleted.remove(&k) {
            out_updates.write(UpdateMessage {
                old: old_row,
                new: new_row,
            });
        } else {
            out_inserts.write(InsertMessage { row: new_row });
        }
    }

    // Any remaining deletions did not have a matching insertion this frame.
    for (_k, old_row) in state.deleted.drain() {
        out_deletes.write(DeleteMessage { row: old_row });
    }
}

#[derive(bevy::prelude::Resource)]
struct ViewPkFn<TRow, TPk>(Box<dyn Fn(&TRow) -> TPk + Send + Sync + 'static>);

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
        self.table_registers.lock().unwrap().push(Box::new(register));

        self
    }

    /// Registers a table without primary key for the bevy application with all messages enabled.
    pub fn add_table_without_pk<TRow, TTable, F>(self, accessor: F) -> Self
    where
        TRow: Send + Sync + Clone + 'static,
        TTable: Table<Row = TRow>,
        F: 'static + Send + Sync + Fn(&'static C::DbView) -> TTable,
    {
        self.add_partial_table_without_pk(accessor, TableMessagesWithoutPrimaryKey::all())
    }

    /// Registers a *view-like* table (no `TableWithPrimaryKey`) whose row type *does* have a stable primary key,
    /// and reconciles per-frame delete+insert pairs for the same key into [`UpdateMessage<TRow>`].
    ///
    /// ## Why this exists
    /// SpacetimeDB codegen models `#[view(...)] fn ... -> Vec<T>` as a "table-like" handle that streams
    /// individual rows `T` entering/leaving the view result set via `on_insert` / `on_delete`.
    /// These generated view handles typically do **not** implement [`TableWithPrimaryKey`], even when `T`
    /// contains a stable primary key (e.g. `Actor { id, ... }`), so consumers may observe `Delete`+`Insert`
    /// for what is logically an update.
    ///
    /// `add_view_with_pk` fixes the Bevy-facing ergonomics by coalescing same-frame `Delete(pk)` + `Insert(pk)`
    /// into a single [`UpdateMessage<TRow>`].
    ///
    /// The reconciliation happens in the same Bevy frame:
    ///
    /// - `Delete(pk)` + `Insert(pk)` in the same frame => `Update(old, new)`
    /// - `Insert(pk)` only => `Insert(row)`
    /// - `Delete(pk)` only => `Delete(row)`
    ///
    /// ## Limitations
    /// - Only one `add_view_with_pk` registration is supported per `(TRow, TPk)` pair.
    ///   Registering multiple views with the same `(TRow, TPk)` is not supported (the first registration wins).
    ///
    /// # Requirements
    /// - The view must produce at most one row per primary key at a time (uniqueness by `pk_fn`).
    pub fn add_view_with_pk<TRow, TPk, TView, FAcc, FPk>(
        mut self,
        accessor: FAcc,
        pk_fn: FPk,
    ) -> Self
    where
        TRow: Send + Sync + Clone + 'static,
        TPk: Send + Sync + Clone + Eq + Hash + 'static,
        TView: Table<Row = TRow>,
        FAcc: 'static + Send + Sync + Fn(&'static C::DbView) -> TView,
        FPk: 'static + Send + Sync + Fn(&TRow) -> TPk,
    {
        // `table_registers` stores `Fn`, not `FnOnce`, so we must not move `pk_fn` into the closure directly.
        // Wrap it in an `Arc` so the closure can clone it each call.
        let pk_fn = std::sync::Arc::new(pk_fn);

        let register = move |plugin: &Self, app: &mut App, db: &'static C::DbView| {
            let view = accessor(db);

            // Create buffered message channel (once per TRow).
            let buffered_type_id = TypeId::of::<ViewPkBufferedMessage<TRow>>();
            let buffered_sender: Sender<ViewPkBufferedMessage<TRow>> = {
                let mut map = plugin.message_senders.lock().unwrap();
                map.entry(buffered_type_id)
                    .or_insert_with(|| {
                        let (send, recv) = channel::<ViewPkBufferedMessage<TRow>>();
                        app.add_message_channel(recv);
                        Box::new(send)
                    })
                    .downcast_ref::<Sender<ViewPkBufferedMessage<TRow>>>()
                    .expect("Sender type mismatch")
                    .clone()
            };

            // Enforce exactly one `add_view_with_pk` registration per (TRow, TPk) pair.
            let mut installed = plugin.view_pk_reconcilers.lock().unwrap();
            if !installed.insert(TypeId::of::<(TRow, TPk)>()) {
                panic!(
                    "add_view_with_pk was registered more than once for the same (TRow, TPk) pair. \
Only one registration is supported per unique (TRow, TPk)."
                );
            }

            let send_insert = buffered_sender.clone();
            view.on_insert(move |_ctx, row| {
                let _ = send_insert.send(ViewPkBufferedMessage::Insert(row.clone()));
            });

            let send_delete = buffered_sender;
            view.on_delete(move |_ctx, row| {
                let _ = send_delete.send(ViewPkBufferedMessage::Delete(row.clone()));
            });

            // Insert pk fn resource (clone `Arc` so we don't consume it).
            let pk_fn = pk_fn.clone();
            app.insert_resource(ViewPkFn::<TRow, TPk>(Box::new(move |row: &TRow| {
                (pk_fn)(row)
            })));

            // Ensure output message channels exist by touching senders so writers can be used.
            let mut map = plugin.message_senders.lock().unwrap();
            ensure_message_channel::<InsertMessage<TRow>>(app, &mut map);
            ensure_message_channel::<UpdateMessage<TRow>>(app, &mut map);
            ensure_message_channel::<DeleteMessage<TRow>>(app, &mut map);

            app.add_systems(Update, reconcile_view_pk_frame::<TRow, TPk>);
        };

        self.table_registers.push(Box::new(register));
        self
    }

    ///Registers a table without primary key for the bevy application with the specified messages in the `messages` parameter.
    pub fn add_partial_table_without_pk<TRow, TTable, F>(
        self,
        accessor: F,
        messages: TableMessagesWithoutPrimaryKey,
    ) -> Self
    where
        TRow: Send + Sync + Clone + 'static,
        TTable: Table<Row = TRow>,
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
        };
        // Store this table, and later when the plugin is built, call them on .
        self.table_registers.lock().unwrap().push(Box::new(register));

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
