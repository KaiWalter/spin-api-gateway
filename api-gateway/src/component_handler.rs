use lru::LruCache;
use once_cell::sync::Lazy;
use std::convert::Infallible;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};
use wasmtime::component::{Component, Linker, ResourceTable};
use wasmtime::*;
use wasmtime_wasi::p2::{IoView, WasiCtx, WasiCtxBuilder, WasiView};
use wasmtime_wasi_http::{WasiHttpCtx, WasiHttpView};

use crate::bindings::{host, Api};

const COMPONENT_CACHE_SIZE: usize = 10;

static COMPONENT_CACHE: Lazy<Mutex<LruCache<String, Arc<Vec<u8>>>>> = Lazy::new(|| {
    Mutex::new(LruCache::new(
        NonZeroUsize::new(COMPONENT_CACHE_SIZE).unwrap(),
    ))
});

pub struct ComponentRunStates {
    pub wasi_ctx: WasiCtx,
    pub resource_table: ResourceTable,
    pub http: WasiHttpCtx,
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

impl WasiHttpView for ComponentRunStates {
    fn ctx(&mut self) -> &mut WasiHttpCtx {
        &mut self.http
    }
}

impl host::Host for ComponentRunStates {
    async fn host_api_request(&mut self, request: host::ApiRequest) -> host::ApiResponse {
        host::ApiResponse {
            status: 200,
            headers: vec![("content-type".to_string(), "text/plain".to_string())],
            body: Some(format!("Hello from Host! You said: {}", request.path).into_bytes()),
        }
    }
}

fn get_component_bytes(wasm_path: &str) -> std::io::Result<Arc<Vec<u8>>> {
    let mut cache = COMPONENT_CACHE.lock().unwrap();
    if let Some(bytes) = cache.get(wasm_path) {
        return Ok(bytes.clone());
    }
    let bytes = std::fs::read(wasm_path)?;
    let arc_bytes = Arc::new(bytes);
    cache.put(wasm_path.to_string(), arc_bytes.clone());
    Ok(arc_bytes)
}

pub async fn handle_api_component(
    method: String,
    path: &str,
    headers: Vec<(String, String)>,
    body: Option<Vec<u8>>,
    wasm_path: &str,
) -> Result<impl warp::Reply, Infallible> {
    let mut config = Config::new();
    config.async_support(true); // Enable async support
    config.wasm_component_model(true); // Enable component model
    let engine = Engine::new(&config).unwrap();
    let mut linker: Linker<ComponentRunStates> = Linker::new(&engine);

    // Add WASI functionality
    wasmtime_wasi::p2::add_to_linker_async(&mut linker).unwrap();

    // Add WASI HTTP functionality
    wasmtime_wasi_http::add_only_http_to_linker_async(&mut linker).unwrap();

    // Add our host implementation
    host::add_to_linker::<ComponentRunStates, wasmtime::component::HasSelf<_>>(
        &mut linker,
        |state| state,
    )
    .unwrap();

    let wasi = WasiCtxBuilder::new().inherit_stdio().inherit_args().build();
    let state = ComponentRunStates {
        wasi_ctx: wasi,
        resource_table: ResourceTable::new(),
        http: WasiHttpCtx::new(),
    };
    let mut store = Store::new(&engine, state);

    let bytes = match get_component_bytes(wasm_path) {
        Ok(b) => b,
        Err(e) => {
            return Ok(warp::http::Response::builder()
                .status(warp::http::StatusCode::INTERNAL_SERVER_ERROR)
                .body(format!("Failed to load component bytes: {e}").into())
                .unwrap());
        }
    };

    let component = match Component::new(&engine, bytes.as_ref()) {
        Ok(c) => c,
        Err(e) => {
            return Ok(warp::http::Response::builder()
                .status(warp::http::StatusCode::INTERNAL_SERVER_ERROR)
                .body(format!("Failed to create component: {e}").into())
                .unwrap());
        }
    };

    let req = host::ApiRequest {
        method,
        path: path.to_string(),
        headers,
        body,
    };

    // Instantiate and call component asynchronously
    match Api::instantiate_async(&mut store, &component, &linker).await {
        Ok(api_instance) => {
            match api_instance
                .guest()
                .call_handle_api_request(&mut store, &req)
                .await
            {
                Ok(r) => {
                    let mut builder = warp::http::Response::builder().status(
                        warp::http::StatusCode::from_u16(r.status)
                            .unwrap_or(warp::http::StatusCode::OK),
                    );
                    for (k, v) in r.headers.iter() {
                        builder = builder.header(k, v);
                    }
                    let body = r.body.unwrap_or_default();
                    Ok(builder.body(body).unwrap())
                }
                Err(_) => Ok(warp::http::Response::builder()
                    .status(warp::http::StatusCode::INTERNAL_SERVER_ERROR)
                    .body("Internal Server Error".into())
                    .unwrap()),
            }
        }
        Err(e) => Ok(warp::http::Response::builder()
            .status(warp::http::StatusCode::INTERNAL_SERVER_ERROR)
            .body(format!("Failed to instantiate component: {e}").into())
            .unwrap()),
    }
}
