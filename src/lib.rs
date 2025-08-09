//! Rig Playground: Minimal MCP server examples.
//!
//! This crate contains example implementations of MCP (Model Context Protocol)
//! servers built with the `rmcp` crate. It demonstrates how to expose simple
//! tools over different transports (stdio, SSE, and HTTP with streamable
//! responses).
//!
//! Modules
//! - `example_mcp`: contains a combined tool router `ExampleTools` exposing:
//!   - `increment`: a stateful counter that increments an in-memory value.
//!   - `calculate`: a stateless calculator that performs basic arithmetic.
//!
//! To run a server binary, see the crate's `main.rs`. Environment variables and
//! endpoints are documented on the `main` function.
pub mod example_mcp;