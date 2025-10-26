# Setting Up bevy_spacetimedb_wasm in Squatch

This tutorial will guide you through integrating SpacetimeDB into your Squatch game using the WASM bridge.

## Prerequisites

- Rust with `wasm32-unknown-unknown` target installed
- Node.js and npm (for building the TypeScript bridge)
- wasm-pack (`cargo install wasm-pack`)
- A running SpacetimeDB server (or access to one)

## Step 1: Add the Dependency

Add `bevy_spacetimedb_wasm` to your Squatch project's `Cargo.toml`:

```toml
[dependencies]
bevy = "0.16"
bevy_spacetimedb_wasm = { path = "../bevy_spacetimedb_wasm/bevy_spacetimedb" }
serde = { version = "1.0", features = ["derive"] }
```

## Step 2: Build the TypeScript Bridge

The TypeScript bridge must be built before you can use it:

```bash
# Navigate to the bridge directory
cd ../bevy_spacetimedb_wasm/bevy_spacetimedb/js

# Install dependencies
npm install

# Build the bridge (compiles TypeScript to JavaScript)
npm run build

# You should now have a dist/ directory with the compiled bridge
```

## Step 3: Define Your SpacetimeDB Schema

Create a module in your Squatch project for SpacetimeDB types. For example, `src/stdb_types.rs`:

```rust
use bevy_spacetimedb_wasm::*;
use serde::{Deserialize, Serialize};

// Define your table types
// These should match your SpacetimeDB module schema

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SquatchPlayer {
    pub id: u64,
    pub name: String,
    pub position_x: f32,
    pub position_y: f32,
    pub health: i32,
}

impl TableRow for SquatchPlayer {
    const TABLE_NAME: &'static str = "players";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub id: u64,
    pub match_id: String,
    pub state: String,  // "waiting", "active", "finished"
    pub player_count: u32,
}

impl TableRow for GameState {
    const TABLE_NAME: &'static str = "game_states";
}

// Define your reducers
// These should match your SpacetimeDB module reducers

define_reducer!(JoinGame(player_name: String));
define_reducer!(UpdatePlayerPosition(player_id: u64, x: f32, y: f32));
define_reducer!(PlayerAction(player_id: u64, action: String));
define_reducer!(LeaveGame(player_id: u64));
```

## Step 4: Add the Plugin to Squatch

In your main Squatch app setup (likely `src/main.rs` or similar):

```rust
use bevy::prelude::*;
use bevy_spacetimedb_wasm::*;

mod stdb_types;
use stdb_types::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)

        // Add the SpacetimeDB plugin
        .add_plugins(
            StdbPlugin::default()
                // Configure your SpacetimeDB server
                .with_uri("ws://localhost:3000")  // Update with your server URI
                .with_module_name("squatch")      // Your SpacetimeDB module name

                // Register your tables
                .add_table::<SquatchPlayer>()
                .add_table::<GameState>()
        )

        // Add your game systems
        .add_systems(Startup, setup_game)
        .add_systems(Update, (
            handle_connection,
            handle_player_events,
            handle_game_state_events,
            player_input_system,
        ))
        .run();
}

fn setup_game(mut commands: Commands) {
    // Spawn camera, etc.
    commands.spawn(Camera2d);
    info!("Squatch game initialized, connecting to SpacetimeDB...");
}

// Handle SpacetimeDB connection
fn handle_connection(
    mut connected_events: EventReader<StdbConnectedEvent>,
    mut disconnected_events: EventReader<StdbDisconnectedEvent>,
    mut error_events: EventReader<StdbConnectionErrorEvent>,
    stdb: Res<StdbConnection>,
) {
    for _event in connected_events.read() {
        info!("✅ Connected to SpacetimeDB!");

        // Optionally call a reducer when connected
        // Example: Join the game
        if let Err(e) = stdb.reducers().call::<JoinGame>(("Player1".to_string(),)) {
            error!("Failed to join game: {:?}", e);
        }
    }

    for event in disconnected_events.read() {
        warn!("Disconnected from SpacetimeDB: {:?}", event.err);
    }

    for event in error_events.read() {
        error!("SpacetimeDB connection error: {}", event.err);
    }
}

// Handle player insert/update/delete events
fn handle_player_events(
    mut insert_events: EventReader<InsertEvent<SquatchPlayer>>,
    mut update_events: EventReader<UpdateEvent<SquatchPlayer>>,
    mut delete_events: EventReader<DeleteEvent<SquatchPlayer>>,
    mut commands: Commands,
    // Add your sprite assets here
) {
    // When a new player joins
    for event in insert_events.read() {
        info!("Player joined: {:?}", event.row);

        // Spawn a player entity in Bevy
        commands.spawn((
            // Your player components here
            // Sprite, Transform, etc.
        ));
    }

    // When a player updates (position, etc.)
    for event in update_events.read() {
        info!("Player updated: {} moved from ({}, {}) to ({}, {})",
            event.new.name,
            event.old.position_x, event.old.position_y,
            event.new.position_x, event.new.position_y
        );

        // Update the corresponding Bevy entity
        // Query for entity with player_id and update Transform
    }

    // When a player leaves
    for event in delete_events.read() {
        info!("Player left: {:?}", event.row);

        // Despawn the corresponding Bevy entity
    }
}

// Handle game state events
fn handle_game_state_events(
    mut events: EventReader<InsertUpdateEvent<GameState>>,
) {
    for event in events.read() {
        info!("Game state changed: {:?}", event.new);

        match event.new.state.as_str() {
            "waiting" => {
                info!("Waiting for players...");
            }
            "active" => {
                info!("Game is active!");
            }
            "finished" => {
                info!("Game finished!");
            }
            _ => {}
        }
    }
}

// Example: Send player input to SpacetimeDB
fn player_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    stdb: Res<StdbConnection>,
    // Your player query here
) {
    let player_id = 123; // Get from your player entity

    let mut x = 0.0;
    let mut y = 0.0;

    if keyboard.pressed(KeyCode::ArrowLeft) {
        x -= 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowRight) {
        x += 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowUp) {
        y += 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowDown) {
        y -= 1.0;
    }

    if x != 0.0 || y != 0.0 {
        // Send position update to server
        let _ = stdb.reducers().call::<UpdatePlayerPosition>((
            player_id,
            x,
            y,
        ));
    }

    if keyboard.just_pressed(KeyCode::Space) {
        // Send an action to the server
        let _ = stdb.reducers().call::<PlayerAction>((
            player_id,
            "jump".to_string(),
        ));
    }
}
```

