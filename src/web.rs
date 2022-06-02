mod handler;

use std::convert::Infallible;
use std::error::Error;
use crate::web::handler::{gateway_handler, swagger_conf_handler, swagger_def_handler};
use crate::RwGatewayEntries;
use axum::body::{Body, Bytes, StreamBody};
use axum::http::{HeaderMap, Request, Uri};
use axum::routing::{any, get};
use axum::{AddExtensionLayer, Router};
use hyper::client::HttpConnector;
use hyper_rustls::HttpsConnector;
use std::net::SocketAddr;
use axum::response::{IntoResponse, Response};
use tokio::io::AsyncReadExt;
use tokio::time::{Duration, sleep};

pub type HttpsClient = hyper::client::Client<HttpsConnector<HttpConnector>, Body>;

pub fn new_https_client() -> HttpsClient {
    let https = hyper_rustls::HttpsConnectorBuilder::new()
        .with_webpki_roots()
        .https_only()
        .enable_http1()
        .build();

    hyper::Client::builder().build(https)
}

pub async fn simple_get(client: &HttpsClient, uri: &Uri) -> Result<(HeaderMap, Bytes), HttpError> {
    let response = client
        .request(Request::get(uri).body(Body::empty()).unwrap())
        .await;

    match response {
        Ok(response) => {
            let headers = response.headers().clone();

            let bytes = hyper::body::to_bytes(response.into_body()).await.unwrap();

            Ok((headers, bytes))
        }
        Err(err) => { Err(HttpError::Error(err)) }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum HttpError {
    #[error("Some error")]
    Error(#[from] hyper::Error)
}

pub async fn serve_with_config(client: HttpsClient, entries: RwGatewayEntries) {
    let app = Router::new()
        .route("/docs/swagger-config.json", get(swagger_conf_handler))
        .route("/docs/defs/:def", get(swagger_def_handler))
        .fallback(any(gateway_handler))
        .layer(AddExtensionLayer::new(client))
        .layer(AddExtensionLayer::new(entries));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!("gateway proxy listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
