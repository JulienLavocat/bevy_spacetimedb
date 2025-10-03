<div align="center">

# bevy_spacetimedb

Use [SpacetimeDB](https://spacetimedb.com) in your Bevy application.

[![crates.io](https://img.shields.io/crates/v/bevy_spacetimedb)](https://crates.io/crates/bevy_spacetimedb)
[![docs.rs](https://docs.rs/bevy_spacetimedb/badge.svg)](https://docs.rs/bevy_spacetimedb)

</div>

## Highlights

This plugin will provide you with:

- A resource `StdbConnection` to call your reducers, subscribe to tables, etc.
- Connection lifecycle messages: `StdbConnectedMessage`, `StdbDisconnectedMessage`, `StdbConnectionErrorMessage` as Bevy's `MessageReader`
- All the table messages (row inserted/updated/deleted): `MessageReader`:
  - `ReadInsertMessage<T>`
  - `ReadUpdateMessage<T>`
  - `ReadInsertUpdateMessage<T>`
  - `ReadDeleteMessage<T>`

Check the example app in `/example_app` for a complete example of how to use the plugin.

## Bevy versions

This plugin is compatible with Bevy 0.15.x, 0.16.x, and 0.17.x; the latest version targets Bevy 0.17.x.

| bevy_spacetimedb version | Bevy version |
| ------------------------ | ------------ |
| <= 0.3.x                 | 0.15.x       |
| 0.4.x - 1.0.x            | 0.16.x       |
| >= 1.1.x                 | 0.17.x       |

## Usage

0. Add the plugin to your project: `cargo add bevy_spacetimedb`
1. Add the plugin to your Bevy application:

```rust
use bevy::{log::LogPlugin, prelude::*};
App::new()
        .add_plugins((MinimalPlugins, LogPlugin::default()))
        .add_plugins(
            StdbPlugin::default()
                .with_uri("http://localhost:3000")
                .with_module_name("chat")
                .with_run_fn(DbConnection::run_threaded)
                .add_table(RemoteTables::lobby)
                .add_table(RemoteTables::user)
                .add_partial_table(RemoteTables::player, TableMessages::no_update())
                .add_reducer::<CreateLobby>()
                .add_reducer::<SetName>(),
        )
```

3. Add a system handling connection messages
   You can also add systems for `StdbDisconnectedMessage` and `StdbConnectionErrorMessage`

```rust
fn on_connected(
    mut messages: ReadStdbConnectedMessage,
    stdb: Res<StdbConnection<DbConnection>>,
) {
    for _ in messages.read() {
        info!("Connected to SpacetimeDB");

        // Call any reducers
        stdb.reducers()
            .my_super_reducer("A suuuuppeeeeer argument for a suuuuppeeeeer reducer")
            .unwrap();

        // Subscribe to any tables
        stdb.subscription_builder()
            .on_applied(|_| info!("Subscription to players applied"))
            .on_error(|_, err| error!("Subscription to players failed for: {}", err))
            .subscribe("SELECT * FROM players");

        // Access your database cache (since it's not yet populated here this line might return 0)
        info!("Players count: {}", stdb.db().players().count());
    }
}
```

3. Add any systems that you need in order to handle the table messages you
   declared and do whatever you want:

```rust
fn on_player_inserted(mut messages: ReadInsertMessage<Player>, mut commands: Commands) {
    for message in messages.read() {
        commands.spawn(Player { id: message.row.id });
        info!("Player inserted: {:?} -> {:?}", message.row);
    }
}

fn on_player_updated(mut messages: ReadUpdateMessage<Player>) {
    for message in messages.read() {
        info!("Player updated: {:?} -> {:?}", message.old, message.new);
    }
}

fn on_player_insert_update(mut messages: ReadInsertUpdateMessage<Player>, q_players: Query<Entity, Player>) {
    for message in messages.read() {
        info!("Player deleted: {:?} -> {:?}", message.row);
        // Delete the player's entity
    }
}

fn on_player_deleted(mut messages: ReadDeleteMessage<Player>, q_players: Query<Entity, Player>) {
    for message in messages.read() {
        info!("Player deleted: {:?} -> {:?}", message.row);
        // Delete the player's entity
    }
}
```

## Tips and tricks

### Shorthand for `StdbConnection`

You can use `Res<StdbConnection<DbConnection>>` to get the resource but this is
quite verbose, you can create the following type alias for convenience:

```rust
pub type SpacetimeDB<'a> = Res<'a, StdbConnection<DbConnection>>;

fn my_system(stdb: SpacetimeDB) {
    // Use the `DbConnection` type alias
    stdb.reducers().my_reducer("some argument").unwrap();
}
```

## Special thanks

Special thanks to:

- @abos-gergo for the improvements toward reducing the boilerplate needed to use
  the plugin
- @PappAdam for the improvements toward reducing the boilerplate needed to use
  the plugin
