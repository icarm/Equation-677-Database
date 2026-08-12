use axum::{
    extract::DefaultBodyLimit,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::sync::Semaphore;

#[derive(Serialize, Deserialize)]
struct CanonReq {
    table: Vec<Vec<usize>>,
}

#[derive(Serialize, Deserialize)]
struct CanonResp {
    canonical: String,
    is255: bool,
    // π such that canonical element k corresponds to input element perm[k].
    // The caller can invert this to obtain a display_reorder that reproduces
    // the submitter's original labeling when applied to the canonical form.
    perm: Vec<usize>,
}

// Canonicalization runs in a child process (this same binary, re-executed with
// --worker) so that a pathological input — some highly symmetric magmas send
// nauty/Traces into unbounded memory growth — is contained by an address-space
// rlimit and a wall-clock timeout instead of OOM-killing the whole container.
// Default fits a standard-2 instance (6 GiB), leaving ~512 MiB of headroom
// for the server process and container overhead.
const DEFAULT_MEM_LIMIT_BYTES: u64 = 5632 * 1024 * 1024;
// The slowest known-legitimate submissions (size-961 magmas) take ~25 s on a
// fast desktop core; container vCPUs are slower, so allow several minutes.
// Beyond ~300 s browsers drop the connection anyway. Memory (RLIMIT_AS) is
// the primary guard against pathological inputs, which hit it within ~5 min.
const DEFAULT_TIMEOUT_SECS: u64 = 300;

fn mem_limit_bytes() -> u64 {
    std::env::var("CANON_MEM_LIMIT_BYTES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_MEM_LIMIT_BYTES)
}

fn timeout_secs() -> u64 {
    std::env::var("CANON_TIMEOUT_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_TIMEOUT_SECS)
}

// One canonicalization at a time: each child may legitimately use several GiB,
// and the container instance only has 8 GiB total.
static CANON_SLOT: Semaphore = Semaphore::const_new(1);

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

    let payload = serde_json::to_vec(&req).map_err(internal)?;
    let _slot = CANON_SLOT.acquire().await.map_err(internal)?;

    let exe = std::env::current_exe().map_err(internal)?;
    let mut child = tokio::process::Command::new(exe)
        .arg("--worker")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .kill_on_drop(true)
        .spawn()
        .map_err(internal)?;

    let mut stdin = child.stdin.take().expect("child stdin is piped");
    stdin.write_all(&payload).await.map_err(internal)?;
    drop(stdin);

    let secs = timeout_secs();
    let output = match tokio::time::timeout(
        Duration::from_secs(secs),
        child.wait_with_output(),
    )
    .await
    {
        // Dropping the wait future on timeout kills the child (kill_on_drop).
        Err(_elapsed) => {
            tracing::warn!("canonicalization of {n}-element magma timed out after {secs}s");
            return Err((
                StatusCode::UNPROCESSABLE_ENTITY,
                format!(
                    "canonicalization timed out after {secs}s; \
                     this magma may be too symmetric for automatic canonicalization"
                ),
            ));
        }
        Ok(res) => res.map_err(internal)?,
    };

    if !output.status.success() {
        tracing::warn!(
            "canonicalization worker for {n}-element magma exited abnormally: {}",
            output.status
        );
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            format!(
                "canonicalization aborted (worker {}); it likely exceeded the memory limit \
                 of {} MiB — this magma may be too symmetric for automatic canonicalization",
                output.status,
                mem_limit_bytes() / (1024 * 1024),
            ),
        ));
    }

    let resp: CanonResp = serde_json::from_slice(&output.stdout).map_err(internal)?;
    Ok(Json(resp))
}

fn internal<E: std::fmt::Display>(e: E) -> (StatusCode, String) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("canonicalize failed: {e}"),
    )
}

// ---------------------------------------------------------------------------
// Worker mode: read a CanonReq JSON on stdin, write a CanonResp JSON on
// stdout. Runs under an RLIMIT_AS cap so runaway canonicalizations die here
// (Rust aborts on allocation failure) without taking the server down.
// ---------------------------------------------------------------------------

fn set_memory_limit(bytes: u64) {
    let lim = libc::rlimit {
        rlim_cur: bytes,
        rlim_max: bytes,
    };
    let rc = unsafe { libc::setrlimit(libc::RLIMIT_AS, &lim) };
    if rc != 0 {
        eprintln!(
            "warning: setrlimit(RLIMIT_AS, {bytes}) failed: {}",
            std::io::Error::last_os_error()
        );
    }
}

fn worker_main() {
    use eq677::{Magma, MatrixMagma};
    use std::io::Read;

    set_memory_limit(mem_limit_bytes());

    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .expect("failed to read stdin");
    let req: CanonReq = serde_json::from_str(&input).expect("invalid worker input");
    let n = req.table.len();

    let m = MatrixMagma::by_fn(n, |x, y| req.table[x][y]);
    let (canon, perm) = m.canonicalize2_with_perm();
    let resp = CanonResp {
        canonical: canon.to_string(),
        is255: canon.is255(),
        perm,
    };
    serde_json::to_writer(std::io::stdout().lock(), &resp).expect("failed to write result");
}

async fn health() -> &'static str {
    "ok"
}

fn main() {
    if std::env::args().nth(1).as_deref() == Some("--worker") {
        worker_main();
        return;
    }
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(server_main());
}

async fn server_main() {
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
    tracing::info!(
        "listening on {addr} (worker mem limit {} MiB, timeout {}s)",
        mem_limit_bytes() / (1024 * 1024),
        timeout_secs()
    );
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
