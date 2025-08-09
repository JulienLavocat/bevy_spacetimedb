use bevy::{log::LogPlugin, prelude::*};
use bevy_spacetimedb::{
    ReadDeleteEvent, ReadInsertEvent, ReadInsertUpdateEvent, ReadReducerEvent,
    ReadStdbConnectedEvent, ReadUpdateEvent, ReducerResultEvent, RegisterReducerEvent,
    StdbConnection, StdbPlugin, TableEvents,
};
use spacetimedb_sdk::ReducerEvent;
use stdb::{DbConnection, Reducer};

use crate::stdb::gs_register_reducer::gs_register;
use crate::stdb::gs_set_ready_reducer::gs_set_ready;
use crate::stdb::{
    GameServersTableAccess, PlanetsTableAccess, Player, PlayersTableAccess, RemoteModule,
    RemoteReducers, RemoteTables,
};
mod stdb;

#[derive(Debug, RegisterReducerEvent)]
#[allow(dead_code)]
pub struct GsRegister {
    event: ReducerEvent<Reducer>,
    ip: String,
    port: u16,
}

#[derive(Debug, RegisterReducerEvent)]
#[allow(dead_code)]
pub struct GsSetReady {
    event: ReducerEvent<Reducer>,
}

pub type SpacetimeDB<'a> = Res<'a, StdbConnection<DbConnection>>;

pub fn main() {
    App::new()
        .add_plugins((MinimalPlugins, LogPlugin::default()))
        .add_plugins(
            StdbPlugin::default()
                .with_uri("http://localhost:3000")
                .with_module_name("chat")
                .with_run_fn(DbConnection::run_threaded)
                .add_table(RemoteTables::planets)
                .add_table(RemoteTables::players)
                .add_table(RemoteTables::game_servers)
                .add_partial_table(RemoteTables::players, TableEvents::no_update()) // Some tables
                // do not have update events, especially those without primary keys.
                .add_reducer::<GsRegister>()
                .add_reducer::<GsSetReady>(),
        )
        .add_systems(Update, on_connected)
        .add_systems(Update, on_player_inserted)
        .add_systems(Update, on_player_updated)
        .add_systems(Update, on_player_deleted)
        .add_systems(Update, on_player_inserted_updated)
        .add_systems(Update, on_gs_register)
        .add_systems(Update, on_gs_set_ready)
        .run();
}

// SpacetimeDB is defined as an alias for the StdbConnection with DbConnection.
fn on_connected(mut events: ReadStdbConnectedEvent, stdb: SpacetimeDB) {
    for _ev in events.read() {
        info!("Connected to SpacetimeDB");

        stdb.subscription_builder()
            .on_applied(|_| info!("Subscription to lobby applied"))
            .on_error(|_, err| error!("Subscription to lobby failed for: {}", err))
            .subscribe("SELECT * FROM lobby");

        stdb.subscription_builder()
            .on_applied(|_| info!("Subscription to user applied"))
            .on_error(|_, err| error!("Subscription to user failed for: {}", err))
            .subscribe("SELECT * FROM user");
    }
}

fn on_player_inserted(mut events: ReadInsertEvent<Player>) {
    for event in events.read() {
        // Row below is just an example, does not actually compile.
        // commands.spawn(Player { id: event.row.id });
        info!("Player inserted: {:?}", event.row);
    }
}

fn on_player_updated(mut events: ReadUpdateEvent<Player>) {
    for event in events.read() {
        info!("Player updated: {:?} -> {:?}", event.old, event.new);
    }
}

fn on_player_deleted(mut events: ReadDeleteEvent<Player>) {
    for event in events.read() {
        info!("Player deleted: {:?}", event.row);
    }
}

fn on_player_inserted_updated(mut events: ReadInsertUpdateEvent<Player>) {
    for event in events.read() {
        info!(
            "Player insert/update event: old={:?}, new={:?}",
            event.old, event.new
        );
    }
}

fn on_gs_register(mut events: ReadReducerEvent<GsRegister>) {
    for event in events.read() {
        info!("Game server registered: {:?}", event.result);
    }
}

fn on_gs_set_ready(mut events: ReadReducerEvent<GsSetReady>) {
    for event in events.read() {
        info!("Game server set ready: {:?}", event.result);
    }
}
