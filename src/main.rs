use axum::{routing::get, Router};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let app = Router::new().route("/ping", get(|| async { "Pong" }));
    let listener = TcpListener::bind("127.0.0.1:8080")
        .await
        .expect("Bind failed");
    let _ = axum::serve(listener, app).await;
}
