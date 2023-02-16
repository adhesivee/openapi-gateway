use serde::Deserialize;
use std::collections::BTreeMap;

type PathName = String;
type HttpMethod = String;

#[derive(Clone, Deserialize)]
pub struct OpenApiV3 {
    #[serde(default)]
    pub servers: Vec<Server>,
    pub paths: BTreeMap<PathName, BTreeMap<HttpMethod, PathMethod>>,
}

#[derive(Clone, Deserialize)]
pub struct Server {
    pub url: String,
}

#[derive(Clone, Deserialize)]
pub struct PathMethod {
    #[serde(default)]
    pub parameters: Vec<Parameter>,
}

#[derive(Clone, Deserialize)]
pub struct Parameter {
    pub name: String,
    #[serde(rename = "in")]
    pub in_type: String,
}
