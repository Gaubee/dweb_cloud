use std::{
    collections::HashMap,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use axum::{
    Router,
    body::Body,
    extract::{Path as AxumPath, State},
    http::{Method, Request, Response, StatusCode, Uri, header},
    response::IntoResponse,
    routing::{any, get, post},
};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use clap::Parser;
use dav_server::{DavConfig, DavHandler, body::Body as DavBody, fakels::FakeLs};
use dav_server_opendalfs::OpendalFs;
use dweb_cloud_identity_core::{SignedChallenge, verify_signed_challenge};
use dweb_cloud_storage_core::{AppConfig, ChallengeRecord, FileStore};
use opendal::{Operator, services::Fs};
use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};

#[derive(Debug, Parser)]
struct Args {
    #[arg(long, env = "DWEB_CLOUD_HTTP", default_value = "127.0.0.1:9080")]
    http: SocketAddr,
    #[arg(long, env = "DWEB_CLOUD_DATA_DIR", default_value = "./.data")]
    data_dir: PathBuf,
    #[arg(
        long,
        env = "DWEB_CLOUD_APP_CONFIG",
        default_value = "./config/apps.json"
    )]
    app_config: PathBuf,
}

#[derive(Clone)]
struct AppState {
    store: Arc<FileStore>,
    apps: Arc<HashMap<String, AppConfig>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChallengeRequest {
    public_key_hint: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ChallengeResponse {
    nonce: String,
    server_time_ms: i64,
    expires_at_ms: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TokenIssueRequest {
    public_key_hex: String,
    signature_hex: String,
    app_id: String,
    timestamp_ms: i64,
    device_id: String,
    nonce: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TokenIssueResponse {
    app_id: String,
    webdav_base_url: String,
    username: String,
    password: String,
    expires_at_ms: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ApiError {
    error: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let store = Arc::new(FileStore::new(&args.data_dir)?);
    let apps = FileStore::load_app_configs(Path::new(&args.app_config))?
        .into_iter()
        .map(|app| (app.app_id.clone(), app))
        .collect::<HashMap<_, _>>();
    let state = AppState {
        store,
        apps: Arc::new(apps),
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_headers(Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
            Method::from_bytes(b"PROPFIND")?,
            Method::from_bytes(b"MKCOL")?,
        ]);

    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/api/v1/auth/challenge", post(post_challenge))
        .route("/api/v1/apps/{app_id}/tokens", post(post_issue_token))
        .route("/dav/{app_id}", any(handle_dav_root))
        .route("/dav/{app_id}/{*tail}", any(handle_dav_tail))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(args.http).await?;
    println!("dweb-cloud listening on http://{}", args.http);
    axum::serve(listener, app).await?;
    Ok(())
}

async fn healthz() -> impl IntoResponse {
    axum::Json(serde_json::json!({ "ok": true }))
}

async fn post_challenge(
    State(state): State<AppState>,
    axum::Json(request): axum::Json<ChallengeRequest>,
) -> impl IntoResponse {
    let record = ChallengeRecord {
        nonce: uuid::Uuid::new_v4().to_string(),
        public_key_hint: request.public_key_hint,
        expires_at_ms: now_ms() + 5 * 60 * 1000,
        created_at_ms: now_ms(),
    };
    match state.store.save_challenge(&record) {
        Ok(()) => (
            StatusCode::OK,
            axum::Json(ChallengeResponse {
                nonce: record.nonce,
                server_time_ms: record.created_at_ms,
                expires_at_ms: record.expires_at_ms,
            }),
        )
            .into_response(),
        Err(error) => api_error(StatusCode::INTERNAL_SERVER_ERROR, error.to_string()),
    }
}

async fn post_issue_token(
    State(state): State<AppState>,
    AxumPath(app_id): AxumPath<String>,
    axum::Json(request): axum::Json<TokenIssueRequest>,
) -> impl IntoResponse {
    let Some(app) = state.apps.get(&app_id).cloned() else {
        return api_error(StatusCode::NOT_FOUND, "app not found");
    };
    if request.app_id != app_id {
        return api_error(StatusCode::BAD_REQUEST, "app id mismatch");
    }
    let challenge = match state.store.take_challenge(&request.nonce) {
        Ok(Some(challenge)) => challenge,
        Ok(None) => return api_error(StatusCode::UNAUTHORIZED, "challenge not found"),
        Err(error) => return api_error(StatusCode::INTERNAL_SERVER_ERROR, error.to_string()),
    };
    if challenge.expires_at_ms < now_ms() {
        return api_error(StatusCode::UNAUTHORIZED, "challenge expired");
    }
    let signed = SignedChallenge {
        public_key_hex: request.public_key_hex,
        signature_hex: request.signature_hex,
        app_id: request.app_id,
        timestamp_ms: request.timestamp_ms,
        device_id: request.device_id,
        nonce: request.nonce,
    };
    if let Err(error) = verify_signed_challenge(&signed) {
        return api_error(StatusCode::UNAUTHORIZED, error.to_string());
    }
    let created_at_ms = now_ms();
    let expires_at_ms = created_at_ms + app.token_ttl_secs * 1000;
    match state.store.issue_token(
        &signed.public_key_hex,
        &app_id,
        created_at_ms,
        expires_at_ms,
    ) {
        Ok(issued) => {
            let base = format!("/dav/{app_id}");
            (
                StatusCode::OK,
                axum::Json(TokenIssueResponse {
                    app_id,
                    webdav_base_url: base,
                    username: signed.public_key_hex,
                    password: issued.password,
                    expires_at_ms,
                }),
            )
                .into_response()
        }
        Err(error) => api_error(StatusCode::INTERNAL_SERVER_ERROR, error.to_string()),
    }
}

async fn handle_dav_root(
    State(state): State<AppState>,
    AxumPath(app_id): AxumPath<String>,
    request: Request<Body>,
) -> Response<Body> {
    handle_dav(state, app_id, String::new(), request).await
}

async fn handle_dav_tail(
    State(state): State<AppState>,
    AxumPath((app_id, tail)): AxumPath<(String, String)>,
    request: Request<Body>,
) -> Response<Body> {
    handle_dav(state, app_id, tail, request).await
}

async fn handle_dav(
    state: AppState,
    app_id: String,
    tail: String,
    request: Request<Body>,
) -> Response<Body> {
    if !state.apps.contains_key(&app_id) {
        return plain_error(StatusCode::NOT_FOUND, "app not found");
    }
    let Some((username, password)) = parse_basic_auth(request.headers().get(header::AUTHORIZATION))
    else {
        return unauthorized();
    };
    let token = match state
        .store
        .authenticate_token(&username, &app_id, &password, now_ms())
    {
        Ok(Some(token)) => token,
        Ok(None) => return unauthorized(),
        Err(error) => return plain_error(StatusCode::INTERNAL_SERVER_ERROR, &error.to_string()),
    };

    let app_dir = state
        .store
        .account_app_dir(&token.public_key_hex, &token.app_id);
    if let Err(error) = std::fs::create_dir_all(&app_dir) {
        return plain_error(StatusCode::INTERNAL_SERVER_ERROR, &error.to_string());
    }

    if request.method().as_str() == "MKCOL" {
        return handle_mkcol(&app_dir, &tail);
    }

    let op = match new_fs_operator(&app_dir) {
        Ok(op) => op,
        Err(error) => return plain_error(StatusCode::INTERNAL_SERVER_ERROR, &error),
    };
    let dav_fs = OpendalFs::new(op);
    let handler = DavHandler::builder()
        .filesystem(dav_fs)
        .locksystem(FakeLs::new())
        .build_handler();
    let request = rewrite_dav_request(request, &app_id, &tail);
    let dav_response = handler
        .handle_with(DavConfig::new().principal(username), request)
        .await;
    convert_dav_response(dav_response)
}

fn handle_mkcol(root: &Path, tail: &str) -> Response<Body> {
    if tail.is_empty() {
        return empty_response(StatusCode::METHOD_NOT_ALLOWED);
    }
    let target = match resolve_relative_path(root, tail) {
        Some(path) => path,
        None => return plain_error(StatusCode::BAD_REQUEST, "invalid dav path"),
    };
    if target.exists() {
        return empty_response(StatusCode::METHOD_NOT_ALLOWED);
    }
    let Some(parent) = target.parent() else {
        return plain_error(StatusCode::BAD_REQUEST, "invalid dav path");
    };
    if !parent.exists() {
        return empty_response(StatusCode::CONFLICT);
    }
    match std::fs::create_dir(&target) {
        Ok(()) => empty_response(StatusCode::CREATED),
        Err(error) => plain_error(StatusCode::INTERNAL_SERVER_ERROR, &error.to_string()),
    }
}

fn resolve_relative_path(root: &Path, tail: &str) -> Option<PathBuf> {
    let mut path = root.to_path_buf();
    for segment in tail.split('/') {
        if segment.is_empty() || segment == "." {
            continue;
        }
        if segment == ".." || segment.contains('\\') {
            return None;
        }
        path.push(segment);
    }
    Some(path)
}

fn new_fs_operator(root: &Path) -> Result<Operator, String> {
    let builder = Fs::default().root(root.to_string_lossy().as_ref());
    Operator::new(builder)
        .map(|builder| builder.finish())
        .map_err(|error| error.to_string())
}

fn rewrite_dav_request(request: Request<Body>, app_id: &str, tail: &str) -> Request<Body> {
    let (mut parts, body) = request.into_parts();
    let path = if tail.is_empty() {
        "/".to_string()
    } else {
        format!("/{tail}")
    };
    let query = parts
        .uri
        .query()
        .map(|value| format!("?{value}"))
        .unwrap_or_default();
    let path_and_query = format!("{path}{query}");
    parts.uri = Uri::builder()
        .path_and_query(path_and_query)
        .build()
        .unwrap_or_else(|_| {
            Uri::builder()
                .path_and_query(format!("/dav/{app_id}"))
                .build()
                .unwrap()
        });
    Request::from_parts(parts, body)
}

fn convert_dav_response(response: http::Response<DavBody>) -> Response<Body> {
    let (parts, body) = response.into_parts();
    Response::from_parts(parts, Body::new(body))
}

fn parse_basic_auth(value: Option<&header::HeaderValue>) -> Option<(String, String)> {
    let header = value?.to_str().ok()?;
    let encoded = header.strip_prefix("Basic ")?;
    let decoded = BASE64.decode(encoded.as_bytes()).ok()?;
    let credentials = String::from_utf8(decoded).ok()?;
    let mut parts = credentials.splitn(2, ':');
    let username = parts.next()?.trim().to_string();
    let password = parts.next()?.trim().to_string();
    if username.is_empty() || password.is_empty() {
        return None;
    }
    Some((username, password))
}

fn unauthorized() -> Response<Body> {
    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header(header::WWW_AUTHENTICATE, "Basic realm=\"dwebCloud\"")
        .body(Body::from("please auth"))
        .unwrap()
}

fn empty_response(status: StatusCode) -> Response<Body> {
    Response::builder()
        .status(status)
        .body(Body::from(""))
        .unwrap()
}

fn plain_error(status: StatusCode, text: &str) -> Response<Body> {
    Response::builder()
        .status(status)
        .body(Body::from(text.to_string()))
        .unwrap()
}

fn api_error(status: StatusCode, error: impl Into<String>) -> Response<Body> {
    let payload = serde_json::to_vec(&ApiError {
        error: error.into(),
    })
    .unwrap_or_else(|_| b"{\"error\":\"internal error\"}".to_vec());
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(payload))
        .unwrap()
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0)
}
