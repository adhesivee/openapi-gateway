pub mod openapi;

use axum::http::Uri;

#[derive(Debug)]
pub struct OpenApiEntry {
    pub uri: Uri,
    pub name: String,
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
