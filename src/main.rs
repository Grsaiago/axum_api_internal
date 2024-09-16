use axum::{routing::get, Router};
use axum_prometheus::PrometheusMetricLayerBuilder;
use tokio::{net::TcpListener, signal};
use tower_http::trace::{DefaultOnFailure, DefaultOnRequest, DefaultOnResponse, TraceLayer};

mod setup;
use setup::setup_graceful_shutdown;
use setup::setup_logging;

const METRICS_PREFIX: &str = "app";

#[tokio::main]
async fn main() {
    setup_logging();

    // setup prometheus exporting
    let (prom_layer, metric_handle) = PrometheusMetricLayerBuilder::new()
        .with_prefix(METRICS_PREFIX)
        .with_ignore_pattern("/metrics")
        .with_default_metrics()
        .build_pair();

    let app = Router::new()
        .route("/ping", get(|| async { "Pong" }))
        .route("/metrics", get(|| async move { metric_handle.render() }))
        .layer(
            TraceLayer::new_for_http()
                // setup log level for each event
                .on_request(DefaultOnRequest::new().level(tracing::Level::INFO))
                .on_response(DefaultOnResponse::new().level(tracing::Level::INFO))
                .on_failure(DefaultOnFailure::new().level(tracing::Level::ERROR)),
        )
        .layer(prom_layer);

    let listener = TcpListener::bind("127.0.0.1:8080")
        .await
        .expect("Bind failed");

    tracing::info!("Starting server on localhost port 8080");
    match axum::serve(listener, app)
        .with_graceful_shutdown(setup_graceful_shutdown())
        .await
    {
        Ok(_) => tracing::info!("Server shutdown succesfully"),
        Err(error) => tracing::error!("Server shutdown error: {}", error),
    }
}
