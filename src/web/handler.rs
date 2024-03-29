use crate::ui::{SwaggerUiConfig, Url};
use crate::web::HttpClient;
use crate::{GatewayEntry, RwGatewayEntries};
use axum::body::Body;
use axum::extract::{State, Path};
use axum::http::header::CONTENT_TYPE;
use axum::http::{HeaderValue, Method, Request, Response, StatusCode, Uri};
use axum::Json;
use crate::config::CorsConfig;

pub async fn swagger_def_handler(
    State(entries): State<RwGatewayEntries>,
    Path(def): Path<String>,
) -> Response<Body> {
    let entries = entries.read().await;

    let entry = entries
        .iter()
        .filter(|entry| base64::encode(entry.config.name.clone()) == def)
        .last();

    if let Some(entry) = entry {
        if let Some(openapi_file) = &entry.openapi_file {
            Response::builder()
                .status(StatusCode::OK)
                .header(CONTENT_TYPE, openapi_file.content_type.clone())
                .body(Body::from(entry.openapi_file.as_ref().unwrap().contents.clone()))
                .unwrap()
        } else {
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::empty())
                .unwrap()
        }
    } else {
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap()
    }
}

pub async fn swagger_conf_handler(
    State(entries): State<RwGatewayEntries>,
) -> (StatusCode, Json<SwaggerUiConfig>) {
    let entries = entries.read().await;
    let config = SwaggerUiConfig {
        urls: entries
            .iter()
            .map(|entry| Url {
                name: entry.config.name.clone(),
                url: format!("/docs/defs/{}", base64::encode(entry.config.name.clone())),
            })
            .collect(),
    };

    (StatusCode::OK, Json(config))
}

pub async fn gateway_handler(
    State(entries): State<RwGatewayEntries>,
    State(global_cors_config): State<Option<CorsConfig>>,
    State(client): State<HttpClient>,
    mut req: Request<Body>,
) -> Response<Body> {
    let entries = entries.read().await;

    let path = req.uri().path();
    let path_query = req
        .uri()
        .path_and_query()
        .map(|v| v.as_str())
        .unwrap_or(path);

    if req.method() == Method::GET && path.starts_with("/docs") {
        let mut file = &path[5..].trim_start_matches("/");

        if file.starts_with(".") || file.starts_with("..") {
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::empty())
                .unwrap();
        }

        if file.is_empty() {
            file = &"index.html";
        }

        let mut builder = Response::builder().status(StatusCode::OK);

        if file.ends_with(".json") {
            builder = builder.header(CONTENT_TYPE, "application/json");
        }

        let exe_file = std::env::current_exe();
        let exe_folder = exe_file
            .as_ref()
            .map(|val| val.as_path().parent());


        let default = format!("swagger-ui/{}", file);
        let file_path = if let Ok(Some(path)) = exe_folder {
            let swagger_ui_folder = path.join("swagger-ui");

            if swagger_ui_folder.exists() {
                swagger_ui_folder.join(file)
                    .to_str()
                    .unwrap()
                    .to_string()
            } else {
                default
            }
        } else {
            default
        };

        tracing::info!("Loading from disk: {file_path}");
        let content_type = if let Some(split) = file_path.rsplit_once(".") {
            match split.1 {
                "css" => "text/css",
                "js" => "text/javascript",
                "html" => "text/html",
                "png" => "image/png",
                "json" => "application/json",
                "yml" => "application/yaml",
                "yaml" => "application/yaml",
                _ => "text/plain"
            }
        } else {
            "text/plain"
        };

        let bytes = std::fs::read(file_path).unwrap();
        return builder
            .header(CONTENT_TYPE, content_type)
            .body(Body::from(bytes))
            .unwrap();
    }

    let entry = matching_route_with_least_matching_parameters(
        path,
        req.method().as_str(),
        &*entries
    );

    let req = if let Some(entry) = entry {
        let entry_uri: &Uri = &entry.config.uri();

        let uri = if let Some(port) = entry_uri.port_u16() {
            format!(
                "{}://{}:{}{}",
                entry_uri.scheme_str().unwrap(),
                entry_uri.host().unwrap(),
                port,
                path_query
            )
        } else {
            format!(
                "{}://{}{}",
                entry_uri.scheme_str().unwrap(),
                entry_uri.host().unwrap(),
                path_query
            )
        };

        *req.uri_mut() = Uri::try_from(uri).unwrap();
        req.headers_mut().insert(
            "host",
            HeaderValue::from_str(entry.config.uri().host().unwrap()).unwrap(),
        );

        req
    } else {
        let entry = entries
            .iter()
            .filter(|val| val.contains_route(path))
            .last();

        // If route is found and cors config is available create OK response
        if let (Some(_), Some(global_cors_config)) = (entry, global_cors_config) {
            let mut response = Response::builder()
                .status(StatusCode::OK)
                .body(Body::empty())
                .unwrap();

            // @TODO: Duplicated code, fix
            let mut headers = response.headers_mut();
            headers.insert("Access-Control-Allow-Origin", HeaderValue::from_str(&global_cors_config.allowed_origin).unwrap());
            headers.insert(
                "Access-Control-Allow-Methods",
                HeaderValue::from_str(
                    &global_cors_config.allowed_methods
                        .iter()
                        .map(|method| method.into())
                        .collect::<Vec<&str>>()
                        .join(", ")

                ).unwrap()
            );
            headers.insert("Access-Control-Allow-Headers", HeaderValue::from_str(&global_cors_config.allowed_headers.join(", ")).unwrap());

            return response;
        } else {
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::empty())
                .unwrap()
        }
    };

    // Make sure to free up read lock
    drop(entries);

    let mut response: Response<Body> = client.request(req).await.unwrap();

    if let Some(global_cors_config) = global_cors_config {
        let mut headers = response.headers_mut();
        headers.insert("Access-Control-Allow-Origin", HeaderValue::from_str(&global_cors_config.allowed_origin).unwrap());
        headers.insert(
            "Access-Control-Allow-Methods",
            HeaderValue::from_str(
                &global_cors_config.allowed_methods
                    .iter()
                    .map(|method| method.into())
                    .collect::<Vec<&str>>()
                    .join(", ")

            ).unwrap()
        );
        headers.insert("Access-Control-Allow-Headers", HeaderValue::from_str(&global_cors_config.allowed_headers.join(", ")).unwrap());
    }

    response
}

