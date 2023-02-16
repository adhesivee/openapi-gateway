mod config;
mod gateway;
mod openapi;
mod ui;
mod web;

use std::fmt::Debug;
use crate::config::{Config, OpenApiConfig};
use crate::gateway::openapi::{ContentType, parse_openapi};
use crate::gateway::GatewayEntry;
use crate::web::{simple_get, new_https_client, serve_with_config, HttpsClient, HttpError};
use chrono::Utc;
use cron_parser::parse;
use std::sync::Arc;
use axum::http::{HeaderValue, Uri};
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};
use tracing::Level;
use thiserror::Error;

const CONFIG_FILE: &str = "openapi-gateway-config.toml";

pub type RwGatewayEntries = Arc<RwLock<Vec<GatewayEntry>>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let collector = tracing_subscriber::fmt()
        .json()
        .with_max_level(Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(collector)
        .unwrap();


    let config = Config::parse_from_file(&format!("{}/{CONFIG_FILE}", std::env::current_dir().unwrap().to_str().unwrap())).unwrap();
    let reload_cron = config.reload_cron.clone();

    let client = new_https_client();

    let mut entries = vec![];
    for config in config.openapi_urls.into_iter() {
        entries.push(fetch_entry(&client, &config).await?);
    }

    let entries = Arc::new(RwLock::from(entries));
    spawn_reload_cron(reload_cron, Arc::clone(&entries)).await;

    Ok(serve_with_config(client, entries).await)
}

async fn spawn_reload_cron(reload_cron: String, entries: Arc<RwLock<Vec<GatewayEntry>>>) {
    tokio::spawn(async move {
        let client = new_https_client();

        loop {
            if let Ok(next) = parse(&reload_cron, &Utc::now()) {
                let diff = next - Utc::now();

                sleep(Duration::from_secs(diff.num_seconds() as u64)).await;

                tracing::info!("Start collecting OpenAPI files");

                let mut reloaded_entries = {
                    let mut entries = entries.read().await;

                    let mut reload_entries = vec![];
                    for entry in entries.iter() {
                        reload_entries.push(
                            fetch_entry(&client, &entry.config).await
                        );
                    }

                    reload_entries
                };

                {
                    let mut entries = entries.write().await;

                    entries.iter_mut()
                        .for_each(|entry| {
                            if let Ok(reload_entry) = reloaded_entries.remove(0) {
                                *entry = reload_entry;
                            }
                        })
                }

                sleep(Duration::from_secs(1)).await;
            }
        }
    });
}

async fn fetch_entry(client: &HttpsClient, config: &OpenApiConfig) -> Result<GatewayEntry, FetchError> {
    let response = simple_get(&client, &config.uri()).await?;

    let content_type = response.0.get("content-type")
        .unwrap_or(&HeaderValue::from_str("application/json").unwrap())
        .to_str()
        .unwrap()
        .to_lowercase();

    let content_type = if content_type == "application/yaml" {
        ContentType::YAML
    } else {
        ContentType::JSON
    };

    Ok(
        parse_openapi(
            content_type,
            config.clone(),
            &response.1,
        )
    )
}

#[derive(Error, Debug)]
pub enum FetchError {
    #[error("HTTP error")]
    HttpError(#[from] HttpError),
    #[error("Parse error")]
    ParseError(#[from] Box<dyn std::error::Error + Send>),
}