<div align="center">

# bevy_spacetimedb

Use [SpacetimeDB](https://spacetimedb.com) in your Bevy application.

[![crates.io](https://img.shields.io/crates/v/bevy_spacetimedb)](https://crates.io/crates/bevy_spacetimedb)
[![docs.rs](https://docs.rs/bevy_spacetimedb/badge.svg)](https://docs.rs/bevy_spacetimedb)

</div>

## Highlights

This plugin will provide you with:

- A resource `StdbConnection` to call your reducers, subscribe to tables, etc.
- Connection lifecycle events: `StdbConnectedEvent`, `StdbDisconnectedEvent`, `StdbConnectionErrorEvent` as Bevy's `EventsReader`
- All the tables events (row inserted/updated/deleted): `InsertEvent\<MyRow>`, `UpdateEvent\<MyRow>`, `DeleteEvent\<MyRow>` as `EventsReader`

Check the example app in `/example_app` for a complete example of how to use the plugin.

## Bevy versions

This plugin is compatible with Bevy 0.15.x and 0.16.x, the latest version of the plugin is compatible with Bevy 0.16.x.

| bevy_spacetimedb version | Bevy version |
| ------------------------ | ------------ |
| <= 0.3.x                 | 0.15.x       |
| >= 0.4.x                 | 0.16.x       |

## Usage

0. Add to your crate: `cargo add bevy_spacetimedb`
1. Add the plugin to your bevy application:

```rust
App::new()
    .add_plugins(
        StdbPlugin::default()
            // Required, this method is used to configure your SpacetimeDB connection
            // you will also need to send the connected, disconnected and connect_error with_events back to the plugin
            // Don't forget to call run_threaded() on your connection
            .with_connection(|send_connected, send_disconnected, send_connect_error, _| {
                let conn = DbConnection::builder()
                    .with_module_name("<your module name>")
                    .with_uri("<your spacetimedb instance uri>")
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

                // Do what you want with the connection here

                // This is very important, otherwise your client will never connect and receive data
                conn.run_threaded();
                conn
            })
            /// Register the events you want to receive (example: players and enemies inserted, updated, deleted) and your reducers
            .with_events(|plugin, app, db| {
                tables!(
                    players,
                    enemies,
                    (player_without_update, no_update),
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


                let send_register_player = plugin.reducer_event::<RegisterPlayerEvent>(app);
                reducers.on_register_player(move |ctx, reducer_arg_1, reducer_arg_2| {
                    send_register_player
                        .send(ReducerResultEvent::new(RegisterPlayerEvent {
                            event: ctx.event.clone(),
                            // You can add any data you want here, even reducer arguments
                        }))
                        .unwrap();
                    });
            }),
    );
```

2. Add a system handling connection events
   You can also add systems for `StdbDisconnectedEvent` and `StdbConnectionErrorEvent`

```rust
fn on_connected(
    mut events: EventReader<StdbConnectedEvent>,
    stdb: Res<StdbConnection<DbConnection>>,
) {
    for _ in events.read() {
        info!("Connected to SpacetimeDB");

        // Call any reducers
        stdb.reducers()
            .my_super_reducer("A suuuuppeeeeer argument for a suuuuppeeeeer reducer")
            .unwrap();

        // Subscribe to any tables
        stdb.subscribe()
            .on_applied(|_| info!("Subscription to players applied"))
            .on_error(|_, err| error!("Subscription to players failed for: {}", err))
            .subscribe("SELECT * FROM players");

        // Access your database cache (since it's not yet populated here this line might return 0)
        info!("Players count: {}", stdb.db().players().count());
    }
}
```

3. Add any systems that you need in order to handle the table events you declared and do whatever you want:

```rust
fn on_player_inserted(mut events: EventReader<InsertEvent<Player>>, mut commands: Commands) {
    for event in events.read() {
        commands.spawn(Player { id: event.row.id });
        info!("Player inserted: {:?} -> {:?}", event.row);
    }
}

fn on_player_updated(mut events: EventReader<UpdateEvent<Player>>) {
    for event in events.read() {
        info!("Player updated: {:?} -> {:?}", event.old, event.new);
    }
}

fn on_player_deleted(mut events: EventReader<DeleteEvent<Player>>, q_players: Query<Entity, Player>) {
    for event in events.read() {
        info!("Player deleted: {:?} -> {:?}", event.row);
        // Delete the player's entity
    }
}
```
