# Copilot Instructions for wasm-api-gateway

## Overview
- Multi-language plugins (Rust, JS, TinyGo) compiled to Wasm (wasip2), routed by a Rust gateway using Warp + Wasmtime Component Model.
- Shared WIT world `api` in `wit/shared-api.wit` defines request/response types and guest/host functions for cross-component calls.

## Architecture & Flow
- Gateway (`api-gateway`) starts Warp on 127.0.0.1:3030 (`api-gateway/src/main.rs`).
- Routes are loaded from `routes.json` by `api-gateway/src/routes.rs` and mapped to Wasm artifacts under `baseFolder`.
- For each request (`routes.rs`):
  1) Load Wasm bytes with an LRU cache (size 10) keyed by file path (`component_handler.rs`).
  2) Instantiate component with Wasmtime using bindings from `api-gateway/src/bindings.rs` (WIT world `api`, async).
  3) Call guest `handle_api_request` to transform the outgoing backend call parameters.
  4) Gateway performs the backend HTTP call with `reqwest` (`call_backend_async`) using method/host/path/query/headers/body.
  5) Call guest `handle_api_response` to post-process the backend response.
  6) Return final HTTP response to the client.
- A host import `host.host-api-request` is implemented in the gateway (see `impl host::Host for ComponentRunStates`), available for plugins if needed.
- Cache note: rebuilt Wasm at the same path will be served from cache; restart the gateway to pick up changes.

## Build & Run (Makefile)
- Build everything: `make all`
  - Rust plugin: `make api-rs` -> `target/wasm32-wasip2/debug/api_rs.wasm`
  - JS plugin: `make api-js` -> `target/wasm32-wasip2/debug/api-js.component.wasm`
  - TinyGo plugin (experimental): `make api-go` -> `target/wasm32-wasip2/debug/api-go.wasm`
  - Gateway: `make api-gateway`
- Run gateway: `make run` (logs loaded routes; serves on 127.0.0.1:3030)
- Quick test: `curl 'http://127.0.0.1:3030/api-rs?foo=bar'` (Rust plugin defaults backend to `https://httpbin.org/get`).

## Routing
- `routes.json` maps `path` (single path segment, no wildcards) to a Wasm `component` under `baseFolder`.
- Example (`routes.json`): `/api-rs -> target/wasm32-wasip2/debug/api_rs.wasm`, `/api-js -> .../api-js.component.wasm`, `/api-go -> .../api-go.wasm`.
- Edit `routes.json` and restart the gateway to add/remove routes.

## Plugins & WIT
- WIT: `wit/shared-api.wit` (package `gateway:api`, world `api`) with records `http-handler.api-request` and `api-response` and functions:
  - guest: `handle-api-request(api-request) -> api-request`, `handle-api-response(api-response) -> api-response`
  - host: `host-api-request(api-request) -> api-response`
- Rust plugin (`api-rs/src/lib.rs`): uses `wit_bindgen`, implements `exports::guest::Guest` and sets backend (e.g., `https://httpbin.org`, `/get`).
- JS plugin (`api-js/index.js`): exports `guest` with async `handleApiRequest/handleApiResponse`.
- TinyGo plugin (`api-go/main.go`): uses generated bindings, sets `guest.Exports.*` handlers. Tools required: `wkg`, `wit-bindgen-go`, `tinygo` (available via `nix develop`).

## Conventions & Tips
- Target: `wasm32-wasip2` for all plugins. Gateway is native Rust.
- Backend call defaults to HTTPS if `request.host` has no scheme; set `http://...` explicitly for non-TLS or custom ports.
- Incoming headers are forwarded to the backend; plugins can modify headers/method/path/query/body via `handle_api_request`.
- Route matching is an exact segment (e.g., `/api-rs`), not a prefix; adjust `routes.rs` if subpaths are needed.
- Dev shell (`flake.nix`): provides rust toolchain, Wasmtime, wasm-tools, Spin, Node.js, Go/TinyGo, `wkg`, `wit-bindgen-go`, OpenSSL env.

Questions or missing patterns? Specify which area (routing, cache invalidation, plugin build, WIT changes), and we will refine these instructions.
