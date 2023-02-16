use std::sync::Arc;
use axum::body::Body;
use axum::extract::{Extension, Path};
use axum::http::{HeaderValue, Method, Request, Response, StatusCode, Uri};
use axum::http::header::CONTENT_TYPE;
use axum::Json;
use crate::gateway::GatewayEntry;
use crate::ui::{SwaggerUiConfig, Url};
use crate::web::HttpsClient;

pub async fn swagger_def_handler(
    Extension(entries): Extension<Arc<Vec<GatewayEntry>>>,
    Path(def): Path<String>,
) -> Response<Body> {
    let entry = entries
        .iter()
        .filter(|entry| base64::encode(entry.config.name.clone()) == def)
        .last();

    if let Some(entry) = entry {
        Response::builder()
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(entry.openapi_file.clone()))
            .unwrap()
    } else {
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap()
    }
}

pub async fn swagger_conf_handler(
    Extension(entries): Extension<Arc<Vec<GatewayEntry>>>,
) -> (StatusCode, Json<SwaggerUiConfig>) {
    // @TODO: These files should be proxied

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
    Extension(entries): Extension<Arc<Vec<GatewayEntry>>>,
    Extension(client): Extension<HttpsClient>,
    mut req: Request<Body>,
) -> Response<Body> {
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

        println!("Static file {file}");
        let mut builder = Response::builder().status(StatusCode::OK);

        if file.ends_with(".json") {
            builder = builder.header(CONTENT_TYPE, "application/json");
        }


        let bytes = std::fs::read(format!("swagger-ui/{}", file)).unwrap();
        return builder.body(Body::from(bytes)).unwrap();
    }

    println!("{path_query}");

    let entry = entries
        .iter()
        .filter(|val| {
            val.routes
                .iter()
                .find(|entry| entry.uri == path_query.to_string() && entry.methods.contains(&req.method().to_string().to_lowercase()))
                .is_some()
        })
        .last();

    if let Some(entry) = entry {
        println!("Entry found");
        let entry_uri: &Uri = &entry.config.uri();

        let uri = format!(
            "{}://{}{}",
            entry_uri.scheme_str().unwrap(),
            entry_uri.host().unwrap(),
            path_query
        );

        println!("Forward to {uri}");

        *req.uri_mut() = Uri::try_from(uri).unwrap();
        req.headers_mut().insert("host", HeaderValue::from_str(entry.config.uri().host().unwrap()).unwrap());
        client.request(req).await.unwrap()
    } else {
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap()
    }
}
