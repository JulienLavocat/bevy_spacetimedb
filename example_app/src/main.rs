use bevy::{log::LogPlugin, prelude::*};
use bevy_spacetimedb::{
    ReadInsertEvent, ReadReducerEvent, ReducerResultEvent, StdbConnectedEvent, StdbConnection,
    StdbConnectionErrorEvent, StdbDisconnectedEvent, StdbPlugin, register_reducers, tables,
};
use spacetimedb_sdk::{ReducerEvent, Table};
use stdb::{
    DbConnection, GameServersTableAccess, Planet, PlanetsTableAccess, Player, PlayersTableAccess,
    Reducer, StarSystemsTableAccess, gs_register, player_register,
};

mod stdb;

#[derive(Clone, Debug)]
pub struct RegisterPlayerEvent {
    pub event: ReducerEvent<Reducer>,
    pub id: u64,
}

#[derive(Clone, Debug)]
pub struct GsRegisterEvent {
    pub event: ReducerEvent<Reducer>,
    pub ip: String,
    pub port: u16,
}

pub fn main() {
    App::new()
        .add_plugins((MinimalPlugins, LogPlugin::default()))
        .add_plugins(
            StdbPlugin::default()
                .with_connection(|send_connected, send_disconnected, send_connect_error, _| {
                    let conn = DbConnection::builder()
                        .with_module_name("stellarwar")
                        .with_uri("https://stdb.jlavocat.eu")
                        .on_connect_error(move |_ctx, err| {
                            send_connect_error
                                .send(StdbConnectionErrorEvent { err })
                                .unwrap();
                        })
                        .on_disconnect(move |_ctx, err| {
                            send_disconnected
                                .send(StdbDisconnectedEvent { err })
                                .unwrap();
                        })
                        .on_connect(move |_ctx, _id, _c| {
                            send_connected.send(StdbConnectedEvent {}).unwrap();
                        })
                        .build()
                        .expect("SpacetimeDB connection failed");

                    conn.run_threaded();
                    conn
                })
                .with_events(|plugin, app, db, reducers| {
                    tables!(
                        players,
                        game_servers,
                        (star_systems, no_update),
                        (planets, no_update)
                    );

                    register_reducers!(
                        on_player_register(ctx, id) => RegisterPlayerEvent {
                            event: ctx.event.clone(),
                            id: *id
                        },
                        on_gs_register(ctx, ip, port) => GsRegisterEvent {
                            event: ctx.event.clone(),
                            ip: ip.clone(),
                            port: *port
                        }
                    );
                }),
        )
        .add_systems(
            Update,
            (on_connected, on_register_player, on_gs_register, on_player),
        )
        .run();
}

fn on_connected(
    mut events: EventReader<StdbConnectedEvent>,
    stdb: Res<StdbConnection<DbConnection>>,
) {
    for _ in events.read() {
        info!("Connected to SpacetimeDB");

        // Call any reducers
        stdb.reducers().player_register(1).unwrap();

        // Subscribe to any tables
        stdb.subscribe()
            .on_applied(|_| info!("Subscription to players applied"))
            .on_error(|_, err| error!("Subscription to players failed for: {}", err))
            .subscribe("SELECT * FROM players");

        // Access your database cache (since it's not yet populated here this line might return 0)
        info!("Players count: {}", stdb.db().players().count());
    }
}

fn on_register_player(mut events: ReadReducerEvent<RegisterPlayerEvent>) {
    for event in events.read() {
        info!("Registered player: {:?}", event);
    }
}

fn on_gs_register(mut events: ReadReducerEvent<GsRegisterEvent>) {
    for event in events.read() {
        info!("Registered game server: {:?}", event);
    }
}

fn on_player(mut events: ReadInsertEvent<Player>) {
    for event in events.read() {
        info!("Player inserted: {:?}", event.row);
    }
}
