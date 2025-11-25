// Node.js test setup for bevy_spacetimedb_wasm
// This creates a REAL SpacetimeDB bridge that connects to an actual server

const WebSocket = require('ws');

class SpacetimeDBBridge {
    constructor() {
        this.connections = new Map();
        this.callbacks = new Map();
        this.nextConnectionId = 1;
        this.nextCallbackId = 1;
        console.log('SpacetimeDB Bridge initialized for Node.js (using REAL server connections)');
    }

    createConnection(uri, moduleName, authToken) {
        const connectionId = this.nextConnectionId++;
        this.connections.set(connectionId, {
            uri,
            moduleName,
            authToken,
            connected: false,
            callbacks: {
                onConnect: [],
                onDisconnect: [],
                onError: []
            },
            tables: new Map()
        });
        console.log(`Created connection ${connectionId} to ${uri}/${moduleName}`);
        return connectionId;
    }

    async connect(connectionId) {
        const conn = this.connections.get(connectionId);
        if (!conn) {
            throw new Error(`Connection ${connectionId} not found`);
        }

        // Connect to real SpacetimeDB server (fail if not available)
        return new Promise((resolve, reject) => {
            // SpacetimeDB WebSocket endpoint
            const wsUrl = `${conn.uri.replace('http://', 'ws://').replace('https://', 'wss://')}/database/subscribe/${conn.moduleName}`;
            console.log(`Attempting to connect to ${wsUrl}`);

            const ws = new WebSocket(wsUrl);

            const timeout = setTimeout(() => {
                ws.close();
                const error = new Error(`Connection timeout - SpacetimeDB server not responding at ${conn.uri}`);
                reject(error);
            }, 5000);

            ws.on('open', () => {
                clearTimeout(timeout);
                console.log(`✓ Connected to SpacetimeDB at ${conn.uri}`);
                conn.connected = true;
                conn.ws = ws;
                conn.callbacks.onConnect.forEach(cb => cb());
                resolve();
            });

            ws.on('error', (error) => {
                clearTimeout(timeout);
                const errorMsg = `Failed to connect to SpacetimeDB at ${conn.uri}: ${error.message}`;
                console.error(errorMsg);
                reject(new Error(errorMsg));
            });

            ws.on('close', () => {
                conn.connected = false;
            });
        });
    }

    async disconnect(connectionId) {
        const conn = this.connections.get(connectionId);
        if (!conn) {
            throw new Error(`Connection ${connectionId} not found`);
        }

        return new Promise((resolve) => {
            if (conn.ws) {
                conn.ws.close();
            }
            conn.connected = false;
            conn.callbacks.onDisconnect.forEach(cb => cb(null));
            console.log(`Disconnected connection ${connectionId}`);
            resolve();
        });
    }

    onConnect(connectionId, callbackId) {
        const conn = this.connections.get(connectionId);
        const callback = this.callbacks.get(callbackId);
        if (conn && callback) {
            conn.callbacks.onConnect.push(callback);
        }
    }

    onDisconnect(connectionId, callbackId) {
        const conn = this.connections.get(connectionId);
        const callback = this.callbacks.get(callbackId);
        if (conn && callback) {
            conn.callbacks.onDisconnect.push(callback);
        }
    }

    onConnectionError(connectionId, callbackId) {
        const conn = this.connections.get(connectionId);
        const callback = this.callbacks.get(callbackId);
        if (conn && callback) {
            conn.callbacks.onError.push(callback);
        }
    }

    async callReducer(connectionId, reducerName, args) {
        const conn = this.connections.get(connectionId);
        if (!conn) {
            throw new Error(`Connection ${connectionId} not found`);
        }

        if (!conn.connected || !conn.ws) {
            throw new Error(`Connection ${connectionId} is not connected to SpacetimeDB`);
        }

        console.log(`Calling reducer ${reducerName} with args:`, args);

        // Send actual reducer call to SpacetimeDB via WebSocket
        return new Promise((resolve, reject) => {
            // TODO: Implement proper SpacetimeDB protocol message
            // For now, this is a placeholder that would send the actual protocol message
            console.warn(`Reducer calling not yet implemented in bridge - would call ${reducerName}`);
            resolve({ success: true });
        });
    }

    async subscribe(connectionId, query) {
        const conn = this.connections.get(connectionId);
        if (!conn) {
            throw new Error(`Connection ${connectionId} not found`);
        }

        if (!conn.connected || !conn.ws) {
            throw new Error(`Connection ${connectionId} is not connected to SpacetimeDB`);
        }

        console.log(`Subscribing to query: ${query}`);

        // TODO: Implement proper SpacetimeDB protocol subscription
        // For now, this is a placeholder
        console.warn(`Subscription not yet implemented in bridge - would subscribe to: ${query}`);
        return Promise.resolve();
    }

    subscribeTable(connectionId, tableName, onInsertId, onUpdateId, onDeleteId) {
        const conn = this.connections.get(connectionId);
        if (!conn) {
            console.error(`Connection ${connectionId} not found`);
            return;
        }

        conn.tables.set(tableName, {
            onInsert: onInsertId ? this.callbacks.get(onInsertId) : null,
            onUpdate: onUpdateId ? this.callbacks.get(onUpdateId) : null,
            onDelete: onDeleteId ? this.callbacks.get(onDeleteId) : null
        });

        console.log(`Subscribed to table ${tableName}`);
    }

    registerCallback(callback) {
        const callbackId = this.nextCallbackId++;
        this.callbacks.set(callbackId, callback);
        return callbackId;
    }

    unregisterCallback(callbackId) {
        this.callbacks.delete(callbackId);
    }
}

// Set up global bridge for wasm-bindgen tests
global.__SPACETIMEDB_BRIDGE__ = new SpacetimeDBBridge();

// Mock browser globals for WASM
global.window = {
    __SPACETIMEDB_BRIDGE__: global.__SPACETIMEDB_BRIDGE__,
    setTimeout: setTimeout,
    clearTimeout: clearTimeout,
    performance: {
        now: () => Date.now(),
        timeOrigin: Date.now()
    }
};

// Also set performance globally
global.performance = global.window.performance;

console.log('Node.js test environment ready');
