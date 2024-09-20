use axum::{
    routing::{get, trace},
    Router,
};
use axum_prometheus::PrometheusMetricLayerBuilder;
use tokio::{net::TcpListener, signal};
use tower_http::trace::{DefaultOnFailure, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
mod setup;
use setup::{setup_connection_string, setup_graceful_shutdown, setup_host_port, setup_logging};

const METRICS_PREFIX: &str = "app";

#[utoipa::path(
    get,
    path = "/healthcheck",
    responses (
        (status = OK, body = inline(str), description = "Healthcheck route"),
    ),
)]
// general healthcheck route, returns "ok"
async fn healthcheck() -> &'static str {
    "ok"
}

#[derive(OpenApi)]
#[openapi(
    paths(healthcheck),
    info(
        contact(
            name = "Gabriel Saiago",
            email = "grsaiago@gmail.com",
            url = "github.com/Grsaiago",
        ),
        title = "Axum API!",
    )
)]
struct ApiDoc;

#[tokio::main]
async fn main() {
    setup_logging();

    if let Err(err) = dotenvy::dotenv() {
        tracing::warn!("Failed to load .env file: {}", err);
    }

    // setup prometheus exporting
    let (prom_layer, metric_handle) = PrometheusMetricLayerBuilder::new()
        .with_prefix(METRICS_PREFIX)
        .with_ignore_pattern("/metrics")
        .with_default_metrics()
        .build_pair();

    let connection_string = match setup_connection_string() {
        Ok(conn_str) => conn_str,
        Err(err) => {
            tracing::error!("error setting up connection string: {}", err);
            return;
        }
    };

    let db_connection = match sea_orm::Database::connect(connection_string).await {
        Ok(connection) => connection,
        Err(err) => {
            tracing::error!("not able to connect to database: {}", err.to_string());
            return;
        }
    };

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/healthcheck", get(healthcheck))
        .route("/metrics", get(|| async move { metric_handle.render() }))
        .layer(
            TraceLayer::new_for_http()
                // setup log level for each event
                .on_request(DefaultOnRequest::new().level(tracing::Level::INFO))
                .on_response(DefaultOnResponse::new().level(tracing::Level::INFO))
                .on_failure(DefaultOnFailure::new().level(tracing::Level::ERROR)),
        )
        .layer(prom_layer);

    let host_port = setup_host_port();
    let listener = TcpListener::bind(&host_port).await.unwrap_or_else(|err| {
        tracing::error!("Error binding on {}: {}", &host_port, err);
        panic!("Error binding on {}", &host_port);
    });

    tracing::info!("Starting server on {}", &host_port);
    match axum::serve(listener, app)
        .with_graceful_shutdown(setup_graceful_shutdown())
        .await
    {
        Ok(_) => tracing::info!("Server shutdown succesfully"),
        Err(error) => tracing::error!("Server shutdown error: {}", error),
    }
}
