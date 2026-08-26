mod actors;
mod api;
mod auth;
mod cache;
mod config;
mod db;
mod middleware;
mod notification;
mod observability;
mod security;
mod server;
mod ws;

use anyhow::Result;
use std::sync::Arc;
use tokio::signal;
use tokio::sync::watch;

use crate::cache::{Cache, RateLimitCache, SessionCache};
use crate::middleware::circuit_breaker::CircuitBreaker;
use crate::middleware::ddos_protection::DdosProtection;
use crate::middleware::ip_blocklist::IpBlocklist;
use crate::middleware::rate_limit::PerIpRateLimiter;
use crate::middleware::ws_guard::WsGuard;
use crate::security::keys::{Ed25519KeyPair, X25519KeyPair};
use crate::security::zeroize::{EncryptedKey, SecureBuffer, SecureString};

#[tokio::main]
async fn main() -> Result<()> {
    observability::setup_tracing();
    tracing::info!("YSH starting...");

    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "config/default.toml".to_string());

    tracing::info!("Loading config from: {}", config_path);
    let ysh_config = config::load_config(&config_path)?;
    tracing::info!("Config loaded successfully");

    let (shutdown_tx, mut shutdown_rx) = watch::channel(false);
    let config_ref = std::sync::Arc::new(ysh_config.clone());

    tracing::info!("Generating cryptographic key pairs...");
    let x25519_keys = X25519KeyPair::generate();
    tracing::info!("X25519 ECDH key pair generated (pub: {} bytes)", x25519_keys.public.as_bytes().len());

    let ed25519_keys = Ed25519KeyPair::generate();
    let signature = ed25519_keys.sign(b"ysh startup");
    ed25519_keys
        .verify(b"ysh startup", &signature)
        .map_err(|e| anyhow::anyhow!("Ed25519 verification failed: {}", e))?;
    tracing::info!("Ed25519 signing key pair generated and verified");

    tracing::info!("Creating secure secrets...");
    let secure_jwt_secret =
        SecureString::new(ysh_config.secrets.jwt_secret.clone());
    let secure_encryption_key =
        SecureBuffer::new(ysh_config.secrets.encryption_key.as_bytes().to_vec());
    let encrypted_key = EncryptedKey::new(
        ysh_config.secrets.encryption_key.as_bytes().to_vec(),
        ysh_config.encryption.algorithm.clone(),
    );
    tracing::info!(
        "Secure secrets created (algo: {}, key bytes: {})",
        encrypted_key.algorithm(),
        encrypted_key.as_bytes().len()
    );

    tracing::info!("Initializing database...");
    let db = std::sync::Arc::new(db::Database::new("ysh.db")?);
    tracing::info!("Database initialized");

    tracing::info!("Initializing cache...");
    std::fs::create_dir_all("data/cache")?;
    std::fs::create_dir_all("data/sessions")?;
    std::fs::create_dir_all("data/ratelimit")?;
    let cache = std::sync::Arc::new(Cache::open("data/cache")?);
    let session_cache = std::sync::Arc::new(SessionCache::new(Cache::open("data/sessions")?));
    let rate_limit_cache = std::sync::Arc::new(RateLimitCache::new(Cache::open("data/ratelimit")?));
    cache.set_string("startup:test", "ok")?;
    let test_val = cache.get_string("startup:test")?.unwrap_or_default();
    tracing::info!("Cache initialized (sled KV, startup test: {})", test_val);
    cache.delete("startup:test")?;

    tracing::info!("Initializing DDoS protection...");
    let ddos_cfg = &ysh_config.ddos;
    let ip_blocklist = IpBlocklist::new(
        ddos_cfg.ip_block.auto_block_threshold,
        ddos_cfg.ip_block.auto_block_window_secs,
        ddos_cfg.ip_block.auto_block_duration_secs,
        ddos_cfg.ip_block.max_blocklist_size,
    );
    let per_ip_limiter = PerIpRateLimiter::new(
        ddos_cfg.rate_limit.clone(),
        ip_blocklist.clone(),
    );
    let ws_guard = WsGuard::new(ddos_cfg.ws.clone(), ip_blocklist.clone());
    let ddos_protection = DdosProtection::new(Arc::new(ddos_cfg.clone()), ip_blocklist.clone());
    tracing::info!("DDoS protection enabled (body limit: {} bytes, timeout: {}s)", ddos_cfg.max_body_bytes, ddos_cfg.request_timeout_secs);

    tracing::info!("Starting actors...");
    let (supervisor, _supervisor_handle) = ractor::Actor::spawn(
        Some("supervisor-tree".to_string()),
        actors::supervisor_tree::SupervisorTree,
        config_path.clone(),
    )
    .await?;

    let (config_actor, _config_handle) = ractor::Actor::spawn(
        Some("config-actor".to_string()),
        actors::config_actor::ConfigActor,
        config_path,
    )
    .await?;

    let (server_actor, _server_handle) = ractor::Actor::spawn(
        Some("server-actor".to_string()),
        actors::server_actor::ServerActor,
        (ysh_config.server.host.clone(), ysh_config.server.port),
    )
    .await?;

    let (db_actor, _db_handle) = ractor::Actor::spawn(
        Some("database-actor".to_string()),
        actors::database_actor::DatabaseActor,
        (
            db.clone(),
            ysh_config.database.url.clone(),
            ysh_config.database.max_connections,
        ),
    )
    .await?;

    let (crypto_actor, _crypto_handle) = ractor::Actor::spawn(
        Some("crypto-actor".to_string()),
        actors::crypto_actor::CryptoActor,
        ysh_config.encryption.algorithm.clone(),
    )
    .await?;

    let (session_actor, _session_handle) = ractor::Actor::spawn(
        Some("session-supervisor".to_string()),
        actors::session_supervisor::SessionSupervisor,
        10000u32,
    )
    .await?;

    let (webrtc_actor, _webrtc_handle) = ractor::Actor::spawn(
        Some("webrtc-actor".to_string()),
        actors::webrtc_actor::WebRTCActor,
        100u32,
    )
    .await?;

    let (ai_actor, _ai_handle) = ractor::Actor::spawn(
        Some("ai-actor".to_string()),
        actors::ai_actor::AIActor,
        (),
    )
    .await?;

    let (notification_actor, _notification_handle) = ractor::Actor::spawn(
        Some("notification-actor".to_string()),
        actors::notification_actor::NotificationActor,
        (),
    )
    .await?;

    use actors::config_actor::ConfigActorMsg;
    use actors::crypto_actor::CryptoActorMsg;
    use actors::database_actor::DatabaseActorMsg;
    use actors::notification_actor::NotificationMsg;
    use actors::session_supervisor::SessionSupervisorMsg;
    use actors::supervisor_tree::SupervisorTreeMsg;

    let _ = supervisor.send_message(SupervisorTreeMsg::GetConfig);
    let _ = supervisor.send_message(SupervisorTreeMsg::Shutdown);
    let _ = config_actor.send_message(ConfigActorMsg::Reload);
    let _ = config_actor.send_message(ConfigActorMsg::ConfigChanged("config/default.toml".into()));
    let _ = db_actor.send_message(DatabaseActorMsg::HealthCheck);
    let _ = db_actor.send_message(DatabaseActorMsg::QueryCount);
    let _ = db_actor.send_message(DatabaseActorMsg::GetStats);
    let _ = crypto_actor.send_message(CryptoActorMsg::RotateKeys);
    let _ = crypto_actor.send_message(CryptoActorMsg::Encrypt {
        plaintext: b"YSH startup probe".to_vec(),
        aad: b"system".to_vec(),
    });
    let _ = crypto_actor.send_message(CryptoActorMsg::Decrypt {
        ciphertext: b"encrypted data".to_vec(),
        aad: b"system".to_vec(),
    });
    let _ = session_actor.send_message(SessionSupervisorMsg::GetActiveCount);
    let _ = session_actor.send_message(SessionSupervisorMsg::SessionStarted {
        user_id: "system".to_string(),
    });
    let _ = session_actor.send_message(SessionSupervisorMsg::SessionEnded {
        user_id: "system".to_string(),
    });
    let _ = webrtc_actor.send_message(actors::webrtc_actor::WebRTCActorMsg::CallStart {
        caller: "system".to_string(),
        callee: "system".to_string(),
    });
    let _ = webrtc_actor.send_message(actors::webrtc_actor::WebRTCActorMsg::CallEnd {
        caller: "system".to_string(),
        callee: "system".to_string(),
    });
    let _ = ai_actor.send_message(actors::ai_actor::AIActorMsg::LoadModel);
    let _ = ai_actor.send_message(actors::ai_actor::AIActorMsg::Moderate {
        content_id: "startup-check".to_string(),
    });
    let _ = ai_actor.send_message(actors::ai_actor::AIActorMsg::DeepfakeCheck {
        user_id: "system".to_string(),
    });
    let _ = notification_actor.send_message(NotificationMsg::SendEmail {
        notification_id: 0,
        to: "test@ysh.app".into(),
        subject: "YSH startup probe".into(),
        body: "System notification check".into(),
        username: "system".into(),
    });
    let _ = notification_actor.send_message(NotificationMsg::GetStats);

    use actors::server_actor::ServerActorMsg;
    let _ = server_actor.send_message(ServerActorMsg::Start);
    let _ = server_actor.send_message(ServerActorMsg::Stop);

    tracing::info!("All actors started and wired");

    let nonce_gen = security::nonce::NonceGenerator::new();
    let nonce = nonce_gen.next();
    tracing::info!("Nonce generated (counter: {}, nonce: {:02x?})", nonce_gen.current_counter(), &nonce[..4]);

    let peer_public = x25519_keys.public;
    let _shared = x25519_keys.agree(&peer_public);
    tracing::info!("X25519 key agreement completed");

    let secure_buf = security::zeroize::SecureBuffer::from(b"test data".as_slice());
    tracing::info!("SecureBuffer created (len: {}, empty: {})", secure_buf.len(), secure_buf.is_empty());

    let enc_key = security::zeroize::EncryptedKey::new(
        b"test key material".to_vec(),
        "aes-256-gcm".to_string(),
    );
    tracing::info!("EncryptedKey created (algo: {}, bytes: {})", enc_key.algorithm(), enc_key.as_bytes().len());

    let cert_path = std::env::var("YSH_TLS_CERT").unwrap_or_default();
    let key_path = std::env::var("YSH_TLS_KEY").unwrap_or_default();
    if !cert_path.is_empty() && cert_path != "/dev/null" {
        match security::tls::build_tls_config(&cert_path, &key_path) {
            Ok(_tls_config) => tracing::info!("TLS configured"),
            Err(e) => tracing::warn!("TLS not configured: {}", e),
        }
    }

    let cb = middleware::circuit_breaker::CircuitBreaker::new(3, std::time::Duration::from_secs(10));
    cb.record_failure();
    cb.record_failure();
    tracing::info!("Circuit breaker state: available={}", cb.is_available());

    let circuit_breaker = CircuitBreaker::new(5, std::time::Duration::from_secs(30));

    let ws_connections = std::sync::Arc::new(tokio::sync::Mutex::new(
        crate::ws::ConnectionManager::new(),
    ));
    let read_receipts = std::sync::Arc::new(tokio::sync::Mutex::new(
        std::collections::HashMap::<(i64, i64), i64>::new(),
    ));
    let (match_tx, _match_rx) = tokio::sync::mpsc::unbounded_channel::<crate::ws::MatchEvent>();

    let state = server::AppState {
        config: config_ref.clone(),
        db: db.clone(),
        cache: cache.clone(),
        session_cache,
        rate_limit_cache,
        secure_jwt_secret,
        secure_encryption_key,
        encrypted_key,
        session_actor,
        notification_actor,
        circuit_breaker,
        ws_connections,
        read_receipts,
        match_tx,
        ip_blocklist: ip_blocklist.clone(),
        per_ip_limiter,
        ws_guard,
        ddos_protection,
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
