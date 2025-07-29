# Copilot Instructions for spin-api-gateway

## Project Overview
This is an experimental project to test whether a plugin-based API Gateway can be implemented using Spin, Wasmtime, or WebAssembly components. The repository is a multi-crate Rust workspace for building WebAssembly (Wasm) API components using Fermyon Spin. It includes:
- `api-gateway/`: Main gateway service (Rust binary), responsible for routing requests to plugins (API components)
- `api1/`, `api2/`: Example Wasm API components (Rust libraries, acting as plugins)
- `wit/`: Shared WIT interface definitions for Spin components

## Architecture & Data Flow
- The gateway (`api-gateway/src/main.rs`) routes requests to Wasm API components (`api1`, `api2`).
- Communication between gateway and API components uses Spin SDK and WIT interfaces.
- Each API component is a separate crate, compiled to Wasm, and loaded by the gateway at runtime as a plugin.
- WIT files in `wit/` define shared APIs and types for cross-component communication.
- The architecture is designed to validate routing and plugin extensibility in Spin.

## Developer Workflows
- **Build all crates:**
  ```sh
  cargo build --workspace
  ```
- **Build Wasm API components:**
  ```sh
  cargo build -p api1 -p api2 --target wasm32-wasi
  ```
- **Run gateway (native):**
  ```sh
  cargo run -p api-gateway
  ```
- **DevShell setup:**
  Enter with Nix (`nix develop`), which auto-starts zsh and provides all required tools (Rust, Spin, Wasm, etc).

## Conventions & Patterns
- All Rust code uses 2021 edition and is formatted with `rustfmt`.
- WIT interfaces are versioned and shared in `wit/`.
- Use `cargo-component` for Wasm builds and Spin integration.
- Environment variables for OpenSSL are set in `flake.nix` for compatibility.
- All new API components should be added as workspace members and have their own WIT interface if needed.

## Integration Points
- External dependencies: Fermyon Spin, Wasmtime, Wasm tools, GitHub CLI, OpenSSL (via Nix).
- Gateway loads Wasm modules dynamically using Wasmtime and Spin SDK.
- API components communicate via WIT-defined APIs.

## Key Files & Directories
- `api-gateway/src/main.rs`: Gateway logic and Wasm loading
- `api1/src/lib.rs`, `api2/src/lib.rs`: API component/plugin logic
- `wit/shared-api.wit`: Shared API definitions
- `flake.nix`: Nix devShell and toolchain setup

## Example: Adding a New API Component/Plugin
1. Create a new crate (e.g., `api3/`).
2. Add to workspace in root `Cargo.toml`.
3. Define WIT interface in `wit/`.
4. Build with `cargo build -p api3 --target wasm32-wasi`.
5. Update gateway to route requests to new component.

---
For questions or unclear patterns, ask for clarification or request examples from maintainers.
