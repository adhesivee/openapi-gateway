mod handler;

use std::net::SocketAddr;
use std::sync::Arc;
use axum::body::{Body, Bytes};
use axum::http::{Request, Uri};
use axum::{AddExtensionLayer, Router};
use axum::routing::{any, get};
use hyper::client::HttpConnector;
use hyper_rustls::HttpsConnector;
use crate::gateway::GatewayEntry;
use crate::web::handler::{gateway_handler, swagger_conf_handler, swagger_def_handler};

pub type HttpsClient = hyper::client::Client<HttpsConnector<HttpConnector>, Body>;

pub fn new_https_client() -> HttpsClient {
    let https = hyper_rustls::HttpsConnectorBuilder::new()
        .with_webpki_roots()
        .https_only()
        .enable_http1()
        .build();

    hyper::Client::builder().build(https)
}

pub async fn get_bytes(client: &HttpsClient, uri: &Uri) -> Bytes {
    let response = client
        .request(Request::get(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();

    hyper::body::to_bytes(response.into_body()).await.unwrap()
}

pub async fn serve_with_config(client: HttpsClient, entries: Vec<GatewayEntry>) {

    let app = Router::new()
        .route("/docs/swagger-config.json", get(swagger_conf_handler))
        .route("/docs/defs/:def", get(swagger_def_handler))
        .fallback(any(gateway_handler))
        .layer(AddExtensionLayer::new(client))
        .layer(AddExtensionLayer::new(Arc::new(entries)));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("reverse proxy listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}