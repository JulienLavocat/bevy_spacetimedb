/**
 * SpacetimeDB TypeScript SDK Bridge for Rust WASM
 *
 * This bridge allows Rust WASM code to interact with the SpacetimeDB TypeScript SDK.
 * It must be loaded and initialized BEFORE the WASM module is loaded.
 *
 * Usage:
 * ```html
 * <script type="module">
 *   import { SpacetimeDBBridge } from './js/spacetimedb-bridge.js';
 *   window.__SPACETIMEDB_BRIDGE__ = new SpacetimeDBBridge();
 *
 *   // Then load your WASM module
 *   import init from './pkg/my_game.js';
 *   await init();
 * </script>
 * ```
 */

import { DbConnection } from '@clockworklabs/spacetimedb-sdk';

export class SpacetimeDBBridge {
    constructor() {
        /** @type {Map<number, DbConnection>} */
        this.connections = new Map();
        this.nextConnectionId = 0;

        /** @type {Map<number, Function>} */
        this.callbacks = new Map();
        this.nextCallbackId = 0;

        console.log('[SpacetimeDB Bridge] Initialized');
    }

    /**
     * Create a new connection to SpacetimeDB
     * @param {string} uri - WebSocket URI (e.g., "ws://localhost:3000")
     * @param {string} moduleName - Name of the SpacetimeDB module
     * @param {string|null} authToken - Optional authentication token
     * @returns {number} Connection ID
     */
    createConnection(uri, moduleName, authToken) {
        const conn = new DbConnection(uri, moduleName, authToken);
        const id = this.nextConnectionId++;
        this.connections.set(id, conn);
        console.log(`[SpacetimeDB Bridge] Created connection ${id} to ${uri}/${moduleName}`);
        return id;
    }

    /**
     * Connect to the SpacetimeDB server
     * @param {number} connectionId - Connection ID
     * @returns {Promise<void>}
     */
    async connect(connectionId) {
        const conn = this.connections.get(connectionId);
        if (!conn) throw new Error(`Invalid connection ID: ${connectionId}`);
        console.log(`[SpacetimeDB Bridge] Connecting ${connectionId}...`);
        await conn.connect();
        console.log(`[SpacetimeDB Bridge] Connected ${connectionId}`);
    }

    /**
     * Disconnect from the SpacetimeDB server
     * @param {number} connectionId - Connection ID
     * @returns {Promise<void>}
     */
    async disconnect(connectionId) {
        const conn = this.connections.get(connectionId);
        if (!conn) throw new Error(`Invalid connection ID: ${connectionId}`);
        console.log(`[SpacetimeDB Bridge] Disconnecting ${connectionId}...`);
        await conn.disconnect();
        this.connections.delete(connectionId);
        console.log(`[SpacetimeDB Bridge] Disconnected ${connectionId}`);
    }

    /**
     * Register a callback for connection events
     * @param {number} connectionId - Connection ID
     * @param {number} callbackId - Callback ID
     */
    onConnect(connectionId, callbackId) {
        const conn = this.connections.get(connectionId);
        const callback = this.callbacks.get(callbackId);
        if (!conn || !callback) {
            console.error(`[SpacetimeDB Bridge] onConnect: Invalid connection or callback ID`);
            return;
        }

        conn.onConnect(() => {
            console.log(`[SpacetimeDB Bridge] Connection ${connectionId} connected event`);
            callback();
        });
    }

    /**
     * Register a callback for disconnection events
     * @param {number} connectionId - Connection ID
     * @param {number} callbackId - Callback ID
     */
    onDisconnect(connectionId, callbackId) {
        const conn = this.connections.get(connectionId);
        const callback = this.callbacks.get(callbackId);
        if (!conn || !callback) {
            console.error(`[SpacetimeDB Bridge] onDisconnect: Invalid connection or callback ID`);
            return;
        }

        conn.onDisconnect((err) => {
            console.log(`[SpacetimeDB Bridge] Connection ${connectionId} disconnected event`, err);
            callback(err?.message || null);
        });
    }

    /**
     * Register a callback for connection error events
     * @param {number} connectionId - Connection ID
     * @param {number} callbackId - Callback ID
     */
    onConnectionError(connectionId, callbackId) {
        const conn = this.connections.get(connectionId);
        const callback = this.callbacks.get(callbackId);
        if (!conn || !callback) {
            console.error(`[SpacetimeDB Bridge] onConnectionError: Invalid connection or callback ID`);
            return;
        }

        conn.onConnectionError((err) => {
            console.error(`[SpacetimeDB Bridge] Connection ${connectionId} error:`, err);
            callback(err?.message || 'Unknown error');
        });
    }

