pub mod openapi;

use crate::config::OpenApiConfig;

#[derive(Debug)]
pub struct GatewayEntry {
    pub config: OpenApiConfig,
    pub openapi_file: Vec<u8>,
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
