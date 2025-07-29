use wasmtime::component::{Component, HasSelf, Linker, ResourceTable};
use wasmtime::*;
use wasmtime_wasi::p2::bindings::sync::Command;
use wasmtime_wasi::p2::{IoView, WasiCtx, WasiCtxBuilder, WasiView};

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

fn main() -> Result<()> {
    let engine = Engine::default();
    let mut linker: Linker<ComponentRunStates> = Linker::new(&engine);
    wasmtime_wasi::p2::add_to_linker_sync(&mut linker)?;

    let wasi = WasiCtxBuilder::new().inherit_stdio().inherit_args().build();
    let state = ComponentRunStates {
        wasi_ctx: wasi,
        resource_table: ResourceTable::new(),
    };
    let mut store = Store::new(&engine, state);

    println!("instantiating component...");
    let component = Component::from_file(&engine, "target/wasm32-wasip2/debug/api1.wasm").expect("Failed to load component");
    let wasi = WasiCtxBuilder::new().inherit_stdio().inherit_args().build();
    let state = ComponentRunStates {
        wasi_ctx: wasi,
        resource_table: ResourceTable::new(),
    };
    let mut store = Store::new(&engine, state);

    Api::add_to_linker::<_, HasSelf<_>>(&mut linker, |state| state).unwrap();
    let api_instance = Api::instantiate(&mut store, &component, &linker)
        .expect("Failed to instantiate component");

    let req = exports::gateway::api::http_handler::ApiRequest {
        body: None,
        path: "/hello".to_string(),
        headers: vec![],
        method: "POST".to_string(),
    };

    let response = api_instance
        .gateway_api_http_handler()
        .call_handle_api_request(&mut store, &req)
        .expect("Failed to call handle_request");

    println!("response: {:?}", response);

    Ok(())
}
