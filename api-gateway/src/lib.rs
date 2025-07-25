mod deps;

use deps::component_api1::component::api::request_handler::{handle_data as handle_data_api1, ApiRequest as request_api1};
use deps::component_api2::component::api::request_handler::{handle_data as handle_data_api2, ApiRequest as request_api2};
use spin_sdk::http::{IntoResponse, Request, Response};
use spin_sdk::http_component;
use serde_json::from_slice;

/// A simple Spin HTTP component.
#[http_component]
fn handle_api_gateway(req: Request) -> anyhow::Result<impl IntoResponse> {
    println!("Handling request to {:?}", req.header("spin-full-url"));

    let response_body = match req.path() {
        "/api1" => {
            let api_request: request_api1 = from_slice::<_>(req.body())?;
            format!("{:?}", handle_data_api1(&api_request))
        }
        "/api2" => {
            let api_request: request_api2 = from_slice::<_>(req.body())?;
            format!("{:?}", handle_data_api2(&api_request))
        }
        _ => {
            return Ok(Response::builder()
                .status(404)
                .body("Not Found")
                .build());
        }
    };

    Ok(Response::builder()
        .status(200)
        .header("content-type", "text/plain")
        .body(response_body)
        .build())
}
