use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use std::sync::OnceLock;
use std::time::Instant;

static PROMETHEUS_HANDLE: OnceLock<PrometheusHandle> = OnceLock::new();
static STARTED: OnceLock<Instant> = OnceLock::new();

/// Installs the global metrics recorder and the Prometheus exporter.
///
/// Idempotent: safe to call multiple times (e.g. from unit tests). Returns
/// `false` when a global recorder could not be installed.
pub fn init() -> bool {
    if PROMETHEUS_HANDLE.get().is_some() {
        return true;
    }

    let handle = match PrometheusBuilder::new().install_recorder() {
        Ok(h) => h,
        Err(_) => return false,
    };
    let _ = PROMETHEUS_HANDLE.set(handle);
    let _ = STARTED.set(Instant::now());

    metrics::describe_counter!(
        "http_requests_total",
        "Total de peticiones HTTP procesadas, etiquetadas por familia de código"
    );
    metrics::describe_counter!(
        "http_rate_limited_total",
        "Peticiones rechazadas por rate limiting por IP"
    );
    metrics::describe_gauge!(
        "circuit_breaker_open",
        "1 si el circuit breaker está abierto (rechazando tráfico), 0 si no"
    );
    metrics::describe_gauge!("ysh_ws_connections_active", "Conexiones WebSocket activas");
    metrics::describe_counter!(
        "ysh_ws_connections_total",
        "Conexiones WebSocket aceptadas desde el arranque"
    );
    metrics::describe_gauge!("ysh_uptime_seconds", "Segundos de uptime del proceso");
    metrics::describe_gauge!(
        "ysh_db_size_bytes",
        "Tamaño del fichero de base de datos en bytes"
    );
    metrics::describe_gauge!("ysh_cache_entries", "Entradas vivas en la caché KV");
    metrics::describe_gauge!(
        "ysh_blocked_ips",
        "IPs actualmente bloqueadas por la protección DDoS"
    );
    metrics::describe_counter!(
        "ws_auth_failures_total",
        "Intentos de conexión WebSocket no autenticados"
    );
    true
}

pub fn is_initialized() -> bool {
    PROMETHEUS_HANDLE.get().is_some()
}

/// Seconds elapsed since the exporter was installed.
pub fn uptime_secs() -> u64 {
    STARTED.get().map(|t| t.elapsed().as_secs()).unwrap_or(0)
}

/// Renders the current snapshot in Prometheus text exposition format.
pub fn render() -> Option<String> {
    PROMETHEUS_HANDLE.get().map(|h| h.render())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metrics_are_recorded_and_rendered_as_prometheus_text() {
        if !init() {
            eprintln!("skipping: recorder not installed");
            return;
        }
        metrics::counter!("test_rendered_requests_total").increment(1u64);
        metrics::gauge!("test_rendered_gauge").set(42.0f64);
        let output = render().unwrap_or_default();

        assert!(
            output.contains("test_rendered_requests_total"),
            "counter missing from exposition:\n{output}"
        );
        assert!(
            output.contains("test_rendered_gauge"),
            "gauge missing from exposition:\n{output}"
        );
        assert!(!output.trim().is_empty(), "empty exposition output");
    }

    #[test]
    fn init_is_idempotent() {
        let _ = init();
        let _ = init();
        assert!(is_initialized());
    }
}
