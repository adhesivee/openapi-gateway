mod config;
mod gateway;
mod openapi;
mod ui;
mod web;

use crate::config::Config;
use crate::gateway::openapi::parse_from_json;
use crate::gateway::GatewayEntry;
use crate::web::{get_bytes, new_https_client, serve_with_config};
use chrono::Utc;
use cron_parser::parse;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};
use tracing::Level;

const CONFIG_FILE: &str = "openapi-gateway-config.toml";

pub type RwGatewayEntries = Arc<RwLock<Vec<GatewayEntry>>>;

#[tokio::main]
async fn main() {
    let collector = tracing_subscriber::fmt()
        .json()
        .with_max_level(Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(collector)
        .unwrap();

    let config = Config::parse_from_file(CONFIG_FILE).unwrap();
    let reload_cron = config.reload_cron.clone();

    let client = new_https_client();

    let mut entries = vec![];
    for config in config.openapi_urls.into_iter() {
        let bytes = get_bytes(&client, &config.uri()).await;

        entries.push(parse_from_json(config, &bytes));
    }

    let entries = Arc::new(RwLock::from(entries));

    let cron_entries = Arc::clone(&entries);
    tokio::spawn(async move {
        let entries = cron_entries;
        loop {
            if let Ok(next) = parse(&reload_cron, &Utc::now()) {
                let diff = next - Utc::now();

                sleep(Duration::from_secs(diff.num_seconds() as u64)).await;

                tracing::info!("Start collecting OpenAPI files");

                let mut entries = entries.write().await;

                for entry in entries.iter_mut() {
                    *entry = parse_from_json(entry.config.clone(), &entry.openapi_file.to_vec());
                }

                sleep(Duration::from_secs(1)).await;
            }
        }
    });

    serve_with_config(client, entries).await
}
