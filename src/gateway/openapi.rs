use crate::config::OpenApiConfig;
use crate::gateway::{GatewayEntry, OpenApiFile, Route};
use crate::openapi::{OpenApiV3, Parameter, Server};
use regex::{escape, Regex};
use std::str::FromStr;
use serde_json::{Value as JsonValue, Value};
use serde_yaml::Value as YmlValue;

#[derive(Debug)]
pub enum ContentType {
    JSON,
    YAML
}

#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error("JSON Parse error")]
    JsonParseError(#[from] serde_json::Error),
    #[error("YAML Parse error")]
    YamlParseError(#[from] serde_yaml::Error),
}
pub fn parse_openapi(
    content_type: ContentType,
    config: OpenApiConfig,
    file_buffer: &[u8]
) -> Result<GatewayEntry, ParseError> {
    let mut buffer = file_buffer.to_vec();
    let (content_type, mut document): (&'static str, OpenApiV3) = match content_type {
        ContentType::JSON => {
            let mut value: JsonValue = serde_json::from_slice(buffer.as_slice())?;
            let value_map = value.as_object_mut().unwrap();
            // Remove servers as it has to go through this application
            value_map.remove("servers");

            buffer = serde_json::to_vec(&value).unwrap();
            ("application/json", serde_json::from_slice(&buffer)?)
        }
        ContentType::YAML => {
            let mut value: YmlValue = serde_yaml::from_slice(buffer.as_slice())?;
            let value_map = value.as_mapping_mut().unwrap();
            // Remove servers as it has to go through this application
            value_map.remove(&serde_yaml::Value::String("servers".to_string()));

            buffer = serde_json::to_vec(&value).unwrap();

            ("application/yaml", serde_yaml::from_slice(&buffer)?)
        }
    };

    if document.servers.is_empty() {
        document.servers.push(Server {
            url: "/".to_string(),
        })
    }

    let routes: Vec<_> = document
        .servers
        .iter()
        .map(|server| {
            let server_prefix = server.url.trim_end_matches("/");

            let routes = collect_routes(&document, server_prefix);

            routes.iter()
                .for_each(|route| {
                    tracing::info!("Register route: {} {}", route.method, route.uri_regex.as_str())
                });

            routes
        })
        .flatten()
        .collect();

    Ok(GatewayEntry {
        config,
        openapi_file: Some(OpenApiFile {
            content_type: content_type.to_string(),
            contents: buffer.to_vec()

        }),
        routes,
    })
}

fn collect_routes(json: &OpenApiV3, server_prefix: &str) -> Vec<Route> {
    json.paths
        .iter()
        .map(|path| {
            let path_uri = format!("{}{}", server_prefix, path.0);

            path.1
                .methods
                .iter()
                .map(|method| {
                    let parameters = method.1
                        .parameters
                        .as_ref()
                        .unwrap_or(&path.1.parameters);

                    let regex = regex_from_route(&path_uri, parameters);

                    Route {
                        uri_regex: regex,
                        method: method.0.clone(),
                        path_parameters: parameters.iter()
                            .filter(|param| param.in_type == "path")
                            .map(|param| param.clone())
                            .collect::<Vec<_>>()
                    }
                })
                .collect::<Vec<_>>()
        })
        .flatten()
        .collect()
}

fn regex_from_route(url: &str, parameters: &Vec<Parameter>) -> Regex {
    let pattern = parameters
        .iter()
        .filter(|param| param.in_type == "path")
        .fold(escape(url), |pattern, param| {
            pattern.replace(&format!("\\{{{}\\}}", param.name.replace("-", "\\-")), "[^/]*")
        });

    Regex::from_str(&format!("^{}$", &pattern)).unwrap()
}

#[cfg(test)]
mod tests {
    use crate::gateway::openapi::regex_from_route;
    use crate::openapi::Parameter;

    #[test]
    fn test_valid_regex() {
        let url = "/v1/users/{user_id}";

        let regex = regex_from_route(
            url,
            &vec![Parameter {
                name: "user_id".to_string(),
                in_type: "path".to_string(),
            }],
        );

        eprintln!("^{}$", regex.as_str());
        assert_eq!(regex.as_str(), "^/v1/users/[^/]*$");
    }

    #[test]
    fn test_regex_for_validation() {
        let url = "/v1/users/{user_id}";

        let regex = regex_from_route(
            url,
            &vec![Parameter {
                name: "user_id".to_string(),
                in_type: "path".to_string(),
            }],
        );

        assert!(regex.is_match("/v1/users/123"));
    }

    #[test]
    fn test_regex_for_validation_with_suffix() {
        let url = "/v1/users/{user_id}-suffix/subroute";

        let regex = regex_from_route(
            url,
            &vec![Parameter {
                name: "user_id".to_string(),
                in_type: "path".to_string(),
            }],
        );

        assert!(regex.is_match("/v1/users/123-suffix/subroute"));
    }
}
