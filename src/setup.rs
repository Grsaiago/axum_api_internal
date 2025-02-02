use std::io::IsTerminal;

pub fn setup_logging() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .with_ansi(std::io::stdout().is_terminal())
        .compact()
        .init();
}

pub fn setup_host_port() -> String {
    let host = std::env::var("SERVER_HOST").unwrap_or_else(|err| {
        tracing::warn!(
            "Error loading SERVER_HOST env, fallback to 127.0.0.1: {}",
            err.to_string()
        );
        "127.0.0.1".to_string()
    });

    let port = std::env::var("SERVER_PORT").unwrap_or_else(|err| {
        tracing::warn!(
            "Error loading SERVER_PORT env, fallback to 8080: {}",
            err.to_string()
        );
        "8080".to_string()
    });

    format!("{}:{}", host, port)
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

pub fn setup_connection_string() -> Result<String, std::env::VarError> {
    let config_vars = [
        "POSTGRES_DB",
        "POSTGRES_USER",
        "POSTGRES_PASSWORD",
        "DB_HOST",
    ];

    let unset_vars: Vec<_> = config_vars
        .iter()
        .filter(|&var| std::env::var(var).is_err())
        .copied()
        .collect();

    if !unset_vars.is_empty() {
        for var in unset_vars {
            tracing::info!("unable to load env var {}", &var);
        }
        Err(std::env::VarError::NotPresent)
    } else {
        let db_name = std::env::var("POSTGRES_DB").unwrap();
        let db_user = std::env::var("POSTGRES_USER").unwrap();
        let db_password = std::env::var("POSTGRES_PASSWORD").unwrap();
        let db_host = std::env::var("DB_HOST").unwrap();
        Ok(format!(
            "postgres://{db_user}:{db_password}@{db_host}/{db_name}"
        ))
    }
}
