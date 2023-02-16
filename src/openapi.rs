use serde::Deserialize;
use std::collections::BTreeMap;

type PathName = String;
type HttpMethod = String;

#[derive(Clone, Deserialize, Debug)]
pub struct OpenApiV3 {
    #[serde(default)]
    pub servers: Vec<Server>,
    pub paths: BTreeMap<PathName, Path>,
}

#[derive(Clone, Deserialize, Debug)]
pub struct Server {
    pub url: String,
}

#[derive(Clone, Deserialize, Debug)]
pub struct Path {
    #[serde(default)]
    pub parameters: Vec<Parameter>,
    #[serde(flatten)]
    pub methods: BTreeMap<HttpMethod, PathMethod>
}

#[derive(Clone, Deserialize, Debug)]
pub struct PathMethod {
    pub parameters: Option<Vec<Parameter>>,
}

#[derive(Clone, Deserialize, Debug)]
pub struct Parameter {
    pub name: String,
    #[serde(rename = "in")]
    pub in_type: String,
}
