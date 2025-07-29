use wasmtime::component::{Component, Linker, ResourceTable};
use wasmtime::*;
use wasmtime_wasi::p2::{IoView, WasiCtx, WasiCtxBuilder, WasiView};
use std::convert::Infallible;
use warp::Filter;
use warp::filters::BoxedFilter;
use std::fs;
use serde::Deserialize;
use std::sync::Arc;

wasmtime::component::bindgen!("api" in "../wit/shared-api.wit");

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
            .and(warp::get())
            .and_then({
                let path = path.clone();
                let wasm_path = wasm_path.clone();
                move || {
                    let path = path.clone();
                    let wasm_path = wasm_path.clone();
                    async move {
                        match handle_api_component(&format!("/{}", path), &wasm_path).await {
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
