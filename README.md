<div align="center">

# bevy_spacetimedb_wasm

**WASM-only SpacetimeDB integration for Bevy using the TypeScript SDK bridge**

A fork of [bevy_spacetimedb](https://github.com/JulienLavocat/bevy_spacetimedb) that enables browser deployment by bridging to the SpacetimeDB TypeScript SDK.

</div>

## ⚠️ Important: WASM-Only

This crate **only works with `wasm32-unknown-unknown` target**. For native builds, use the original [bevy_spacetimedb](https://github.com/JulienLavocat/bevy_spacetimedb) crate.



## 🚀 Features

- **API Compatible**: Maintains similar API to `bevy_spacetimedb` for easy migration
- **Table Events**: Subscribe to insert/update/delete events via Bevy's event system
- **Connection Lifecycle**: Handle connect/disconnect/error events
- **Reducer Calls**: Fire-and-forget reducer invocation
- **TypeScript Bridge**: Leverages the official SpacetimeDB TypeScript SDK

## 📦 Quick Start

### 1. Add Dependencies

```toml
[dependencies]
bevy = "0.16"
bevy_spacetimedb_wasm = { path = "bevy_spacetimedb" }  # or version when published
serde = { version = "1.0", features = ["derive"] }
```

### 2. Define Tables & Reducers

```rust
use bevy_spacetimedb_wasm::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: u64,
    pub name: String,
    pub x: f32,
    pub y: f32,
}

impl TableRow for Player {
    const TABLE_NAME: &'static str = "players";
}

define_reducer!(SpawnPlayer(name: String, x: f32, y: f32));
define_reducer!(MovePlayer(id: u64, x: f32, y: f32));
```

### 3. Add the Plugin

```rust
use bevy::prelude::*;
use bevy_spacetimedb_wasm::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(
            StdbPlugin::default()
                .with_uri("ws://localhost:3000")
                .with_module_name("my_game")
                .add_table::<Player>()
        )
        .add_systems(Update, handle_players)
        .run();
}
```

### 4. Handle Events

```rust
fn handle_players(mut events: EventReader<InsertEvent<Player>>) {
    for event in events.read() {
        info!("New player: {:?}", event.row);
    }
}
```

### 5. Call Reducers

```rust
fn spawn_player(stdb: Res<StdbConnection>) {
    stdb.reducers()
        .call::<SpawnPlayer>(("Alice".to_string(), 0.0, 0.0))
        .expect("Failed to serialize args");
}
```

### 6. Build for WASM

```bash
# Build the TypeScript bridge
cd bevy_spacetimedb/js
npm install && npm run build

# Build your game
wasm-pack build --target web --out-dir www/pkg
```

### 7. Create HTML Page

```html
<!DOCTYPE html>
<html>
<body>
    <script type="module">
        // IMPORTANT: Load bridge BEFORE WASM!
        import { SpacetimeDBBridge } from './bevy_spacetimedb/js/dist/spacetimedb-bridge.js';
        window.__SPACETIMEDB_BRIDGE__ = new SpacetimeDBBridge();

        // Then load your WASM
        import init from './pkg/my_game.js';
        await init();
    </script>
</body>
</html>
```

## 📚 API Reference

### Table Events

```rust
// Subscribe to all events
.add_table::<Player>()

// Subscribe to specific events
.add_partial_table::<Message>(TableEvents::no_update())
.add_partial_table::<Stats>(TableEvents::insert_only())
```

Event types:
- `InsertEvent<T>` - Row inserted
- `UpdateEvent<T>` - Row updated (has `old` and `new`)
- `DeleteEvent<T>` - Row deleted
- `InsertUpdateEvent<T>` - Combined insert or update

### Connection Events

```rust
fn on_connected(mut events: EventReader<StdbConnectedEvent>) { /* ... */ }
fn on_disconnected(mut events: EventReader<StdbDisconnectedEvent>) { /* ... */ }
fn on_error(mut events: EventReader<StdbConnectionErrorEvent>) { /* ... */ }
```

### Calling Reducers

```rust
stdb.reducers().call::<MyReducer>((arg1, arg2, arg3))?;
```

## 🔧 Architecture

```
Bevy App (WASM) → bevy_spacetimedb_wasm → wasm-bindgen
    → JS Bridge → TypeScript SDK → WebSocket → SpacetimeDB Server
```

## ⚠️ Differences from bevy_spacetimedb

### Compatible
- ✅ Plugin configuration (`.with_uri()`, `.with_module_name()`)
- ✅ Table registration (`.add_table()`, `.add_partial_table()`)
- ✅ Table events (`InsertEvent`, `UpdateEvent`, `DeleteEvent`)
- ✅ Connection events
- ✅ Reducer calling

### Not Available
- ❌ `.with_run_fn()` - automatic
- ❌ `.with_compression()` - handled by TS SDK
- ❌ `.with_light_mode()` - handled by TS SDK
- ❌ `stdb.db()` - client cache access (use events instead)
- ❌ `stdb.subscribe()` - SQL subscriptions (use `.add_table()`)
- ❌ Reducer callback events

## 📄 License

Apache-2.0 (same as original bevy_spacetimedb)

## 🙏 Acknowledgments

This is a fork of [bevy_spacetimedb](https://github.com/JulienLavocat/bevy_spacetimedb) by Julien Lavocat, adapted for WASM using the SpacetimeDB TypeScript SDK.

Special thanks to:
- **Julien Lavocat** for the original bevy_spacetimedb implementation
- **@abos-gergo** and **@PappAdam** for improvements to the original plugin
- **The SpacetimeDB team** for the TypeScript SDK
- **The Bevy community**

## 🔗 Links

- [Original bevy_spacetimedb](https://github.com/JulienLavocat/bevy_spacetimedb)
- [SpacetimeDB](https://spacetimedb.com/)
- [SpacetimeDB TypeScript SDK](https://www.npmjs.com/package/@clockworklabs/spacetimedb-sdk)
- [Bevy Game Engine](https://bevyengine.org/)
