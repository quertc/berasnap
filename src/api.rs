use axum::{
    extract::{Path, State},
    http::Request,
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};
use chrono::{DateTime, Utc};
use log::info;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tower_http::services::ServeFile;
use tower_http::{timeout::TimeoutLayer, trace::TraceLayer};

const API_TIMEOUT_SECS: u64 = 30;

#[derive(Debug, Serialize, Deserialize)]
pub struct Snapshot {
    file_name: String,
    sha256: String,
    #[serde(rename = "type")]
    node_type: String,
    upload_time: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    snapshots: Vec<Snapshot>,
}

#[derive(Clone)]
pub struct AppState {
    storage_path: Arc<String>,
}

async fn list_snapshots(State(state): State<AppState>) -> impl IntoResponse {
    let metadata_path = format!("{}/metadata.json", state.storage_path);
    match tokio::fs::read_to_string(metadata_path).await {
        Ok(content) => {
            let metadata: Metadata =
                serde_json::from_str(&content).unwrap_or(Metadata { snapshots: vec![] });
            (StatusCode::OK, axum::Json(metadata))
        }
        Err(_) => (StatusCode::OK, axum::Json(Metadata { snapshots: vec![] })),
    }
}

async fn download_snapshot(
    Path(filename): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let file_path = format!("{}/{}", state.storage_path, filename);
    let req = Request::new(axum::body::Body::empty());
    ServeFile::new(file_path).try_call(req).await.unwrap()
}

pub async fn run_api_server(storage_path: String, port: u16) -> anyhow::Result<()> {
    let app_state = AppState {
        storage_path: Arc::new(storage_path),
    };

    let app = Router::new()
        .route("/snapshots", get(list_snapshots))
        .route("/snapshots/:filename", get(download_snapshot))
        .layer((
            TraceLayer::new_for_http(),
            TimeoutLayer::new(Duration::from_secs(API_TIMEOUT_SECS)),
        ))
        .with_state(app_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    info!("Listening on {}", addr);
    let listener = TcpListener::bind(addr).await.unwrap();

    axum::serve(listener, app).await.unwrap();

    Ok(())
}
