# Copilot Instructions for spin-api-gateway

## Project Overview
This project is a multi-crate Rust workspace for building a plugin-based API Gateway using Fermyon Spin, Wasmtime, and WebAssembly components. It demonstrates dynamic routing to Wasm-based API plugins, with shared WIT interfaces for cross-component communication.

## Architecture & Data Flow
- The gateway (`api-gateway/src/main.rs`) loads and routes HTTP requests to Wasm API components (plugins) at runtime.
- Plugins (`api1`, `api2`, `api-js`) are compiled to Wasm and implement the shared WIT interface in `wit/shared-api.wit`.
- Routing is configured via `routes.json`, mapping HTTP paths to Wasm components in the build output folder.
- Gateway and plugins communicate using Spin SDK and WIT-defined types/functions.
- Wasm modules are cached in-memory for performance (see LRU cache in gateway).

## Developer Workflows
- **Build all Rust crates:**
  ```sh
  cargo build --workspace
  ```
- **Build Wasm API components:**
  ```sh
  cargo build -p api1 -p api2 --target wasm32-wasip2
  cd api-js && npm run build
  ```
- **Run gateway (native):**
  ```sh
  cargo run -p api-gateway
  ```
- **DevShell setup:**
  Use Nix (`nix develop`) for a shell with all required tools (Rust, Spin, Wasm, Node.js, etc). This auto-starts zsh and installs JS dependencies for `api-js`.

## Conventions & Patterns
- All Rust code uses 2021 edition and is formatted with `rustfmt`.
- WIT interfaces are versioned and shared in `wit/`.
- Use `cargo-component` for Wasm builds and Spin integration.
- Environment variables for OpenSSL are set in `flake.nix` for compatibility.
- New API components should be added as workspace members and have their own WIT interface if needed.
- JS plugins use `componentize-js` and must match the shared WIT interface.

## Integration Points
- External dependencies: Fermyon Spin, Wasmtime, Wasm tools, GitHub CLI, OpenSSL (via Nix), Node.js for JS plugins.
- Gateway loads Wasm modules dynamically using Wasmtime and Spin SDK.
- API components communicate via WIT-defined APIs.

## Key Files & Directories
- `api-gateway/src/main.rs`: Gateway logic, Wasm loading, routing
- `api1/src/lib.rs`, `api2/src/lib.rs`, `api-js/index.js`: API plugin logic
- `wit/shared-api.wit`: Shared API definitions
- `routes.json`: Path-to-component mapping
- `flake.nix`: Nix devShell and toolchain setup
- `Makefile`: Build shortcuts for all components

## Example: Adding a New API Component/Plugin
1. Create a new crate (e.g., `api3/`).
2. Add to workspace in root `Cargo.toml`.
3. Define WIT interface in `wit/` if needed.
4. Build with `cargo build -p api3 --target wasm32-wasip2`.
5. Update `routes.json` to route requests to the new component.
6. Restart the gateway to pick up new routes.

---
For questions or unclear patterns, ask for clarification or request examples from maintainers.
