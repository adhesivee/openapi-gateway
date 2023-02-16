use std::collections::BTreeMap;
use serde::Deserialize;

type PathName = String;
type HttpMethod = String;

#[derive(Clone, Deserialize)]
pub struct OpenApiV3 {
    pub servers: Vec<Server>,
    pub paths: BTreeMap<PathName, BTreeMap<HttpMethod, PathMethod>>
}

#[derive(Clone, Deserialize)]
pub struct Server {
    pub url: String
}

#[derive(Clone, Deserialize)]
pub struct PathMethod {

}