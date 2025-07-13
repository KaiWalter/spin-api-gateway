mod deps;

use bindings::deps::component::api1::data_handler::{handle_data, MyObject};
use spin_sdk::http::{IntoResponse, Request, Response};
use spin_sdk::http_component;
use serde_json::from_slice;

/// A simple Spin HTTP component.
#[http_component]
fn handle_api_gateway(req: Request) -> anyhow::Result<impl IntoResponse> {
    println!("Handling request to {:?}", req.header("spin-full-url"));
    let my_object: MyObject = from_slice::<_>(req.body())?;
    let handled_data = handle_data(&my_object);

    Ok(Response::builder()
        .status(200)
        .header("content-type", "text/plain")
        .body(format!("{:?}", handled_data))
        .build())
}