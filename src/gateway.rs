pub mod openapi;

use crate::config::OpenApiConfig;
use regex::Regex;
use crate::openapi::Parameter;

#[derive(Debug)]
pub struct GatewayEntry {
    pub config: OpenApiConfig,
    pub openapi_file: Option<OpenApiFile>,
    pub routes: Vec<Route>,
}

#[derive(Debug)]
pub struct OpenApiFile {
    pub content_type: String,
    pub contents: Vec<u8>
}

#[derive(Debug)]
pub struct Route {
    pub uri_regex: Regex,
    pub method: String,
    pub path_parameters: Vec<Parameter>
}

impl GatewayEntry {
    pub fn contains_route_and_method(&self, path: &str, method: &str) -> bool {
        self.routes
            .iter()
            .find(|route| {
                route.uri_regex.is_match(path)
                    && route.method.to_lowercase() == method.to_lowercase()
            })
            .is_some()
    }

    pub fn contains_route(&self, path: &str) -> bool {
        self.routes
            .iter()
            .find(|route| route.uri_regex.is_match(path))
            .is_some()
    }
}

impl Route {
    fn new(uri_regex: Regex, method: String) -> Route {
        Self { uri_regex, method, path_parameters: vec![] }
    }
}
#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use regex::Regex;
    use crate::config::OpenApiConfig;
    use crate::gateway::{GatewayEntry, OpenApiFile, Route};

    #[test]
    fn test_no_match_on_method() {
        let entry = entry_with_route(
            vec![
                Route::new(Regex::from_str(".*").unwrap(),"put".to_string()),
                Route::new(Regex::from_str(".*").unwrap(),"post".to_string()),
            ]
        );

        assert!(!entry.contains_route_and_method("/test", "get"))
    }

    #[test]
    fn test_match_route_and_method() {
        let entry = entry_with_route(
            vec![
                Route::new(Regex::from_str(".*").unwrap(),"put".to_string()),
                Route::new(Regex::from_str(".*").unwrap(),"post".to_string()),
                Route::new(Regex::from_str(".*").unwrap(),"get".to_string()),
            ]
        );

        assert!(entry.contains_route_and_method("/test", "get"))
    }

    fn entry_with_route(routes: Vec<Route>) -> GatewayEntry {
        GatewayEntry {
            config: OpenApiConfig { name: "".to_string(), url: "".to_string() },
            openapi_file: Some(
                OpenApiFile {
                    content_type: "".to_string(),
                    contents: vec![]
                }
            ),
            routes
        }
    }
}