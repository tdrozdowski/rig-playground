# Rig Playground

Minimal MCP (Model Context Protocol) server example built with the `rmcp` crate. This project exposes simple tools over a single transport:

- HTTP JSON‑RPC with streamable responses at `/mcp` (stateful sessions)

The example tools are intentionally small so you can focus on the server wiring:

- Counter (stateful): increments an in‑memory counter and returns the previous value
- Calculator (stateless): performs basic arithmetic (add, multiply)

Project code contains rustdoc. See:
- mcp/src/example_mcp.rs for tool documentation and types
- mcp/src/main.rs for server entry point, environment variables, and endpoint details


## Contents
- Quick start
- Running the HTTP (streamable) server (default)
- Exploring with the included HTTP requests
- Calling tools over HTTP (curl examples)
- Logging and troubleshooting
- Development


## Quick start
Prerequisites:
- Rust toolchain (Rust 1.80+ recommended; project targets edition 2024)

Install and run the HTTP server (default mode):

```bash
# From project root
cargo run -p mcp
```

This starts an HTTP server on 127.0.0.1:8000, exposing the JSON‑RPC MCP endpoint at:

- POST http://127.0.0.1:8000/mcp

The server uses stateful sessions and supports streamable responses.


## Running the HTTP (streamable) server (default)
The default binary entry point is `main_http()`:
- Binds to 127.0.0.1:8000 (see BIND_ADDRESS in mcp/src/main.rs)
- Exposes a single JSON‑RPC endpoint at `/mcp`
- Uses a LocalSessionManager with stateful sessions enabled

Environment:
- RUST_LOG controls verbosity (defaults to `debug` in code if unset)

Example:
```bash
RUST_LOG=info cargo run -p mcp
```


## Exploring with the included HTTP requests
If you use JetBrains IDEs or VS Code REST Client, you can exercise the API via:
- http/explore.http

The file contains a full flow for the streamable HTTP transport:
1) initialize
2) notifications/initialized
3) tools/list
4) tools/call (Counter::increment), with SSE event handling

Note: The server is stateful. The first initialize response returns an `mcp-session-id` header. Subsequent requests must include this header, e.g.:

- MCP-Protocol-Version: 2025-06-18
- mcp-session-id: <value from initialize response>


## Calling tools over HTTP (curl examples)
Below are minimal examples equivalent to http/explore.http. Replace the session id after the first call.

1) Initialize (capture mcp-session-id from response headers):
```bash
curl -i -X POST http://127.0.0.1:8000/mcp \
  -H 'Content-Type: application/json' \
  -H 'Accept: application/json, text/event-stream' \
  -H 'MCP-Protocol-Version: 2025-06-18' \
  --data '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "initialize",
    "params": {
      "protocolVersion": "2025-06-18",
      "capabilities": { "roots": {"listChanged": true}, "sampling": {}, "elicitation": {} },
      "clientInfo": { "name": "ExampleClient", "title": "Example Client", "version": "1.0.0" }
    }
  }'
```

2) Send required initialized notification:
```bash
curl -s -X POST http://127.0.0.1:8000/mcp \
  -H 'Content-Type: application/json' \
  -H 'Accept: application/json, text/event-stream' \
  -H 'MCP-Protocol-Version: 2025-06-18' \
  -H "mcp-session-id: $SESSION" \
  --data '{ "jsonrpc": "2.0", "method": "notifications/initialized" }'
```

3) List tools:
```bash
curl -s -X POST http://127.0.0.1:8000/mcp \
  -H 'Content-Type: application/json' \
  -H 'Accept: application/json, text/event-stream' \
  -H 'MCP-Protocol-Version: 2025-06-18' \
  -H "mcp-session-id: $SESSION" \
  --data '{ "jsonrpc": "2.0", "id": 2, "method": "tools/list" }'
```

4a) Call Counter::increment (no arguments):
```bash
curl -sN -X POST http://127.0.0.1:8000/mcp \
  -H 'Content-Type: application/json' \
  -H 'Accept: application/json, text/event-stream' \
  -H 'MCP-Protocol-Version: 2025-06-18' \
  -H "mcp-session-id: $SESSION" \
  --data '{
    "jsonrpc": "2.0",
    "id": 3,
    "method": "tools/call",
    "params": { "name": "increment" }
  }'
```

4b) Call Calculator::calculate (arguments a, b, operation):
```bash
curl -s -X POST http://127.0.0.1:8000/mcp \
  -H 'Content-Type: application/json' \
  -H 'Accept: application/json, text/event-stream' \
  -H 'MCP-Protocol-Version: 2025-06-18' \
  -H "mcp-session-id: $SESSION" \
  --data '{
    "jsonrpc": "2.0",
    "id": 4,
    "method": "tools/call",
    "params": {
      "name": "calculate",
      "arguments": { "a": 2, "b": 3, "operation": "multiply" }
    }
  }'
```
Notes:
- Supported operations: "add" and "multiply".
- The exact request envelope follows the MCP JSON‑RPC used by `rmcp`.


## Notes on other transports
The crate depends on `rmcp` features that can enable other transports (e.g., SSE, stdio), but this repository’s binary currently implements only the HTTP streamable server at `/mcp`. Future iterations may wire up additional transports.


## Logging and troubleshooting
This project uses `tracing` and `tracing-subscriber`.
- Set RUST_LOG to control verbosity, e.g. `RUST_LOG=info` or `RUST_LOG=debug`.
- Logs include lifecycle messages like server bind, shutdown, and tool calls.

If ports are busy, change `BIND_ADDRESS` in `mcp/src/main.rs`.


## Development
- Build: `cargo build`
- Run: `cargo run -p mcp`
- Format: `cargo fmt`
- Lint: `cargo clippy`

Key dependencies:
- rmcp = 0.5 (server, schemars, macros, transport-streamable-http-server)
- axum = 0.8, tokio = 1, tracing, serde/serde_json

Project modules:
- `example_mcp` contains:
  - `Counter` (stateful): increments a shared `Arc<Mutex<i32>>`; tool name `increment`
  - `Calculator` (stateless): tool name `calculate` with schema `{ a: i32, b: i32, operation: "add"|"multiply" }`

For deeper API docs, run:
```bash
cargo doc --open
```
