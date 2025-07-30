use wasmtime::component::{Component, Linker, ResourceTable};
use wasmtime::*;
use wasmtime_wasi::p2::{IoView, WasiCtx, WasiCtxBuilder, WasiView};
use std::convert::Infallible;
use warp::Filter;
use warp::filters::BoxedFilter;
use std::fs;
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;
use lru::LruCache;
use warp::http::Method;
use std::num::NonZeroUsize;

wasmtime::component::bindgen!("api" in "../wit/shared-api.wit");

const COMPONENT_CACHE_SIZE: usize = 10;

static COMPONENT_CACHE: Lazy<Mutex<LruCache<String, Arc<Vec<u8>>>>> = Lazy::new(|| {
    Mutex::new(LruCache::new(NonZeroUsize::new(COMPONENT_CACHE_SIZE).unwrap()))
});

#[derive(Debug, Deserialize, Clone)]
struct ApiRoute {
    path: String,
    component: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RouteConfig {
    base_folder: String,
    routes: Vec<ApiRoute>,
}

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

impl host::Host for ComponentRunStates {
    fn host_api_request(
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

async fn handle_api_component(method: String, path: &str, headers: Vec<(String, String)>, body: Option<Vec<u8>>, wasm_path: &str) -> Result<impl warp::Reply, Infallible> {
    let engine = Engine::default();
    let mut linker: Linker<ComponentRunStates> = Linker::new(&engine);
    wasmtime_wasi::p2::add_to_linker_sync(&mut linker).unwrap();
    let wasi = WasiCtxBuilder::new().inherit_stdio().inherit_args().build();
    let state = ComponentRunStates {
        wasi_ctx: wasi,
        resource_table: ResourceTable::new(),
    };
    let mut store = Store::new(&engine, state);
    let bytes = match get_component_bytes(wasm_path) {
        Ok(b) => b,
        Err(e) => {
            return Ok(warp::http::Response::builder()
                .status(warp::http::StatusCode::INTERNAL_SERVER_ERROR)
                .body(format!("Failed to load component bytes: {}", e).into())
                .unwrap());
        }
    };
    let component = match Component::new(&engine, bytes.as_ref()) {
        Ok(c) => c,
        Err(e) => {
            return Ok(warp::http::Response::builder()
                .status(warp::http::StatusCode::INTERNAL_SERVER_ERROR)
                .body(format!("Failed to create component: {}", e).into())
                .unwrap());
        }
    };
    // Api::add_to_linker::<_, wasmtime::component::HasSelf<_>>(&mut linker, |state| state).unwrap();
    let api_instance = Api::instantiate(&mut store, &component, &linker).expect("Failed to instantiate component");
    let req = host::ApiRequest {
        method,
        path: path.to_string(),
        headers,
        body,
    };
    let resp = api_instance.guest().call_handle_api_request(&mut store, &req);
    let reply = match resp {
        Ok(r) => {
            let mut builder = warp::http::Response::builder()
                .status(warp::http::StatusCode::from_u16(r.status as u16).unwrap_or(warp::http::StatusCode::OK));
            for (k, v) in r.headers.iter() {
                builder = builder.header(k, v);
            }
            let body = r.body.as_ref().map(|b| b.clone()).unwrap_or_default();
            let response = builder.body(body).unwrap();
            response
        },
        Err(_) => warp::http::Response::builder()
            .status(warp::http::StatusCode::INTERNAL_SERVER_ERROR)
            .body("Internal Server Error".into())
            .unwrap(),
    };
    Ok(reply)
}

#[tokio::main]
async fn main() {
    // Load routes from routes.json
    let routes_json = fs::read_to_string("routes.json").expect("Failed to read routes.json");
    let config: RouteConfig = serde_json::from_str(&routes_json).expect("Failed to parse routes.json");
    let api_routes = config.routes;
    let base_folder = config.base_folder;
    println!("Loaded API routes:");
    for route in &api_routes {
        println!("  {} -> {}/{}", route.path, base_folder, route.component);
    }
    // Build warp filters for each route and immediately combine into routes
    let mut routes: Option<BoxedFilter<(Box<dyn warp::Reply>,)>> = None;
    for route in api_routes {
        let path = route.path.trim_start_matches('/').to_string();
        let wasm_path = Arc::new(format!("{}/{}", base_folder, route.component));
        let filter = warp::path(path.clone())
            .and(warp::any().and(warp::method()).and(warp::header::headers_cloned()).and(warp::body::bytes()))
            .and_then({
                let path = path.clone();
                let wasm_path = wasm_path.clone();
                move |method: Method, headers: warp::http::HeaderMap, body: bytes::Bytes| {
                    let path = path.clone();
                    let wasm_path = wasm_path.clone();
                    async move {
                        let headers_vec = headers.iter().map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string())).collect();
                        let body_vec = if !body.is_empty() { Some(body.to_vec()) } else { None };
                        match handle_api_component(method.as_str().to_string(), &format!("/{}", path), headers_vec, body_vec, &wasm_path).await {
                            Ok(reply) => Ok::<Box<dyn warp::Reply>, warp::Rejection>(Box::new(reply)),
                            Err(_) => Err(warp::reject()),
                        }
                    }
                }
            })
            .boxed();
        routes = match routes {
            None => Some(filter),
            Some(existing) => Some(existing.or(filter).unify().boxed()),
        };
    }
    let routes = routes.expect("No routes defined");
    println!("Starting warp web server on 127.0.0.1:3030");
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
