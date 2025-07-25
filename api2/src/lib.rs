use exports::component::api1::data_handler::{Guest, MyObject};

wit_bindgen::generate!({
    world: "api",
    path: "../wit/world.wit",
});

struct Component;

impl Guest for Component {
    fn handle_data(mut input: MyObject) -> MyObject {
        println!("{:?}", input);

        // Manipulating the object
        input.steps += 1;
        input.processed = Some(true);

        input
    }
}

export!(Component);
