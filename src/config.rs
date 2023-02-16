use axum::http::Uri;
use serde::Deserialize;
use std::path::Path;
use toml::de::Error;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub openapi_urls: Vec<OpenApiConfig>,
}

#[derive(Deserialize, Debug)]
pub struct OpenApiConfig {
    pub name: String,
    pub url: String,
}

impl Config {
    pub fn parse_from_file<P: AsRef<Path>>(path: P) -> Result<Config, Error> {
        toml::from_str(
            std::fs::read_to_string(path)
                .expect("{path} not found")
                .as_str(),
        )
    }
}

impl OpenApiConfig {
    pub fn uri(&self) -> Uri {
        Uri::try_from(&self.url).unwrap()
    }
}
