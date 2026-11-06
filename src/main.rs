mod brand;
mod clients;
mod config;
mod encoding;
mod db;
mod error;
mod events;
mod models;
mod parser;
mod rate_limit;
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

/// The largest render request: a document with its embedded images. nginx's
/// `client_max_body_size` for /api/ must allow at least this.
const MAX_DOCUMENT_BYTES: usize = 25 * 1024 * 1024;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let config = Config::from_env()?;
    let export = std::env::args().nth(1).filter(|a| a == "export-brands").map(|_| std::env::args().nth(2).unwrap_or_else(|| "brands".into()));
    if export.is_none() {
        tracing::info!("Starting Typeset on {}:{}", config.host, config.port);
    }

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;

    tracing::info!("Connected to database");

    sqlx::migrate!("./migrations").run(&pool).await?;
    tracing::info!("Migrations complete");

    if let Some(dir) = export {
        return export_brands(&Database::new(pool), &dir).await;
    }

    let brands = config.clients_url.as_ref().map(|url| {
        Arc::new(clients::BrandClient::new(clients::Config {
            base: url.clone(),
            service_key: config.clients_key.clone(),
            cache_dir: config.clients_cache.clone().into(),
            asset_cache_bytes: 256 * 1024 * 1024,
        }))
    });
    match &brands {
        Some(_) => tracing::info!("Brands and keys from Clients at {}", config.clients_url.as_deref().unwrap_or("")),
        None => tracing::warn!("CLIENTS_URL isn't set: only the default look, and only TYPESET_TOKEN"),
    }
    if config.api_token.is_some() {
        tracing::warn!("TYPESET_TOKEN is still accepted; unset it once every caller uses a Clients key");
    }

    let state = AppState {
        db: Database::new(pool),
        brands,
        render_semaphore: Arc::new(Semaphore::new(config.max_render_concurrency)),
        rate_limiter: rate_limit::RateLimiter::new(config.rate_limit_per_min),
        config: Arc::new(config.clone()),
    };

    tokio::spawn(events::run(state.clone()));

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let rendering = Router::new()
        .route("/api/render", post(routes::render::handle_render))
        .route("/api/preview", post(routes::preview::handle_preview))
        .route("/api/proof", post(routes::proof::handle_proof))
        // Documents can embed images (data URIs), so allow more than axum's 2MB.
        .layer(axum::extract::DefaultBodyLimit::max(MAX_DOCUMENT_BYTES))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            rate_limit::rate_limit_middleware,
        ));

    let cheap = Router::new()
        .route("/api/clients", get(routes::clients::list_clients))
        .route("/api/clients", post(routes::clients::create_client))
        .route("/api/clients/:slug", get(routes::clients::get_client))
        .route(
            "/api/clients/:slug",
            delete(routes::clients::delete_client),
        )
        .route("/api/history", get(routes::history::get_history));

    let authenticated = rendering
        .merge(cheap)
        .layer(middleware::from_fn_with_state(
            state.clone(),
            routes::auth::auth_middleware,
        ));

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
    tracing::info!("Typeset listening on {addr} (rate limit {}/min)", config.rate_limit_per_min);

    axum::serve(listener, app.into_make_service_with_connect_info::<std::net::SocketAddr>())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

/// `typeset export-brands [dir]`: each old client profile as brand YAML for
/// Clients, with its logos downloaded beside it (plan P4.1).
async fn export_brands(db: &Database, dir: &str) -> anyhow::Result<()> {
    std::fs::create_dir_all(dir)?;
    let profiles = db.list_client_profiles().await?;
    for p in &profiles {
        let base = std::path::Path::new(dir).join(&p.slug);
        std::fs::write(base.with_extension("yaml"), brand::source::brand_yaml(p))?;
        let logos = brand::assets::fetch_logos(p).await;
        for logo in [logos.light, logos.dark].into_iter().flatten() {
            std::fs::write(std::path::Path::new(dir).join(format!("{}-{}", p.slug, logo.name)), &logo.bytes)?;
        }
        println!("{}: {}.yaml", p.name, base.display());
    }
    println!("{} profile(s) exported to {dir}", profiles.len());
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
