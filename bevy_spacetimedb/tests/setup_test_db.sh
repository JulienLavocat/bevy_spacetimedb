#!/usr/bin/env bash
set -e

echo "Setting up SpacetimeDB test module..."

# Navigate to test module directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/test_module"

# Build the module
echo "Building test module..."
cargo build --release --target wasm32-unknown-unknown

# Publish to SpacetimeDB
echo "Publishing to SpacetimeDB..."
spacetime publish \
    --project-path . \
    bevy_spacetimedb_test_module \
    --clear-database

echo "✓ Test module published successfully!"
echo "Module name: bevy_spacetimedb_test_module"
