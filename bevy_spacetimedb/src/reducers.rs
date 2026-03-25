// Global reducer callbacks were removed in SpacetimeDB SDK 2.0.
// `add_reducer` / `RegisterableReducerMessage` / `#[derive(RegisterReducerMessage)]` no longer exist.
//
// Migration paths:
// - To observe the result of YOUR OWN reducer calls, use the `_then()` variant at the call site:
//     stdb.reducers().my_reducer_then(args, |ctx, result| { ... }).unwrap();
// - To observe reducer invocations from ANY client, define an `#[table(..., event)]` on the server
//   and subscribe to it on the client with `add_table`.

