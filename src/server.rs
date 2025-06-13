use axum::{routing::get, Router};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::{error, info};

pub async fn run_health_server() {
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/metrics", get(metrics_handler));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("Health server listening on http://{}", addr);

    match TcpListener::bind(addr).await {
        Ok(listener) => {
            if let Err(err) = axum::serve(listener, app).await {
                error!("Health server error: {}", err);
            }
        }
        Err(err) => {
            error!("Failed to bind health server listener: {}", err);
        }
    }
}

async fn health_check() -> &'static str {
    "OK"
}

async fn metrics_handler() -> &'static str {
    "# HELP dummy_metric Always returns 1\n# TYPE dummy_metric gauge\ndummy_metric 1"
}

