mod actors;
mod auth;
mod config;
mod db;
mod health;
mod middleware;
mod observability;
mod security;
mod server;

use anyhow::Result;
use tokio::signal;
use tokio::sync::watch;

#[tokio::main]
async fn main() -> Result<()> {
    observability::setup_tracing();

    tracing::info!("YSH starting...");

    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "config/default.lua".to_string());

    tracing::info!("Loading config from: {}", config_path);
    let ysh_config = config::load_config(&config_path)?;
    tracing::info!("Config loaded successfully");

    let (shutdown_tx, mut shutdown_rx) = watch::channel(false);

    let config_ref = std::sync::Arc::new(ysh_config.clone());

    tracing::info!("Starting actors...");
    let (_supervisor, _supervisor_handle) = ractor::Actor::spawn(
        Some("supervisor-tree".to_string()),
        actors::supervisor_tree::SupervisorTree,
        config_path.clone(),
    )
    .await?;

    let (_config_actor, _config_handle) = ractor::Actor::spawn(
        Some("config-actor".to_string()),
        actors::config_actor::ConfigActor,
        config_path,
    )
    .await?;

    let (_server_actor, _server_handle) = ractor::Actor::spawn(
        Some("server-actor".to_string()),
        actors::server_actor::ServerActor,
        (ysh_config.server.host.clone(), ysh_config.server.port),
    )
    .await?;

    let (_db_actor, _db_handle) = ractor::Actor::spawn(
        Some("database-actor".to_string()),
        actors::database_actor::DatabaseActor,
        (
            ysh_config.database.url.clone(),
            ysh_config.database.max_connections,
        ),
    )
    .await?;

    let (_crypto_actor, _crypto_handle) = ractor::Actor::spawn(
        Some("crypto-actor".to_string()),
        actors::crypto_actor::CryptoActor,
        ysh_config.encryption.algorithm.clone(),
    )
    .await?;

    let (_session_actor, _session_handle) = ractor::Actor::spawn(
        Some("session-supervisor".to_string()),
        actors::session_supervisor::SessionSupervisor,
        10000u32,
    )
    .await?;

    tracing::info!("All actors started. Spawning {} server workers", ysh_config.server.workers);

    tracing::info!("Initializing database...");
    let database = db::Database::new("ysh.db")?;
    let db_ref = std::sync::Arc::new(database);
    tracing::info!("Database initialized");

    let state = server::AppState {
        config: config_ref,
        db: db_ref,
    };

    let app = server::build_router(state);

    let addr = format!("{}:{}", ysh_config.server.host, ysh_config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Server listening on {}", addr);

    let server_handle = tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                let _ = shutdown_rx.changed().await;
                tracing::info!("HTTP server shutting down...");
            })
            .await
            .expect("Server failed");
    });

    tokio::spawn(async move {
        signal::ctrl_c()
            .await
            .expect("Failed to listen for Ctrl+C");
        tracing::info!("Shutdown signal received");
        let _ = shutdown_tx.send(true);
    });

    tracing::info!("YSH is ready. Press Ctrl+C to shutdown.");

    server_handle.await?;

    tracing::info!("YSH stopped.");
    Ok(())
}
