use axum::http::Uri;
use serde::Deserialize;
use std::path::Path;
use toml::de::Error;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub reload_cron: String,
    pub openapi_urls: Vec<OpenApiConfig>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct OpenApiConfig {
    pub name: String,
    pub url: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("TOML error")]
    Toml(#[from] Error),
    #[error("IO error")]
    IO(#[from] std::io::Error),
}

const CONFIG_ENVIRONMENT_PREFIX: &str = "OPENAPI_";

impl Config {
    pub fn parse_from_file<P: AsRef<Path>>(path: P) -> Result<Config, ConfigError> {
        Ok(
            toml::from_str(
                std::fs::read_to_string(path)?
                    .as_str(),
            )?
        )
    }

    pub fn parse_from_env() -> Result<Config, ConfigError> {
        let mut configs = vec![];
        let mut count = 0u32;
        loop {
            let url = std::env::var(format!("{}{}_URL", CONFIG_ENVIRONMENT_PREFIX, count));
            let name = std::env::var(format!("{}{}_NAME", CONFIG_ENVIRONMENT_PREFIX, count));

            match (url, name) {
                (Ok(url), Ok(name)) => {
                    configs.push(
                        OpenApiConfig {
                            name,
                            url,
                        }
                    )
                }
                (Ok(_), Err(_)) => {
                    tracing::warn!("URL found, name missing");
                    break;
                }
                (Err(_), Ok(_)) => {
                    tracing::warn!("Name found, URL missing");
                    break;
                }
                (Err(_), Err(_)) => {
                    tracing::warn!("No new config found");
                    break;
                }
            };

            count += 1;
        }

        Ok(Config {
            reload_cron: std::env::var(format!("{}RELOAD_CRON", CONFIG_ENVIRONMENT_PREFIX)).unwrap_or("* * * * *".to_string()),
            openapi_urls: configs,
        })
    }
}

impl OpenApiConfig {
    pub fn uri(&self) -> Uri {
        Uri::try_from(&self.url).unwrap()
    }
}
