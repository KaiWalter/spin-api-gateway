use exports::component::api2::data_handler::{Guest, ApiRequest};

wit_bindgen::generate!({
    world: "api",
    path: "../wit/api2.wit",
});

struct Component;

impl Guest for Component {
    fn handle_data(mut request: ApiRequest) -> ApiRequest {
        println!("{:?}", request);

        // Manipulating the object
        request.steps += 1;
        request.processed = Some(true);

        request
    }
}

export!(Component);
