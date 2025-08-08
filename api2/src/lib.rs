use crate::exports::guest::{ApiRequest, ApiResponse, Guest};

wit_bindgen::generate!({
    world: "api",
    path: "../wit/shared-api.wit",
});

struct Api;

impl Guest for Api {
    fn handle_api_request(request: ApiRequest) -> ApiRequest {
        request.clone()
    }
    fn handle_api_response(response: ApiResponse) -> ApiResponse {
        response.clone()
    }
}

export!(Api);
