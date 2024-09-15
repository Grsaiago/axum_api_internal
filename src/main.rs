use std::io::IsTerminal;

use axum::{routing::get, Router};
use tokio::{net::TcpListener, signal};
use tower_http::trace::{DefaultOnFailure, DefaultOnRequest, DefaultOnResponse, TraceLayer};

fn setup_logging() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .with_ansi(std::io::stdout().is_terminal())
        .compact()
        .init();
}

async fn shutdown_signal() {
    let cntrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Error installing Cntrl+C hadler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = cntrl_c => {},
        _ = terminate => {},
    }
}

#[tokio::main]
async fn main() {
    setup_logging();

    let app = Router::new()
        .route("/ping", get(|| async { "Pong" }))
        .layer(
            TraceLayer::new_for_http()
                // setup log level for each event
                .on_request(DefaultOnRequest::new().level(tracing::Level::INFO))
                .on_response(DefaultOnResponse::new().level(tracing::Level::INFO))
                .on_failure(DefaultOnFailure::new().level(tracing::Level::ERROR)),
        );

    let listener = TcpListener::bind("127.0.0.1:8080")
        .await
        .expect("Bind failed");

    tracing::info!("Starting server on localhost port 8080");
    match axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
    {
        Ok(_) => tracing::info!("Server shutdown succesfully"),
        Err(error) => tracing::error!("Server shutdown error: {}", error),
    }
}
