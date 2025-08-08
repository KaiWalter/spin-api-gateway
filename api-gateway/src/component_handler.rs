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
    request_host: String,
    query: String,
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

    // Create initial API request from incoming HTTP data
    let initial_req = host::ApiRequest {
        method: method.clone(),
        host: request_host,
        path: path.to_string(),
        query: query,
        headers: headers.clone(),
        body: body.clone(),
    };

    // Instantiate component
    let api_instance = match Api::instantiate_async(&mut store, &component, &linker).await {
        Ok(instance) => instance,
        Err(e) => {
            return Ok(warp::http::Response::builder()
                .status(warp::http::StatusCode::INTERNAL_SERVER_ERROR)
                .body(format!("Failed to instantiate component: {e}").into())
                .unwrap());
        }
    };

    // Step 1: Call handle_api_request with the incoming request information
    let processed_req = match api_instance
        .guest()
        .call_handle_api_request(&mut store, &initial_req)
        .await
    {
        Ok(req) => req,
        Err(e) => {
            return Ok(warp::http::Response::builder()
                .status(warp::http::StatusCode::INTERNAL_SERVER_ERROR)
                .body(format!("Failed to call handle_api_request: {e}").into())
                .unwrap());
        }
    };

    // Step 2: Call backend asynchronously based on the processed request parameters
    let backend_response = match call_backend_async(&processed_req).await {
        Ok(response) => response,
        Err(e) => {
            return Ok(warp::http::Response::builder()
                .status(warp::http::StatusCode::INTERNAL_SERVER_ERROR)
                .body(format!("Backend call failed: {e}").into())
                .unwrap());
        }
    };

    // Step 3: Pass the backend response through handle_api_response
    let final_response = match api_instance
        .guest()
        .call_handle_api_response(&mut store, &backend_response)
        .await
    {
        Ok(response) => response,
        Err(e) => {
            return Ok(warp::http::Response::builder()
                .status(warp::http::StatusCode::INTERNAL_SERVER_ERROR)
                .body(format!("Failed to call handle_api_response: {e}").into())
                .unwrap());
        }
    };

    // Convert the final response to HTTP response
    let mut builder = warp::http::Response::builder().status(
        warp::http::StatusCode::from_u16(final_response.status)
            .unwrap_or(warp::http::StatusCode::OK),
    );
    for (k, v) in final_response.headers.iter() {
        builder = builder.header(k, v);
    }
    let body = final_response.body.unwrap_or_default();
    Ok(builder.body(body).unwrap())
}

// Async backend call function
async fn call_backend_async(request: &host::ApiRequest) -> Result<host::ApiResponse, Box<dyn std::error::Error + Send + Sync>> {
    // Build the target URL from the request parameters
    let base_url = if request.host.starts_with("http://") || request.host.starts_with("https://") {
        request.host.clone()
    } else {
        format!("https://{}", request.host)
    };
    
    let url = if request.query.is_empty() {
        format!("{}{}", base_url, request.path)
    } else {
        format!("{}{}?{}", base_url, request.path, request.query)
    };

    // Create HTTP client
    let client = reqwest::Client::new();
    
    // Build the request
    let mut req_builder = match request.method.to_uppercase().as_str() {
        "GET" => client.get(&url),
        "POST" => client.post(&url),
        "PUT" => client.put(&url),
        "DELETE" => client.delete(&url),
        "PATCH" => client.patch(&url),
        "HEAD" => client.head(&url),
        _ => client.get(&url), // Default to GET for unknown methods
    };

    // Add headers
    for (key, value) in &request.headers {
        req_builder = req_builder.header(key, value);
    }

    // Add body if present
    if let Some(body) = &request.body {
        req_builder = req_builder.body(body.clone());
    }

    // Execute the request
    let response = req_builder.send().await?;
    
    // Extract response data
    let status = response.status().as_u16();
    let mut response_headers = Vec::new();
    
    // Convert response headers
    for (key, value) in response.headers().iter() {
        if let Ok(value_str) = value.to_str() {
            response_headers.push((key.to_string(), value_str.to_string()));
        }
    }
    
    // Get response body
    let response_body = response.bytes().await?;
    let body = if response_body.is_empty() {
        None
    } else {
        Some(response_body.to_vec())
    };

    Ok(host::ApiResponse {
        status,
        headers: response_headers,
        body,
    })
}
