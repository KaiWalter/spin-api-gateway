use crate::exports::guest::{ApiRequest, ApiResponse, Guest};

wit_bindgen::generate!({
    world: "api",
    path: "../wit/shared-api.wit",
});

struct Api;

impl Guest for Api {
    fn handle_api_request(request: ApiRequest) -> ApiRequest {
        let mut r = request.clone();
        r.host = "https://httpbin.org".to_string();

        r
    }
    fn handle_api_response(response: ApiResponse) -> ApiResponse {
        response.clone()
    }
}

export!(Api);
