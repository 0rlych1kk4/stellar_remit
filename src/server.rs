use axum::{routing::get, Router};
use prometheus::{gather, Encoder, TextEncoder};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::{error, info};

/// Launches a background HTTP server exposing `/health` and `/metrics`.
pub async fn run_health_server() {
    // Build our router with health & metrics endpoints
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/metrics", get(metrics_handler));

    // Bind to 127.0.0.1:3000
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("Health & metrics server listening on http://{}", addr);

    // Create the TCP listener
    let listener = match TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(err) => {
            error!("Failed to bind health server listener: {}", err);
            return;
        }
    };

    // Serve with axum
    if let Err(err) = axum::serve(listener, app).await {
        error!("Health server error: {}", err);
    }
}

async fn health_check() -> &'static str {
    "OK"
}

async fn metrics_handler() -> String {
    // Gather all metrics and encode in Prometheus text format
    let encoder = TextEncoder::new();
    let mfs = gather();
    let mut buf = Vec::new();
    encoder.encode(&mfs, &mut buf).unwrap();
    String::from_utf8(buf).unwrap()
}
