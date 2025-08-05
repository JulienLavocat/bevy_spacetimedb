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
        .add_plugins((MinimalPlugins, LogPlugin::default()))
        .add_plugins(
            StdbPlugin::default()
                .with_uri("http://localhost:3000")
                .with_module_name("chat")
                .with_run_fn(DbConnection::run_threaded)
                .add_table(RemoteTables::lobby, TableEvents::all())
                .add_table(RemoteTables::user, TableEvents::all())
                .add_reducer::<CreateLobby>()
                .add_reducer::<SetName>(),
        )
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
