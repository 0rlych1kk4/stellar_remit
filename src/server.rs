use axum::{routing::get, Router};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::info;

pub async fn run_health_server() {
    // Define the application routes
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/metrics", get(metrics_handler));

    // Define the address to bind
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("Health server listening on http://{}", addr);

    // Use TcpListener as required by axum::serve in v0.7+
    let listener = TcpListener::bind(addr).await.unwrap();

    // Run the server
    axum::serve(listener, app).await.unwrap();
}

// Health check route handler
async fn health_check() -> &'static str {
    "OK"
}

// Prometheus-style dummy metrics endpoint
async fn metrics_handler() -> &'static str {
    "# HELP dummy_metric Always returns 1\n# TYPE dummy_metric gauge\ndummy_metric 1"
}
