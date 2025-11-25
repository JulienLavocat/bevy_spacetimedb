# bevy_spacetimedb_wasm Tests

Real integration tests for the bevy_spacetimedb_wasm crate.

## Philosophy

**No Mocks. Real Server Only.**

These tests connect to an actual SpacetimeDB server. We don't maintain mocks because:
- The real thing is available in our test environment
- Mocks hide integration bugs
- Testing against the real server gives us confidence

## Prerequisites

- wasm-pack installed (`cargo install wasm-pack`)
- Node.js and npm installed
- **SpacetimeDB server running on `localhost:3000`** ← REQUIRED
- SpacetimeDB CLI (`spacetime`) for publishing test module

## Setup

1. Install test dependencies:
   ```bash
   cd tests
   npm install
   ```

2. Start SpacetimeDB:
   ```bash
   spacetime start
   ```

3. Publish the test module (one-time setup or when module changes):
   ```bash
   ./tests/setup_test_db.sh
   ```

## Running Tests

From the `bevy_spacetimedb` directory:

```bash
nix develop -c wasm-pack test --node -- --test integration_test
```

## Test Suite

**All tests require a real SpacetimeDB server.**

- `test_real_connection` - Connects to SpacetimeDB and validates connection lifecycle

We don't test whether Bevy works, or whether serde works, or whether our dependencies work.
We trust our dependencies. We only test OUR integration code against a REAL server.

## What Happens Without a Server?

The integration test will **FAIL** with:
```
❌ FAILED: SpacetimeDB server not running on localhost:3000! Start with: spacetime start
```

This is **intentional**. The test suite tells you exactly what's wrong and how to fix it.

## Troubleshooting

### "Failed to connect to SpacetimeDB server"

Start the server:
```bash
spacetime start
```

Verify it's running:
```bash
ss -tlnp | grep :3000
```

### "bridge not found"

Make sure `node_setup.js` is loaded. The test runner handles this automatically.

### Build errors

Build for WASM target:
```bash
cargo check --target wasm32-unknown-unknown
```
