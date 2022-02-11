mod config;
mod gateway;
mod ui;
mod web;

use crate::config::Config;
use crate::gateway::openapi::parse_from_json;
use crate::web::{get_bytes, new_https_client, serve_with_config};

const CONFIG_FILE: &str = "openapi-gateway-config.toml";

#[tokio::main]
async fn main() {
    let config = Config::parse_from_file(CONFIG_FILE).unwrap();

    let client = new_https_client();

    let mut entries = vec![];
    for config in config.openapi_urls.into_iter() {
        let bytes = get_bytes(&client, &config.uri())
            .await;

        entries.push(parse_from_json(config, &bytes));
    }

    serve_with_config(client, entries)
        .await
}
