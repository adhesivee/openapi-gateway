mod handler;

use std::convert::Infallible;
use std::error::Error;
use crate::web::handler::{gateway_handler, swagger_conf_handler, swagger_def_handler};
use crate::RwGatewayEntries;
use axum::body::{Body, Bytes, StreamBody};
use axum::http::{HeaderMap, Request, StatusCode, Uri};
use axum::routing::{any, get, get_service};
use axum::Router;
use hyper::client::HttpConnector;
use hyper_rustls::HttpsConnector;
use std::net::SocketAddr;
use axum::response::{IntoResponse, Response};
use tokio::io::AsyncReadExt;
use tokio::time::{Duration, sleep};
use std::sync::Arc;
use axum::http::uri::Scheme;
use axum_macros::FromRef;
use tokio::io;
use tower_http::services::{ServeDir, ServeFile};
use crate::config::CorsConfig;

type HyperHttpsClient = hyper::client::Client<HttpsConnector<HttpConnector>, Body>;
type HyperHttpClient = hyper::client::Client<HttpConnector, Body>;

#[derive(Clone)]
pub struct HttpClient {
    https_client: HyperHttpsClient,
    http_client: HyperHttpClient
}

impl HttpClient {
    pub fn new() -> Self {
        Self {
            https_client: new_hyper_https_client(),
            http_client: Default::default()
        }
    }

    pub async fn request(&self, req: Request<Body>) -> hyper::Result<Response<Body>> {
        let scheme = req.uri()
            .scheme_str()
            .unwrap_or("http");

        match scheme {
            "http" => {
                self.http_client.request(req).await
            }
            "https" => {
                self.https_client.request(req).await
            }
            _ => panic!("Unknown")
        }
    }
}

fn new_hyper_https_client() -> HyperHttpsClient {
    let https = hyper_rustls::HttpsConnectorBuilder::new()
        .with_webpki_roots()
        .https_only()
        .enable_http1()
        .build();

    hyper::Client::builder().build(https)
}

pub async fn simple_get(client: &HttpClient, uri: &Uri) -> Result<(HeaderMap, Bytes), HttpError> {
    let scheme = uri.scheme_str()
        .unwrap_or("http");

    let response = match scheme {
        "http" => {
            client
                .http_client
                .request(Request::get(uri).body(Body::empty()).unwrap())
                .await
        }
        "https" => {
            client
                .https_client
                .request(Request::get(uri).body(Body::empty()).unwrap())
                .await
        }
        _ => panic!("Unknown")
    };

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

#[derive(Clone, FromRef)]
struct AppState {
    client: HttpClient,
    entries: RwGatewayEntries,
    global_cors_config: Option<CorsConfig>
}
pub async fn serve_with_config(
    client: HttpClient,
    entries: RwGatewayEntries,
    global_cors_config: Option<CorsConfig>
) {
    let serve_dir = ServeDir::new("redoc").not_found_service(ServeFile::new("redoc/index.html"));
    let serve_dir = get_service(serve_dir).handle_error(handle_error);

    let app = Router::new()
        .route("/docs/swagger-config.json", get(swagger_conf_handler))
        .route("/docs/defs/:def", get(swagger_def_handler))
        .nest_service("/redoc/", serve_dir)
        .fallback(gateway_handler)
        .with_state(AppState { client, entries, global_cors_config })
        ;

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!("gateway proxy listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handle_error(_err: io::Error) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong...")
}