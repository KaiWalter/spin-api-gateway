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

    world imports-api2 {
        import request-handler;
    }
    "#,
    additional_derives: [serde::Deserialize],
    world: "imports-api2",
});
