use serde::{Deserialize, Serialize};
use std::path::Path;
use toml::de::Error;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub openapi_urls: Vec<OpenApiUrl>,
}

#[derive(Deserialize, Debug)]
pub struct OpenApiUrl {
    pub name: String,
    pub url: String,
}

pub fn read_from_file<P: AsRef<Path>>(path: P) -> Result<Config, Error> {
    toml::from_str(
        std::fs::read_to_string(path)
            .expect("{path} not found")
            .as_str(),
    )
}
