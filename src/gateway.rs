pub mod openapi;

use axum::http::Uri;
use crate::config::OpenApiConfig;

#[derive(Debug)]
pub struct OpenApiEntry {
    pub config: OpenApiConfig,
    pub routes: Vec<Route>,
}

#[derive(Debug)]
pub struct Route {
    pub uri: String,
    pub methods: Vec<String>,
}

impl From<(String, Vec<String>)> for Route {
    fn from(from: (String, Vec<String>)) -> Self {
        Route {
            uri: from.0,
            methods: from.1,
        }
    }
}
