use axum::{routing::post, Router, response::Html};
use std::sync::Arc;
use wasmtime::{Engine, Config, Store};
use wasmtime::component::{Component, Linker};
use wasmtime_wasi::p2::{WasiCtx, WasiCtxBuilder};

// Generate Rust bindings for the host from the same WIT interface
wasmtime::component::bindgen!({world:"api", path:"../wit/shared-api.wit"});

// Host state (if you want WASI support)
struct HostState {
    table: Table,
    wasi: WasiCtx,
}

impl WasiView for HostState {
    fn table(&self) -> &Table {
        &self.table
    }
    fn table_mut(&mut self) -> &mut Table {
        &mut self.table
    }
    fn ctx(&self) -> &WasiCtx {
        &self.wasi
    }
    fn ctx_mut(&mut self) -> &mut WasiCtx {
        &mut self.wasi
    }
}

#[tokio::main]
async fn main() {
    // Shared Wasmtime Engine
    let mut config = Config::new();
    config.wasm_component_model(true); // IMPORTANT for component model
    let engine = Arc::new(Engine::new(&config).unwrap());

    // Setup Axum HTTP server
    let app = Router::new().route("/api1", post({
        let engine = engine.clone();
        move |body| handle_api_request(engine.clone(), "api1.component.wasm", body)
    }));

    println!("🚀 Gateway listening on http://127.0.0.1:3000");
    axum::Server::bind(&"127.0.0.1:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handle_api_request(
    engine: Arc<Engine>,
    wasm_file: &str,
    body: axum::body::Body,
) -> Html<String> {
    // Collect request body
    let bytes = hyper::body::to_bytes(body.into_inner()).await.unwrap();
    let req_str = String::from_utf8_lossy(&bytes);

    println!("Gateway received request: {}", req_str);

    // Load the WASM component
    let component = Component::from_file(&engine, wasm_file)
        .expect("Failed to load component");

    // Create HostState (WASI ctx)
    let mut linker: Linker<HostState> = Linker::new(&engine);
    let table = Table::new();
    let wasi = WasiCtxBuilder::new().inherit_stdio().build();
    let mut store = Store::new(&engine, HostState { table, wasi });

    // Add the component's interface to linker
    Api::add_to_linker(&mut linker, |state| state).unwrap();

    // Instantiate the component
    let (api_instance, _) = Api::instantiate(&mut store, &component, &linker)
        .expect("Failed to instantiate component");

    // Call the function
    let response = api_instance
        .call_handle_request(&mut store, &req_str)
        .expect("Failed to call handle_request");

    Html(response)
}
