use crate::exports::guest::{Guest,ApiRequest,ApiResponse};

wit_bindgen::generate!({
    world: "api",
    path: "../wit/shared-api.wit",
});

struct Api;

impl Guest for Api {
    fn handle_api_request(request: ApiRequest) -> ApiResponse {
        let debug_info = format!(
            "[API2 Debug]\nMethod: {}\nPath: {}\nHeaders: {:#?}\nBody: {}",
            request.method,
            request.path,
            request.headers,
            request.body.as_ref().map(|b| String::from_utf8_lossy(b)).unwrap_or_else(|| "<none>".into())
        );
        ApiResponse {
            status: 200,
            headers: vec![("content-type".to_string(), "text/plain".to_string())],
            body: Some(debug_info.into_bytes()),
        }
    }
}

export!(Api);