fn matching_route_with_least_matching_parameters<'a>(
    path: &str,
    method: &str,
    entries: &'a Vec<GatewayEntry>
) -> Option<&'a GatewayEntry> {
    entries
        .iter()
        .filter_map(|entry| {
            let route = entry.routes.iter()
                .filter(|route| route.uri_regex.is_match(path))
                .filter(|route| route.method.to_lowercase() == method.to_lowercase())
                .min_by(|left, right| left.path_parameters.len().cmp(&right.path_parameters.len()));

            match route {
                Some(route) => Some((route, entry)),
                None => None
            }
        })
        .min_by(|left, right| left.0.path_parameters.len().cmp(&right.0.path_parameters.len()))
        .map(|route_entry| route_entry.1)
}

#[cfg(test)]
mod tests {
    use regex::{escape, Regex};
    use crate::{GatewayEntry, OpenApiConfig};
    use crate::gateway::Route;
    use crate::openapi::Parameter;
    use crate::web::handler::matching_route_with_least_matching_parameters;

    #[test]
    fn route_with_least_parameters() {
        let entry1 = GatewayEntry {
            config: OpenApiConfig {
                name: "entry1".to_string(),
                url: "".to_string()
            },
            openapi_file: None,
            routes: vec![
                Route {
                    uri_regex: Regex::new(&escape("/foo/bar")).unwrap(),
                    method: "GET".to_string(),
                    path_parameters: vec![]
                },
            ]
        };

        let entry2 = GatewayEntry {
            config: OpenApiConfig {
                name: "entry2".to_string(),
                url: "".to_string()
            },
            openapi_file: None,
            routes: vec![
                Route {
                    uri_regex: Regex::new("/foo/.*").unwrap(),
                    method: "GET".to_string(),
                    path_parameters: vec![
                        Parameter {
                            name: "par".to_string(),
                            in_type: "path".to_string()
                        }
                    ]
                },
            ]
        };

        let entries = vec![entry1, entry2];

        let route = matching_route_with_least_matching_parameters(
            "/foo/bar",
            "GET",
            &entries
        );

        assert!(route.is_some());
        assert_eq!("entry1", route.unwrap().config.name);


        let route = matching_route_with_least_matching_parameters(
            "/foo/not-bar",
            "GET",
            &entries
        );

        assert!(route.is_some());
        assert_eq!("entry2", route.unwrap().config.name);
    }
}