## Step 5: Create the HTML Page

Create a file `squatch.html` in your project root (or wherever you'll serve from):

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Squatch - WASM Edition</title>
    <style>
        body {
            margin: 0;
            padding: 0;
            width: 100vw;
            height: 100vh;
            display: flex;
            justify-content: center;
            align-items: center;
            background: #1a1a1a;
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
        }

        #loading {
            color: #ffffff;
            text-align: center;
        }

        #loading h1 {
            margin: 0 0 1rem 0;
            color: #4CAF50;
        }

        #loading .spinner {
            border: 4px solid #333;
            border-top: 4px solid #4CAF50;
            border-radius: 50%;
            width: 40px;
            height: 40px;
            animation: spin 1s linear infinite;
            margin: 20px auto;
        }

        @keyframes spin {
            0% { transform: rotate(0deg); }
            100% { transform: rotate(360deg); }
        }

        canvas {
            background-color: #000000;
            display: block;
        }

        #error {
            display: none;
            color: #ff5555;
            text-align: center;
        }
    </style>
</head>
<body>
    <div id="loading">
        <h1>🐾 Loading Squatch...</h1>
        <div class="spinner"></div>
        <p>Initializing game and connecting to SpacetimeDB</p>
    </div>

    <div id="error">
        <h1>❌ Error Loading Game</h1>
        <p id="error-message"></p>
    </div>

    <!--
        CRITICAL: The SpacetimeDB bridge MUST be loaded BEFORE the WASM module!
        This initializes the global __SPACETIMEDB_BRIDGE__ object.
    -->
    <script type="module">
        console.log('[Squatch] Starting initialization...');

        // Step 1: Load and initialize the SpacetimeDB TypeScript SDK bridge
        try {
            // Update this path to match your setup
            const { SpacetimeDBBridge } = await import('../bevy_spacetimedb_wasm/bevy_spacetimedb/js/dist/spacetimedb-bridge.js');

            console.log('[Squatch] Initializing SpacetimeDB bridge...');
            window.__SPACETIMEDB_BRIDGE__ = new SpacetimeDBBridge();
            console.log('[Squatch] ✅ SpacetimeDB bridge ready');
        } catch (error) {
            console.error('[Squatch] Failed to load SpacetimeDB bridge:', error);
            document.getElementById('loading').style.display = 'none';
            document.getElementById('error').style.display = 'block';
            document.getElementById('error-message').textContent =
                'Failed to load SpacetimeDB bridge: ' + error.message;
            throw error;
        }

        // Step 2: Now that the bridge is ready, load the WASM module
        try {
            console.log('[Squatch] Loading WASM module...');

            // Update this path to match your wasm-pack output
            const init = await import('./pkg/squatch.js');
            await init.default();

            console.log('[Squatch] ✅ WASM module loaded');

            // Hide loading screen
            document.getElementById('loading').style.display = 'none';

            console.log('[Squatch] 🎮 Game ready!');
        } catch (error) {
            console.error('[Squatch] Failed to load WASM module:', error);
            document.getElementById('loading').style.display = 'none';
            document.getElementById('error').style.display = 'block';
            document.getElementById('error-message').textContent =
                'Failed to load game: ' + error.message;
        }
    </script>
