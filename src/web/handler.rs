use crate::ui::{SwaggerUiConfig, Url};
use crate::web::HttpsClient;
use crate::RwGatewayEntries;
use axum::body::Body;
use axum::extract::{Extension, Path};
use axum::http::header::CONTENT_TYPE;
use axum::http::{HeaderValue, Method, Request, Response, StatusCode, Uri};
use axum::Json;

pub async fn swagger_def_handler(
    Extension(entries): Extension<RwGatewayEntries>,
    Path(def): Path<String>,
) -> Response<Body> {
    let entries = entries.read().await;

    let entry = entries
        .iter()
        .filter(|entry| base64::encode(entry.config.name.clone()) == def)
        .last();

    if let Some(entry) = entry {
        Response::builder()
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, &entry.openapi_file.as_ref().unwrap().content_type)
            .body(Body::from(entry.openapi_file.as_ref().unwrap().contents.clone()))
            .unwrap()
    } else {
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap()
    }
}

pub async fn swagger_conf_handler(
    Extension(entries): Extension<RwGatewayEntries>,
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
    Extension(entries): Extension<RwGatewayEntries>,
    Extension(client): Extension<HttpsClient>,
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
            .map(|val| val.as_path())
            .map(|val| val.parent().unwrap());


        let default = format!("swagger-ui/{}", file);
        let file_path = if let Ok(path) = exe_folder {
            let swagger_ui_folder = path.join("swagger-ui");

            if swagger_ui_folder.exists() {
                path.join(file)
                    .to_str()
                    .unwrap()
                    .to_string()
            } else {
                default
            }
        } else {
            default
        };

        let bytes = std::fs::read(file_path).unwrap();
        return builder.body(Body::from(bytes)).unwrap();
    }

    let entry = entries
        .iter()
        .filter(|val| val.contains_route(path, req.method().as_str()))
        .last();

    if let Some(entry) = entry {
        let entry_uri: &Uri = &entry.config.uri();

        let uri = format!(
            "{}://{}{}",
            entry_uri.scheme_str().unwrap(),
            entry_uri.host().unwrap(),
            path_query
        );

        *req.uri_mut() = Uri::try_from(uri).unwrap();
        req.headers_mut().insert(
            "host",
            HeaderValue::from_str(entry.config.uri().host().unwrap()).unwrap(),
        );
        client.request(req).await.unwrap()
    } else {
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap()
    }
}
