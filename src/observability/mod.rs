pub mod metrics;

/// Inicializa tracing con salida humana o JSON estructurado.
///
/// El formato se controla con `YSH_LOG_JSON=1` (útil en contenedores, donde los
/// logs JSON se ingieren por la pila ELK/Loki). La verbosidad se sigue
/// controlando con `RUST_LOG` (por defecto `info`).
pub fn setup_tracing() {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    let json_enabled = std::env::var("YSH_LOG_JSON")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    if json_enabled {
        tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .with_target(true)
            .with_thread_ids(true)
            .json()
            .with_current_span(true)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .with_target(true)
            .with_thread_ids(true)
            .with_file(true)
            .with_line_number(true)
            .init();
    }

    tracing::info!(
        log_format = if json_enabled { "json" } else { "plain" },
        "Tracing initialized"
    );
}
