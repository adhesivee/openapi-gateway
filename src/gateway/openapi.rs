use crate::gateway::{OpenApiEntry, Route};
use axum::http::Uri;
use serde_json::{Map, Value};
use std::slice::Iter;
use crate::config::OpenApiConfig;

pub fn build_from_json(config: OpenApiConfig, buffer: &[u8]) -> OpenApiEntry {
    let json: Value = serde_json::from_slice(buffer).unwrap();

    let mut default_server = Map::new();
    default_server.insert("url".to_string(), Value::String("/".to_string()));

    let default_servers = vec![Value::Object(default_server)];

    let routes: Vec<_> = json["servers"]
        .as_array()
        .unwrap_or(&default_servers)
        .iter()
        .map(|server| {
            let server_prefix = server["url"].as_str().unwrap().trim_end_matches("/");

            let routes = collect_routes(&json, server_prefix);

            println!("{:?}", &routes);
            routes
        })
        .flatten()
        .collect();

    OpenApiEntry {
        config,
        routes,
    }
}

fn collect_routes(json: &Value, server_prefix: &str) -> Vec<Route> {
    json["paths"]
        .as_object()
        .unwrap()
        .iter()
        .map(|path| {
            let path_uri = format!("{}{}", server_prefix, path.0,);

            let methods: Vec<_> = path
                .1
                .as_object()
                .unwrap()
                .iter()
                .map(|method| method.0.clone())
                .collect();

            (path_uri, methods).into()
        })
        .collect()
}
