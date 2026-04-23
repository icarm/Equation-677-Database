use axum::{
    extract::DefaultBodyLimit,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use eq677::{Magma, MatrixMagma};
use serde::{Deserialize, Serialize};
use tokio::task;

#[derive(Deserialize)]
struct CanonReq {
    table: Vec<Vec<usize>>,
}

#[derive(Serialize)]
struct CanonResp {
    canonical: String,
    is255: bool,
}

async fn canonicalize(
    Json(req): Json<CanonReq>,
) -> Result<Json<CanonResp>, (StatusCode, String)> {
    let n = req.table.len();
    if n == 0 {
        return Err((StatusCode::BAD_REQUEST, "empty table".into()));
    }
    for (i, row) in req.table.iter().enumerate() {
        if row.len() != n {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("row {i} has length {}, expected {n}", row.len()),
            ));
        }
        for (j, &v) in row.iter().enumerate() {
            if v >= n {
                return Err((
                    StatusCode::BAD_REQUEST,
                    format!("table[{i}][{j}] = {v} not in [0, {n})"),
                ));
            }
        }
    }

    let result = task::spawn_blocking(move || {
        let m = MatrixMagma::by_fn(n, |x, y| req.table[x][y]);
        let canon = m.canonicalize2();
        let canonical = canon.to_string();
        let is255 = canon.is255();
        CanonResp { canonical, is255 }
    })
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("canonicalize task failed: {e}"),
        )
    })?;

    Ok(Json(result))
}

async fn health() -> &'static str {
    "ok"
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let app = Router::new()
        .route("/health", get(health))
        .route("/canonicalize", post(canonicalize))
        .layer(DefaultBodyLimit::max(32 * 1024 * 1024));

    let addr = "0.0.0.0:8080";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::info!("listening on {addr}");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };
    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
