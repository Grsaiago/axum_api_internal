use std::io::IsTerminal;

use axum::{routing::get, Router};
use tokio::net::TcpListener;
use tower_http::trace::{DefaultOnFailure, DefaultOnRequest, DefaultOnResponse, TraceLayer};

fn setup_logging() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .with_ansi(std::io::stdout().is_terminal())
        .compact()
        .init();
}

#[tokio::main]
async fn main() {
    setup_logging();

    let app = Router::new()
        .route("/ping", get(|| async { "Pong" }))
        .layer(
            TraceLayer::new_for_http()
                .on_request(DefaultOnRequest::new().level(tracing::Level::INFO))
                .on_response(DefaultOnResponse::new().level(tracing::Level::INFO))
                .on_failure(DefaultOnFailure::new().level(tracing::Level::ERROR)),
        );

    tracing::info!("Server started on localhost port 8080");
    let listener = TcpListener::bind("127.0.0.1:8080")
        .await
        .expect("Bind failed");
    let _ = axum::serve(listener, app).await;
}