    /**
     * Call a reducer on the SpacetimeDB server
     * @param {number} connectionId - Connection ID
     * @param {string} reducerName - Name of the reducer
     * @param {any} args - Reducer arguments (will be spread)
     * @returns {Promise<void>}
     */
    async callReducer(connectionId, reducerName, args) {
        const conn = this.connections.get(connectionId);
        if (!conn) throw new Error(`Invalid connection ID: ${connectionId}`);

        console.log(`[SpacetimeDB Bridge] Calling reducer ${reducerName} on connection ${connectionId}`, args);

        // Args should be an array that we spread
        const argsArray = Array.isArray(args) ? args : [args];
        await conn.call(reducerName, ...argsArray);
    }

    /**
     * Subscribe to a SQL query
     * @param {number} connectionId - Connection ID
     * @param {string} query - SQL query string
     * @returns {Promise<void>}
     */
    async subscribe(connectionId, query) {
        const conn = this.connections.get(connectionId);
        if (!conn) throw new Error(`Invalid connection ID: ${connectionId}`);

        console.log(`[SpacetimeDB Bridge] Subscribing to query on connection ${connectionId}:`, query);
        await conn.subscribe([query]);
    }

    /**
     * Subscribe to table events
     * @param {number} connectionId - Connection ID
     * @param {string} tableName - Table name
     * @param {number|null} onInsertId - Insert callback ID (null to skip)
     * @param {number|null} onUpdateId - Update callback ID (null to skip)
     * @param {number|null} onDeleteId - Delete callback ID (null to skip)
     */
    subscribeTable(connectionId, tableName, onInsertId, onUpdateId, onDeleteId) {
        const conn = this.connections.get(connectionId);
        if (!conn) throw new Error(`Invalid connection ID: ${connectionId}`);

        const table = conn.db[tableName];
        if (!table) {
            console.error(`[SpacetimeDB Bridge] Table not found: ${tableName}`);
            throw new Error(`Table not found: ${tableName}`);
        }

        console.log(`[SpacetimeDB Bridge] Subscribing to table ${tableName} on connection ${connectionId}`);

        if (onInsertId !== null) {
            const cb = this.callbacks.get(onInsertId);
            if (cb) {
                table.onInsert((row, reducerEvent) => {
                    const data = {
                        row: row,
                        reducerEvent: reducerEvent ? {
                            callerIdentity: reducerEvent.callerIdentity.toString(),
                            reducerName: reducerEvent.reducerName,
                            args: reducerEvent.args,
                        } : null
                    };
                    cb(JSON.stringify(data));
                });
            }
        }

        if (onUpdateId !== null) {
            const cb = this.callbacks.get(onUpdateId);
            if (cb) {
                table.onUpdate((oldRow, newRow, reducerEvent) => {
                    const data = {
                        oldRow: oldRow,
                        newRow: newRow,
                        reducerEvent: reducerEvent ? {
                            callerIdentity: reducerEvent.callerIdentity.toString(),
                            reducerName: reducerEvent.reducerName,
                            args: reducerEvent.args,
                        } : null
                    };
                    cb(JSON.stringify(data));
                });
            }
        }

        if (onDeleteId !== null) {
            const cb = this.callbacks.get(onDeleteId);
            if (cb) {
                table.onDelete((row, reducerEvent) => {
                    const data = {
                        row: row,
                        reducerEvent: reducerEvent ? {
                            callerIdentity: reducerEvent.callerIdentity.toString(),
                            reducerName: reducerEvent.reducerName,
                            args: reducerEvent.args,
                        } : null
                    };
                    cb(JSON.stringify(data));
                });
            }
        }
    }

    /**
     * Register a JavaScript callback that can be called from Rust
     * @param {Function} callback - JavaScript function
     * @returns {number} Callback ID
     */
    registerCallback(callback) {
        const id = this.nextCallbackId++;
        this.callbacks.set(id, callback);
        return id;
    }

    /**
     * Unregister a callback
     * @param {number} callbackId - Callback ID
     */
    unregisterCallback(callbackId) {
        this.callbacks.delete(callbackId);
    }
}
