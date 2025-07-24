mod deps;

use deps::component_api1::component::api::data_handler as api1;
use deps::component_api2::component::api::data_handler as api2;
use spin_sdk::http::{IntoResponse, Request, Response};
use spin_sdk::http_component;
use serde_json::from_slice;

/// A simple Spin HTTP component.
#[http_component]
fn handle_api_gateway(req: Request) -> anyhow::Result<impl IntoResponse> {
    println!("Handling request to {:?}", req.header("spin-full-url"));

    let path = req.path();
    let response = match path {
        "/api1" => {
            let my_object: api1::MyObject = from_slice::<_>(req.body())?;
            let handled = api1::handle_data(&my_object);
            Response::builder()
                .status(200)
                .header("content-type", "text/plain")
                .body(format!("{:?}", handled))
                .build()
        }
        "/api2" => {
            let my_object: api2::MyObject = from_slice::<_>(req.body())?;
            let handled = api2::handle_data(&my_object);
            Response::builder()
                .status(200)
                .header("content-type", "text/plain")
                .body(format!("{:?}", handled))
                .build()
        }
        _ => Response::builder()
                .status(404)
                .header("content-type", "text/plain")
                .body("not found")
                .build(),
    };

    Ok(response)
}