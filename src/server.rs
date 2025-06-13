use axum::{routing::get, Router};
use std::net::SocketAddr;
use tracing::info;

pub async fn run_health_server() {
    // Define routes
    let app = Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/metrics", get(metrics_handler));

    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("Starting health server on http://{}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn metrics_handler() -> &'static str {
    "# HELP dummy_metric Always returns 1\n# TYPE dummy_metric gauge\ndummy_metric 1"
}

