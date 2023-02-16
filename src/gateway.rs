pub mod openapi;

use crate::config::OpenApiConfig;
use regex::Regex;

#[derive(Debug)]
pub struct GatewayEntry {
    pub config: OpenApiConfig,
    pub openapi_file: Vec<u8>,
    pub routes: Vec<Route>,
}

#[derive(Debug)]
pub struct Route {
    pub uri_regex: Regex,
    pub method: String,
}

impl GatewayEntry {
    pub fn contains_route(&self, path: &str, method: &str) -> bool {
        self.routes
            .iter()
            .find(|route| {
                route.uri_regex.is_match(path)
                    && route.method.to_lowercase() == method.to_lowercase()
            })
            .is_some()
    }
}
