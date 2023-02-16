use std::env::var;
use axum::http::Uri;
use serde::Deserialize;
use std::path::Path;
use hyper::server::conn::Http;
use toml::de::Error;
use std::str::FromStr;
use crate::config::HttpMethod::{DELETE, GET, HEAD, OPTIONS, PATCH, POST, PUT};

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub reload_cron: String,
    pub openapi_urls: Vec<OpenApiConfig>,
    #[serde(default)]
    pub global_cors: Option<CorsConfig>
}

#[derive(Deserialize, Debug, Clone)]
pub struct OpenApiConfig {
    pub name: String,
    pub url: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct CorsConfig {
    pub allowed_origin: String,
    pub allowed_methods: Vec<HttpMethod>,
    pub allowed_headers: Vec<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
    OPTIONS,
    HEAD
}

impl FromStr for HttpMethod {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "GET" => GET,
            "POST" => POST,
            "PUT" => PUT,
            "PATCH" => PATCH,
            "DELETE" => DELETE,
            "OPTIONS" => OPTIONS,
            "HEAD" => HEAD,
            _ => {
                return Err(ConfigError::InvalidHttpMethod(s.to_string()))
            }
        })
    }
}

impl From<&HttpMethod> for &str {
    fn from(value: &HttpMethod) -> Self {
        match value {
            GET => "GET",
            POST => "POST",
            PUT => "PUT",
            PATCH => "PATCH",
            DELETE => "DELETE",
            OPTIONS => "OPTIONS",
            HEAD => "HEAD",
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("TOML error")]
    Toml(#[from] Error),
    #[error("IO error")]
    IO(#[from] std::io::Error),
    #[error("Invalid HttpMethod")]
    InvalidHttpMethod(String)
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

        let origin_config_key = format!("{CONFIG_ENVIRONMENT_PREFIX}CORS_ALLOWED_ORIGIN");
        let methods_config_key = format!("{CONFIG_ENVIRONMENT_PREFIX}CORS_ALLOWED_METHODS");
        let headers_config_key = format!("{CONFIG_ENVIRONMENT_PREFIX}CORS_ALLOWED_HEADERS");


        let config = match (var(&origin_config_key), var(&methods_config_key), var(&headers_config_key)) {
            (Ok(origin), Ok(methods), Ok(headers)) => {
                Some(
                    CorsConfig {
                        allowed_origin: origin,
                        allowed_methods: methods.split(",")
                            .map(|val| HttpMethod::from_str(val.trim()).unwrap())
                            .collect::<Vec<HttpMethod>>(),
                        allowed_headers: headers.split(",")
                            .map(|val| val.trim().to_string())
                            .collect::<Vec<String>>()
                    }
                )
            }
            (Err(_), Err(_), Err(_)) => {
                None
            }
            _ => {
                tracing::info!("One of the ENV is missing: [{origin_config_key}, {methods_config_key}, {headers_config_key}]");
                panic!("Missing configuration");
            }
        };
        Ok(Config {
            reload_cron: std::env::var(format!("{}RELOAD_CRON", CONFIG_ENVIRONMENT_PREFIX)).unwrap_or("* * * * *".to_string()),
            openapi_urls: configs,
            global_cors: config
        })
    }
}

impl OpenApiConfig {
    pub fn uri(&self) -> Uri {
        Uri::try_from(&self.url).unwrap()
    }
}
