use exports::component::api2::request_handler::{Guest, ApiRequest};

wit_bindgen::generate!({
    inline: r#"
    package component:api2;

    interface request-handler {
        handle-data: func(key: api-request) -> api-request;

        record api-request {
            name: string,
            steps: u32,
            processed: option<bool>,
        }
    }

    world api {
        export request-handler;
    }
    "#,
    world: "api",
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
