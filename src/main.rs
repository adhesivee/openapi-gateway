mod config;
mod gateway;
mod ui;

use crate::config::read_from_file;
use crate::gateway::openapi::build_from_json;
use crate::gateway::{OpenApiEntry, Route};
use crate::ui::{SwaggerUiConfig, Url};
use axum::body::{Full, HttpBody};
use axum::http::header::CONTENT_TYPE;
use axum::http::{HeaderValue, Method, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{any, get_service, MethodRouter};
use axum::{extract::Extension, http::{uri::Uri, Request, Response}, routing::get, AddExtensionLayer, Router, Json};
use hyper::{client::HttpConnector, Body};
use hyper_rustls::HttpsConnector;
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::convert::Infallible;
use std::future::Future;
use std::sync::Arc;
use std::{convert::TryFrom, net::SocketAddr};
use axum::extract::Path;
use tower::util::AndThen;
use tower::{ServiceBuilder, ServiceExt};

type Client = hyper::client::Client<HttpsConnector<HttpConnector>, Body>;

const CONFIG_FILE: &str = "openapi-gateway-config.toml";

#[tokio::main]
async fn main() {
    let https = hyper_rustls::HttpsConnectorBuilder::new()
        .with_webpki_roots()
        .https_only()
        .enable_http1()
        .build();

    let client: Client = hyper::Client::builder().build(https);

    let config = read_from_file(CONFIG_FILE).unwrap();

    let mut entries = vec![];
    for config in config.openapi_urls.into_iter() {
        let response = client
            .request(Request::get(&config.uri()).body(Body::empty()).unwrap())
            .await
            .unwrap();

        let bytes = hyper::body::to_bytes(response.into_body()).await.unwrap();

        entries.push(build_from_json(config, &bytes));
    }

    let app = Router::new()
        .route("/docs/swagger-config.json", get(swagger_conf_handler))
        .route("/docs/defs/:def", get(swagger_def_handler))
        .fallback(any(handler))
        .layer(AddExtensionLayer::new(client))
        .layer(AddExtensionLayer::new(Arc::new(entries)));

    let addr = SocketAddr::from(([0, 0, 0, 0], 4000));
    println!("reverse proxy listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn swagger_def_handler(
    Extension(entries): Extension<Arc<Vec<OpenApiEntry>>>,
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

async fn swagger_conf_handler(
    Extension(entries): Extension<Arc<Vec<OpenApiEntry>>>,
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

async fn handler(
    Extension(entries): Extension<Arc<Vec<OpenApiEntry>>>,
    Extension(client): Extension<Client>,
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
            let routes_strs: Vec<_> = val.routes.iter().map(|val| val.uri.clone()).collect();

            routes_strs.contains(&path_query.to_string())
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
