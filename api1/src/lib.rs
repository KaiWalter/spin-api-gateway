// use crate::exports::gateway::api::http_handler::{Guest,HttpRequest,HttpResponse};
//
// wit_bindgen::generate!({
//     world: "api",
//     path: "../wit/shared-api.wit",
// });
//
// struct Api;
//
// impl Guest for Api {
//     fn handle_http_request(request: HttpRequest) -> HttpResponse {
//         HttpResponse {
//             status: 200,
//             headers: vec![("content-type".to_string(), "text/plain".to_string())],
//             body: Some(format!("Hello from API1! You said: {}", request.path).into_bytes()),
//         }
//     }
// }
//
// export!(Api);

use std::thread::sleep;
use std::time::{Duration, Instant};

fn main() {
    println!("Hello, world!");
    let start = Instant::now();
    sleep(Duration::from_millis(100));
    println!("Napped for {:?}", Instant::now().duration_since(start));
}
