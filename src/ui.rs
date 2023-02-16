use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct SwaggerUiConfig {
    pub urls: Vec<Url>,
}

#[derive(Serialize, Debug)]
pub struct Url {
    pub name: String,
    pub url: String,
}
