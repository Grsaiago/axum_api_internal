use std::io::IsTerminal;

pub fn setup_logging() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .with_ansi(std::io::stdout().is_terminal())
        .compact()
        .init();
}

pub async fn setup_graceful_shutdown() {
    let cntrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Error installing Cntrl+C hadler");
    };

    #[cfg(unix)]
    let terminate = async {
        crate::signal::unix::signal(crate::signal::unix::SignalKind::terminate())
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
