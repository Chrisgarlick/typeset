mod brand;
mod config;
mod db;
mod error;
mod models;
mod parser;
mod renderers;
mod routes;
mod state;

use std::sync::Arc;

use axum::{
    middleware,
    routing::{delete, get, post},
    Router,
};
use sqlx::postgres::PgPoolOptions;
use tokio::sync::Semaphore;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use crate::config::Config;
use crate::db::queries::Database;
use crate::state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let config = Config::from_env()?;
    tracing::info!("Starting Typeset on {}:{}", config.host, config.port);

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;

    tracing::info!("Connected to database");

    sqlx::migrate!("./migrations").run(&pool).await?;
    tracing::info!("Migrations complete");

    let state = AppState {
        db: Database::new(pool),
        render_semaphore: Arc::new(Semaphore::new(config.max_render_concurrency)),
        config: Arc::new(config.clone()),
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let authenticated = Router::new()
        .route("/api/render", post(routes::render::handle_render))
        .route("/api/preview", post(routes::preview::handle_preview))
        .route("/api/clients", get(routes::clients::list_clients))
        .route("/api/clients", post(routes::clients::create_client))
        .route("/api/clients/:slug", get(routes::clients::get_client))
        .route(
            "/api/clients/:slug",
            delete(routes::clients::delete_client),
        )
        .route("/api/history", get(routes::history::get_history))
        .layer(middleware::from_fn(routes::auth::auth_middleware));

    let public = Router::new()
        .route("/health", get(routes::health::health_check))
        .route("/api/health", get(routes::health::health_check));

    let app = Router::new()
        .merge(authenticated)
        .merge(public)
        .with_state(state)
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Typeset listening on {addr}");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("Shutdown signal received, starting graceful shutdown");
}
