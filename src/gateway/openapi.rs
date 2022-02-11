use regex::{escape, Regex};
use crate::config::OpenApiConfig;
use crate::gateway::{GatewayEntry, Route};
use crate::openapi::{OpenApiV3, Server};

pub fn parse_from_json(config: OpenApiConfig, buffer: &[u8]) -> GatewayEntry {
    let mut json: OpenApiV3 = serde_json::from_slice(buffer).unwrap();

    if json.servers.is_empty() {
        json.servers.push(Server { url: "/".to_string() })
    }

    let routes: Vec<_> = json.servers.iter()
        .map(|server| {
            let server_prefix = server.url.trim_end_matches("/");

            let routes = collect_routes(&json, server_prefix);

            println!("{:?}", &routes);
            routes
        })
        .flatten()
        .collect();

    GatewayEntry { config, openapi_file: buffer.to_vec(), routes }
}

fn collect_routes(json: &OpenApiV3, server_prefix: &str) -> Vec<Route> {
    json.paths
        .iter()
        .map(|path| {
            let path_uri = format!("{}{}", server_prefix, path.0);

            let methods: Vec<_> = path.1
                .keys()
                .map(|method| method.clone())
                .collect();

            let regex = Regex::new(&format!("^{}$", &escape(&path_uri)))
                .unwrap();
            (regex, methods).into()
        })
        .collect()
}
