use wasmtime::component::{Component, HasSelf, Linker, ResourceTable};
use wasmtime::*;
use wasmtime_wasi::p2::bindings::sync::Command;
use wasmtime_wasi::p2::{IoView, WasiCtx, WasiCtxBuilder, WasiView};
use std::convert::Infallible;
use warp::Filter;

wasmtime::component::bindgen!("api" in "../wit/shared-api.wit");

pub struct ComponentRunStates {
    pub wasi_ctx: WasiCtx,
    pub resource_table: ResourceTable,
}

impl IoView for ComponentRunStates {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.resource_table
    }
}
impl WasiView for ComponentRunStates {
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi_ctx
    }
}

// Host implementation for wit interface remains unchanged
impl gateway::api::http_handler::Host for ComponentRunStates {
    fn handle_api_request(
        &mut self,
        request: gateway::api::http_handler::ApiRequest,
    ) -> gateway::api::http_handler::ApiResponse {
        gateway::api::http_handler::ApiResponse {
            status: 200,
            headers: vec![("content-type".to_string(), "text/plain".to_string())],
            body: Some(format!("Hello from Host! You said: {}", request.path).into_bytes()),
        }
    }
}

// Generalized handler for API requests to Wasm components
async fn handle_api_component(path: &str, wasm_path: &str) -> Result<impl warp::Reply, Infallible> {
    let engine = Engine::default();
    let mut linker: Linker<ComponentRunStates> = Linker::new(&engine);
    wasmtime_wasi::p2::add_to_linker_sync(&mut linker).unwrap();
    let wasi = WasiCtxBuilder::new().inherit_stdio().inherit_args().build();
    let state = ComponentRunStates {
        wasi_ctx: wasi,
        resource_table: ResourceTable::new(),
    };
    let mut store = Store::new(&engine, state);
    let component = Component::from_file(&engine, wasm_path).expect("Failed to load component");
    Api::add_to_linker::<_, wasmtime::component::HasSelf<_>>(&mut linker, |state| state).unwrap();
    let api_instance = Api::instantiate(&mut store, &component, &linker).expect("Failed to instantiate component");
    let req = exports::gateway::api::http_handler::ApiRequest {
        method: "GET".to_string(),
        path: path.to_string(),
        headers: vec![],
        body: None,
    };
    let resp = api_instance.gateway_api_http_handler().call_handle_api_request(&mut store, &req);
    let reply = match resp {
        Ok(r) => warp::reply::with_status(
            r.body.as_ref().map(|b| String::from_utf8_lossy(b).to_string()).unwrap_or_default(),
            warp::http::StatusCode::from_u16(r.status as u16).unwrap_or(warp::http::StatusCode::OK),
        ),
        Err(_) => warp::reply::with_status("Internal Server Error".to_string(), warp::http::StatusCode::INTERNAL_SERVER_ERROR),
    };
    Ok(reply)
}

#[tokio::main]
async fn main() {
    let api1_route = warp::path("api1")
        .and(warp::get())
        .and_then(|| async { handle_api_component("/api1", "target/wasm32-wasip2/debug/api1.wasm").await });
    let api2_route = warp::path("api2")
        .and(warp::get())
        .and_then(|| async { handle_api_component("/api2", "target/wasm32-wasip2/debug/api2.wasm").await });
    let routes = api1_route.or(api2_route);
    println!("Starting warp web server on 127.0.0.1:3030");
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
