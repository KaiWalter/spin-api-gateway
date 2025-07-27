// Import the WIT-generated trait for http-handler
wit_bindgen::generate!({
    world: "api-component",
    path: "../wit/shared-api.wit",
    // or proper path to wit file
});

use exports::gateway::api::http_handler::{self, HttpRequest, HttpResponse};

// Exported implementation for the component
struct ApiHandler;

impl http_handler::Guest for ApiHandler {
    fn handle_http_request(request: HttpRequest) -> HttpResponse {
        // Example logic: echo the path
        let body = format!("Hello from API1! You called: {}", request.path)
            .into_bytes();

        HttpResponse {
            status: 200,
            headers: vec![
                ("content-type".to_string(), "text/plain".to_string())
            ],
            body: Some(body),
        }
    }
}

// Entrypoint for the wasm module (if required by the toolchain)
export!(ApiHandler);
