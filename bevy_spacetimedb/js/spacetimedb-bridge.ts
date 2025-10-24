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

import { DbConnection, ReducerEvent } from '@clockworklabs/spacetimedb-sdk';

/** Callback function type for Rust WASM */
type WasmCallback = (...args: any[]) => void;

/** Data structure for table events passed to Rust */
interface TableEventData {
    row?: any;
    oldRow?: any;
    newRow?: any;
    reducerEvent: {
        callerIdentity: string;
        reducerName: string;
        args: any[];
    } | null;
}

/**
 * Bridge class that connects Rust WASM to the SpacetimeDB TypeScript SDK
 */
export class SpacetimeDBBridge {
    private connections: Map<number, DbConnection>;
    private nextConnectionId: number;
    private callbacks: Map<number, WasmCallback>;
    private nextCallbackId: number;

    constructor() {
        this.connections = new Map();
        this.nextConnectionId = 0;
        this.callbacks = new Map();
        this.nextCallbackId = 0;

        console.log('[SpacetimeDB Bridge] Initialized');
    }

    /**
     * Create a new connection to SpacetimeDB
     */
    createConnection(uri: string, moduleName: string, authToken: string | null): number {
        const conn = new DbConnection(uri, moduleName, authToken || undefined);
        const id = this.nextConnectionId++;
        this.connections.set(id, conn);
        console.log(`[SpacetimeDB Bridge] Created connection ${id} to ${uri}/${moduleName}`);
        return id;
    }

    /**
     * Connect to the SpacetimeDB server
     */
    async connect(connectionId: number): Promise<void> {
        const conn = this.connections.get(connectionId);
        if (!conn) {
            throw new Error(`Invalid connection ID: ${connectionId}`);
        }
        console.log(`[SpacetimeDB Bridge] Connecting ${connectionId}...`);
        await conn.connect();
        console.log(`[SpacetimeDB Bridge] Connected ${connectionId}`);
    }

    /**
     * Disconnect from the SpacetimeDB server
     */
    async disconnect(connectionId: number): Promise<void> {
        const conn = this.connections.get(connectionId);
        if (!conn) {
            throw new Error(`Invalid connection ID: ${connectionId}`);
        }
        console.log(`[SpacetimeDB Bridge] Disconnecting ${connectionId}...`);
        await conn.disconnect();
        this.connections.delete(connectionId);
        console.log(`[SpacetimeDB Bridge] Disconnected ${connectionId}`);
    }

    /**
     * Register a callback for connection events
     */
    onConnect(connectionId: number, callbackId: number): void {
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
     */
    onDisconnect(connectionId: number, callbackId: number): void {
        const conn = this.connections.get(connectionId);
        const callback = this.callbacks.get(callbackId);
        if (!conn || !callback) {
            console.error(`[SpacetimeDB Bridge] onDisconnect: Invalid connection or callback ID`);
            return;
        }

        conn.onDisconnect((err?: Error) => {
            console.log(`[SpacetimeDB Bridge] Connection ${connectionId} disconnected event`, err);
            callback(err?.message || null);
        });
    }

    /**
     * Register a callback for connection error events
     */
    onConnectionError(connectionId: number, callbackId: number): void {
        const conn = this.connections.get(connectionId);
        const callback = this.callbacks.get(callbackId);
        if (!conn || !callback) {
            console.error(`[SpacetimeDB Bridge] onConnectionError: Invalid connection or callback ID`);
            return;
        }

        conn.onConnectionError((err: Error) => {
            console.error(`[SpacetimeDB Bridge] Connection ${connectionId} error:`, err);
            callback(err?.message || 'Unknown error');
        });
    }

    /**
     * Call a reducer on the SpacetimeDB server
     */
    async callReducer(connectionId: number, reducerName: string, args: any): Promise<void> {
        const conn = this.connections.get(connectionId);
        if (!conn) {
            throw new Error(`Invalid connection ID: ${connectionId}`);
        }

        console.log(`[SpacetimeDB Bridge] Calling reducer ${reducerName} on connection ${connectionId}`, args);

        // Args should be an array that we spread
        const argsArray = Array.isArray(args) ? args : [args];
        await conn.call(reducerName, ...argsArray);
    }

    /**
     * Subscribe to a SQL query
     */
    async subscribe(connectionId: number, query: string): Promise<void> {
        const conn = this.connections.get(connectionId);
        if (!conn) {
            throw new Error(`Invalid connection ID: ${connectionId}`);
        }

        console.log(`[SpacetimeDB Bridge] Subscribing to query on connection ${connectionId}:`, query);
        await conn.subscribe([query]);
    }

    /**
     * Subscribe to table events
     */
    subscribeTable(
        connectionId: number,
        tableName: string,
        onInsertId: number | null,
        onUpdateId: number | null,
        onDeleteId: number | null
    ): void {
        const conn = this.connections.get(connectionId);
        if (!conn) {
            throw new Error(`Invalid connection ID: ${connectionId}`);
        }

        const table = (conn.db as any)[tableName];
        if (!table) {
            console.error(`[SpacetimeDB Bridge] Table not found: ${tableName}`);
            throw new Error(`Table not found: ${tableName}`);
        }

        console.log(`[SpacetimeDB Bridge] Subscribing to table ${tableName} on connection ${connectionId}`);

        if (onInsertId !== null) {
            const cb = this.callbacks.get(onInsertId);
            if (cb) {
                table.onInsert((row: any, reducerEvent?: ReducerEvent) => {
                    const data: TableEventData = {
                        row,
                        reducerEvent: reducerEvent ? {
                            callerIdentity: reducerEvent.callerIdentity.toHexString(),
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
                table.onUpdate((oldRow: any, newRow: any, reducerEvent?: ReducerEvent) => {
                    const data: TableEventData = {
                        oldRow,
                        newRow,
                        reducerEvent: reducerEvent ? {
                            callerIdentity: reducerEvent.callerIdentity.toHexString(),
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
                table.onDelete((row: any, reducerEvent?: ReducerEvent) => {
                    const data: TableEventData = {
                        row,
                        reducerEvent: reducerEvent ? {
                            callerIdentity: reducerEvent.callerIdentity.toHexString(),
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
     */
    registerCallback(callback: WasmCallback): number {
        const id = this.nextCallbackId++;
        this.callbacks.set(id, callback);
        return id;
    }

    /**
     * Unregister a callback
     */
    unregisterCallback(callbackId: number): void {
        this.callbacks.delete(callbackId);
    }
}

// Declare the global interface extension
declare global {
    interface Window {
        __SPACETIMEDB_BRIDGE__?: SpacetimeDBBridge;
    }
}
