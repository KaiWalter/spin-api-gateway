use crate::exports::guest::{Guest,ApiRequest,ApiResponse};

wit_bindgen::generate!({
    world: "api",
    path: "../wit/shared-api.wit",
});

struct Api;

impl Guest for Api {
    fn handle_api_request(request: ApiRequest) -> ApiResponse {
        // Call the host callback function
        let host_request = host::ApiRequest {
            method: request.method.clone(),
            path: format!("{}-from-api1", request.path),
            headers: request.headers.clone(),
            body: request.body.clone(),
        };
        
        let host_response = host::host_api_request(&host_request);
        
        let debug_info = format!(
            "[API1 Debug]\nOriginal Request - Method: {}, Path: {}\nHost Response - Status: {}, Body: {}\nOriginal Headers: {:#?}",
            request.method,
            request.path,
            host_response.status,
            host_response.body.as_ref().map(|b| String::from_utf8_lossy(b)).unwrap_or_else(|| "<none>".into()),
            request.headers
        );
        
        ApiResponse {
            status: 200,
            headers: vec![("content-type".to_string(), "text/plain".to_string())],
            body: Some(debug_info.into_bytes()),
        }
    }
}

export!(Api);
