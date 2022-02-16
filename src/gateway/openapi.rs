use crate::config::OpenApiConfig;
use crate::gateway::{GatewayEntry, OpenApiFile, Route};
use crate::openapi::{OpenApiV3, Parameter, Server};
use regex::{escape, Regex};
use std::str::FromStr;

pub enum ContentType {
    JSON,
    YAML
}

pub fn parse_openapi(
    content_type: ContentType,
    config: OpenApiConfig,
    buffer: &[u8]
) -> GatewayEntry {
    let (content_type, mut document): (&'static str, OpenApiV3) = match content_type {
        ContentType::JSON => { ("application/json", serde_json::from_slice(buffer).unwrap()) }
        ContentType::YAML => { ("application/yaml", serde_yaml::from_slice(buffer).unwrap()) }
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

    GatewayEntry {
        config,
        openapi_file: Some(OpenApiFile {
            content_type: content_type.to_string(),
            contents: buffer.to_vec()

        }),
        routes,
    }
}

fn collect_routes(json: &OpenApiV3, server_prefix: &str) -> Vec<Route> {
    json.paths
        .iter()
        .map(|path| {
            let path_uri = format!("{}{}", server_prefix, path.0);

            path.1
                .iter()
                .map(|method| {
                    let regex = regex_from_route(&path_uri, &method.1.parameters);

                    Route {
                        uri_regex: regex,
                        method: method.0.clone(),
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
            pattern.replace(&format!("\\{{{}\\}}", param.name), "[^/]*")
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
