#!/usr/bin/env bash
set -e

echo "Setting up test environment..."

# Install Node.js dependencies for tests
cd tests
if [ ! -d "node_modules" ]; then
    echo "Installing test dependencies..."
    npm install
fi
cd ..

echo "Running wasm-pack tests..."

# Set up the bridge before running tests
export NODE_OPTIONS="--require $(pwd)/tests/node_setup.js"

# Run the tests
wasm-pack test --node --release -- --test integration_test

echo "Tests completed!"
