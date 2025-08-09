// WIT bindings generation for the shared API
wasmtime::component::bindgen!({
    path: "../wit/shared-api.wit",
    world: "api",
    async: true
});

// The generated types are available through the host module and Api struct
// Access them via: bindings::host::ApiRequest, bindings::host::ApiResponse, bindings::Api
