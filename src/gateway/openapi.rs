use crate::config::OpenApiConfig;
use crate::gateway::{GatewayEntry, Route};
use serde_json::{Map, Value};

pub fn parse_from_json(config: OpenApiConfig, buffer: &[u8]) -> GatewayEntry {
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

    GatewayEntry { config, openapi_file: buffer.to_vec(), routes }
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
