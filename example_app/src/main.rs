use bevy::{log::LogPlugin, prelude::*};
use bevy_spacetimedb::{
    DeleteEvent, InsertEvent, ReducerResultEvent, RegisterReducerEvent, StdbConnectedEvent,
    StdbConnection, StdbPlugin, TableEvents, UpdateEvent,
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

pub fn main() {
    App::new()
        .add_plugins((MinimalPlugins, LogPlugin::default()))
        .add_plugins(
            StdbPlugin::default()
                .with_uri("http://localhost:3000")
                .with_module_name("chat")
                .with_run_fn(DbConnection::run_threaded)
                .add_table(RemoteTables::planets, TableEvents::all())
                .add_table(RemoteTables::players, TableEvents::all())
                .add_table(RemoteTables::game_servers, TableEvents::all())
                .add_reducer::<GsRegister>()
                .add_reducer::<GsSetReady>(),
        )
        .add_systems(Update, on_connected)
        .add_systems(Update, on_player_inserted)
        .add_systems(Update, on_player_updated)
        .add_systems(Update, on_player_deleted)
        .run();
}

fn on_connected(
    mut events: EventReader<StdbConnectedEvent>,
    stdb: Res<StdbConnection<DbConnection>>,
) {
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

fn on_player_inserted(mut events: EventReader<InsertEvent<Player>>) {
    for event in events.read() {
        // Row below is just an example, does not actually compile.
        // commands.spawn(Player { id: event.row.id });
        info!("Player inserted: {:?}", event.row);
    }
}

fn on_player_updated(mut events: EventReader<UpdateEvent<Player>>) {
    for event in events.read() {
        info!("Player updated: {:?} -> {:?}", event.old, event.new);
    }
}

fn on_player_deleted(mut events: EventReader<DeleteEvent<Player>>) {
    for event in events.read() {
        info!("Player deleted: {:?}", event.row);
    }
}

#[derive(Debug, RegisterReducerEvent)]
pub struct GsRegister {
    event: ReducerEvent<Reducer>,
    ip: String,
    port: u16,
}

#[derive(Debug, RegisterReducerEvent)]
pub struct GsSetReady {
    event: ReducerEvent<Reducer>,
}
