mod deps;

use deps::component_api1::component::api1::data_handler::{handle_data as handle_data_api1, MyObject};
use deps::component_api2::component::api2::data_handler as api2;
use spin_sdk::http::{IntoResponse, Request, Response};
use spin_sdk::http_component;
use serde_json::from_slice;

/// A simple Spin HTTP component.
#[http_component]
fn handle_api_gateway(req: Request) -> anyhow::Result<impl IntoResponse> {
    println!("Handling request to {:?}", req.header("spin-full-url"));
    let my_object: MyObject = from_slice::<_>(req.body())?;

    let handled_data = match req.path() {
        "/api1" => handle_data_api1(&my_object),
        "/api2" => {
            let obj = api2::MyObject {
                name: my_object.name.clone(),
                steps: my_object.steps,
                processed: my_object.processed,
            };
            let res = api2::handle_data(&obj);
            MyObject {
                name: res.name,
                steps: res.steps,
                processed: res.processed,
            }
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
        .body(format!("{:?}", handled_data))
        .build())
}
