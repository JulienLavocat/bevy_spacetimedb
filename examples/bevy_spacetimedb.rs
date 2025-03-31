use bevy::{log::LogPlugin, prelude::*};
use bevy_spacetimedb::{
    ReducerResultEvent, StdbConnectedEvent, StdbConnection, StdbConnectionErrorEvent,
    StdbDisconnectedEvent, StdbPlugin,
};
use spacetimedb_sdk::{ReducerEvent, Table};
use stdb::{DbConnection, PlayersTableAccess, Reducer, register_player};

mod stdb;

#[derive(Clone, Debug)]
pub struct RegisterPlayerEvent {
    pub event: ReducerEvent<Reducer>,
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
                    plugin
                        .on_insert(app, db.players())
                        .on_update(app, db.players())
                        .on_delete(app, db.players());

                    let send_register_player = plugin.reducer_event::<RegisterPlayerEvent>(app);
                    reducers.on_register_player(move |ctx, _, _| {
                        send_register_player
                            .send(ReducerResultEvent::new(RegisterPlayerEvent {
                                event: ctx.event.clone(),
                            }))
                            .unwrap();
                    });
                }),
        )
        .add_systems(Update, (on_connected, on_register_player))
        .run();
}

fn on_connected(
    mut events: EventReader<StdbConnectedEvent>,
    stdb: Res<StdbConnection<DbConnection>>,
) {
    for _ in events.read() {
        info!("Connected to SpacetimeDB");

        // Call any reducers
        stdb.reducers().register_player("".to_owned(), 1).unwrap();

        // Subscribe to any tables
        stdb.subscribe()
            .on_applied(|_| info!("Subscription to players applied"))
            .on_error(|_, err| error!("Subscription to players failed for: {}", err))
            .subscribe("SELECT * FROM players");

        // Access your database cache (since it's not yet populated here this line might return 0)
        info!("Players count: {}", stdb.db().players().count());
    }
}

fn on_register_player(mut events: EventReader<ReducerResultEvent<RegisterPlayerEvent>>) {
    for event in events.read() {
        info!("Registered player: {:?}", event);
    }
}
