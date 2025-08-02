use serde::Deserialize;
use std::fs;
use std::sync::Arc;
use warp::filters::BoxedFilter;
use warp::http::Method;
use warp::Filter;

use crate::component_handler;

#[derive(Debug, Deserialize, Clone)]
struct ApiRoute {
    path: String,
    component: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RouteConfig {
    base_folder: String,
    routes: Vec<ApiRoute>,
}

pub fn configuration() -> BoxedFilter<(Box<dyn warp::Reply>,)> {
    // Load routes from routes.json
    let routes_json = fs::read_to_string("routes.json").expect("Failed to read routes.json");
    let config: RouteConfig =
        serde_json::from_str(&routes_json).expect("Failed to parse routes.json");
    let api_routes = config.routes;
    let base_folder = config.base_folder;
    println!("Loaded API routes:");
    for route in &api_routes {
        println!("  {} -> {}/{}", route.path, base_folder, route.component);
    }
    // Build warp filters for each route and immediately combine into routes
    let mut routes: Option<BoxedFilter<(Box<dyn warp::Reply>,)>> = None;
    for route in api_routes {
        let path = route.path.trim_start_matches('/').to_string();
        let wasm_path = Arc::new(format!("{}/{}", base_folder, route.component));
        let filter = warp::path(path.clone())
            .and(
                warp::any()
                    .and(warp::method())
                    .and(warp::header::headers_cloned())
                    .and(warp::body::bytes()),
            )
            .and_then({
                let path = path.clone();
                let wasm_path = wasm_path.clone();
                move |method: Method, headers: warp::http::HeaderMap, body: bytes::Bytes| {
                    let path = path.clone();
                    let wasm_path = wasm_path.clone();
                    async move {
                        let headers_vec = headers
                            .iter()
                            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                            .collect();
                        let body_vec = if !body.is_empty() {
                            Some(body.to_vec())
                        } else {
                            None
                        };
                        match component_handler::handle_api_component(
                            method.as_str().to_string(),
                            &format!("/{path}"),
                            headers_vec,
                            body_vec,
                            &wasm_path,
                        )
                        .await
                        {
                            Ok(reply) => {
                                Ok::<Box<dyn warp::Reply>, warp::Rejection>(Box::new(reply))
                            }
                            Err(_) => Err(warp::reject()),
                        }
                    }
                }
            })
            .boxed();
        routes = match routes {
            None => Some(filter),
            Some(existing) => Some(existing.or(filter).unify().boxed()),
        };
    }

    routes.expect("No routes defined")
}