</body>
</html>
```

## Step 6: Build for WASM

Create a build script or run these commands:

```bash
#!/bin/bash
# build-wasm.sh

echo "🔨 Building Squatch for WASM..."

# 1. Build the TypeScript bridge (if not already built)
echo "📦 Building TypeScript bridge..."
cd ../bevy_spacetimedb_wasm/bevy_spacetimedb/js
npm install
npm run build
cd -

# 2. Build the Rust WASM
echo "🦀 Building Rust WASM..."
wasm-pack build --target web --out-dir www/pkg

echo "✅ Build complete!"
echo "📁 Output in: www/pkg/"
echo "🌐 Open www/squatch.html in a browser"
```

Make it executable:
```bash
chmod +x build-wasm.sh
```

## Step 7: Set Up Your SpacetimeDB Module

Your SpacetimeDB module needs to match the schema defined in Rust. Here's an example structure:

```rust
// In your SpacetimeDB module (server-side)

use spacetimedb::{spacetimedb, ReducerContext, Identity};

#[spacetimedb(table)]
pub struct Players {
    #[primarykey]
    #[autoinc]
    pub id: u64,
    pub identity: Identity,
    pub name: String,
    pub position_x: f32,
    pub position_y: f32,
    pub health: i32,
}

#[spacetimedb(table)]
pub struct GameStates {
    #[primarykey]
    #[autoinc]
    pub id: u64,
    pub match_id: String,
    pub state: String,
    pub player_count: u32,
}

#[spacetimedb(reducer)]
pub fn join_game(ctx: ReducerContext, player_name: String) -> Result<(), String> {
    // Create new player
    Players::insert(Players {
        id: 0, // Auto-incremented
        identity: ctx.sender,
        name: player_name,
        position_x: 0.0,
        position_y: 0.0,
        health: 100,
    });

    Ok(())
}

#[spacetimedb(reducer)]
pub fn update_player_position(ctx: ReducerContext, player_id: u64, x: f32, y: f32) -> Result<(), String> {
    // Update player position
    if let Some(mut player) = Players::filter_by_id(&player_id) {
        player.position_x += x;
        player.position_y += y;
        Players::update_by_id(&player_id, player);
    }

    Ok(())
}
```

## Step 8: Run Your SpacetimeDB Server

```bash
# Publish your SpacetimeDB module
spacetime publish --project-path ./spacetimedb_module squatch

# Or run locally
spacetime start --project-path ./spacetimedb_module
```

## Step 9: Serve and Test

You'll need to serve the HTML file (because of WASM/module requirements):

```bash
# Option 1: Python
python -m http.server 8080

# Option 2: Node.js (http-server)
npx http-server -p 8080

# Option 3: Rust (basic-http-server)
cargo install basic-http-server
basic-http-server www
```

Then open `http://localhost:8080/squatch.html` in your browser.

## Troubleshooting

### "SpacetimeDB Bridge Not Found"
- Make sure you built the TypeScript bridge: `cd bevy_spacetimedb/js && npm run build`
- Check that the import path in your HTML matches the actual location
- Verify `window.__SPACETIMEDB_BRIDGE__` is set before WASM loads

### "Connection Failed"
- Check that your SpacetimeDB server is running
- Verify the URI in `with_uri()` matches your server
- Check browser console for WebSocket errors

### "Table not found: players"
- Ensure `TABLE_NAME` in your Rust code matches the SpacetimeDB table name exactly
- Table names are case-sensitive

### Build Errors
- Make sure you're building for `wasm32-unknown-unknown` target
- Run `rustup target add wasm32-unknown-unknown` if needed
- Check that all paths in Cargo.toml are correct

## Next Steps

1. **Add more tables** as your game needs them
2. **Implement game logic** using the event system
3. **Handle player authentication** with `.with_auth_token()`
4. **Add error handling** for network issues
5. **Optimize** by batching reducer calls if needed

## Development Workflow

```bash
# During development, watch for changes:
# Terminal 1: Watch TypeScript
cd bevy_spacetimedb/js
npm run watch

# Terminal 2: Rebuild WASM when Rust changes
cargo watch -x 'build --target wasm32-unknown-unknown'

# Terminal 3: Serve
basic-http-server www
```

Happy coding! 🐾
