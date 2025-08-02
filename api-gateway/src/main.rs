mod bindings;
mod component_handler;
mod routes;

#[tokio::main]
async fn main() {
  let routes = routes::configuration();
    println!("Starting warp web server on 127.0.0.1:3030");
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
