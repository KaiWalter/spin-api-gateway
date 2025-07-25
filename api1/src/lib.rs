use exports::component::api1::data_handler::{Guest, ApiRequest};

wit_bindgen::generate!({
    world: "api",
    path: "../wit/world.wit",
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
