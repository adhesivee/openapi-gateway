pub mod openapi;

use regex::Regex;
use crate::config::OpenApiConfig;

#[derive(Debug)]
pub struct GatewayEntry {
    pub config: OpenApiConfig,
    pub openapi_file: Vec<u8>,
    pub routes: Vec<Route>,
}

#[derive(Debug)]
pub struct Route {
    pub uri_regex: Regex,
    pub methods: Vec<String>,
}

impl GatewayEntry {
    pub fn contains_route(&self, path: &str, method: &str) -> bool {

        self.routes.iter()
            .find(|route| {
                route.uri_regex.is_match(path) && route.methods.contains(&method.to_lowercase())
            })
            .is_some()
    }
}

impl From<(Regex, Vec<String>)> for Route {
    fn from(from: (Regex, Vec<String>)) -> Self {
        Route {
            uri_regex: from.0,
            methods: from.1,
        }
    }
}
