use axum::{
    Json,
    extract::{Path, Query, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use std::collections::HashMap;

use crate::auth::jwt::AuthUser;
use crate::server::AppState;

fn require_admin(auth: &AuthUser) -> Result<(), (StatusCode, String)> {
    if auth.role != "admin" {
        return Err((StatusCode::FORBIDDEN, "Admin access required".into()));
    }
    Ok(())
}

fn http_err(e: anyhow::Error) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}

fn default_range(state: &AppState, params: &HashMap<String, String>) -> i64 {
    params
        .get("days")
        .and_then(|s| s.parse().ok())
        .unwrap_or(state.config.analytics.default_range_days)
}

/// REAL-TIME METRICS: online users, active calls, pending ops, cache & db state.
pub async fn realtime_analytics(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    require_admin(&auth)?;
    let online = state.ws_connections.lock().await.online_count();
    let db_metrics = state.db.realtime_db_metrics().map_err(http_err)?;
    let cache = state.cache.stats();
    let rooms = state.webrtc_rooms.lock().await;
    let room_count = rooms.room_count();
    drop(rooms);

    let day = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let revenue = state
        .db
        .sum_transactions_range(&day, &day)
        .map_err(http_err)?;
    let gifts = state.db.sum_gifts_range(&day, &day).map_err(http_err)?;

    Ok(Json(serde_json::json!({
        "online_users": online,
        "active_rooms": room_count,
        "db": db_metrics,
        "cache_entries": cache.total_entries,
        "cache_bytes": cache.total_bytes,
        "today": { "transactions": revenue, "gifts": gifts },
        "captured_at": chrono::Utc::now().to_rfc3339(),
    })))
}

/// USER ANALYTICS: DAU series, new signups, retention, churn, MAU.
pub async fn user_analytics(
    auth: AuthUser,
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    require_admin(&auth)?;
    let days = default_range(&state, &params);
    let data = state.db.get_user_analytics(days).map_err(http_err)?;
    Ok(Json(data))
}

/// REVENUE ANALYTICS: MRR proxy, ARPU, LTV, gift + call economy.
pub async fn revenue_analytics(
    auth: AuthUser,
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    require_admin(&auth)?;
    let days = default_range(&state, &params);
    let data = state.db.get_revenue_analytics(days).map_err(http_err)?;
    Ok(Json(data))
}

/// AGENCY PERFORMANCE dashboards.
pub async fn agency_analytics(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    require_admin(&auth)?;
    let agencies = state.db.get_agency_performance().map_err(http_err)?;
    Ok(Json(serde_json::json!({ "agencies": agencies })))
}

/// HOST PERFORMANCE leaderboard.
pub async fn hosts_leaderboard(
    auth: AuthUser,
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    require_admin(&auth)?;
    let limit: i64 = params
        .get("limit")
        .and_then(|s| s.parse().ok())
        .unwrap_or(10);
    let hosts = state.db.get_host_leaderboard(limit).map_err(http_err)?;
    Ok(Json(serde_json::json!({ "leaderboard": hosts })))
}

/// GEOGRAPHIC DISTRIBUTION.
pub async fn geo_analytics(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    require_admin(&auth)?;
    let geo = state.db.get_geo_distribution().map_err(http_err)?;
    Ok(Json(geo))
}

/// MODERATION METRICS.
pub async fn moderation_analytics(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    require_admin(&auth)?;
    let metrics = state.db.get_moderation_metrics().map_err(http_err)?;
    Ok(Json(metrics))
}

fn read_proc_meminfo() -> Option<(u64, u64)> {
    let content = std::fs::read_to_string("/proc/meminfo").ok()?;
    let mut total = 0u64;
    let mut available = 0u64;
    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("MemTotal:") {
            total = rest.split_whitespace().next()?.parse::<u64>().ok()? * 1024;
        } else if let Some(rest) = line.strip_prefix("MemAvailable:") {
            available = rest.split_whitespace().next()?.parse::<u64>().ok()? * 1024;
        }
    }
    Some((total, available))
}

fn read_cpu_times() -> Option<(u64, u64)> {
    let content = std::fs::read_to_string("/proc/stat").ok()?;
    let line = content.lines().next()?;
    let parts: Vec<u64> = line
        .split_whitespace()
        .skip(1)
        .filter_map(|p| p.parse().ok())
        .collect();
    if parts.len() < 4 {
        return None;
    }
    let idle: u64 = parts.iter().skip(3).sum();
    let total: u64 = parts.iter().sum();
    Some((total, idle))
}

fn cpu_usage_pct() -> f64 {
    let a = read_cpu_times();
    std::thread::sleep(std::time::Duration::from_millis(150));
    let b = read_cpu_times();
    match (a, b) {
        (Some((t1, i1)), Some((t2, i2))) => {
            let dt = t2.saturating_sub(t1);
            let di = i2.saturating_sub(i1);
            if dt == 0 {
                0.0
            } else {
                let usage = (dt - di) as f64 / dt as f64;
                (usage * 10000.0).round() / 100.0
            }
        }
        _ => 0.0,
    }
}

/// SYSTEM HEALTH: uptime, memory, cpu, cache, db, threads.
pub async fn system_health(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    require_admin(&auth)?;
    let uptime = std::fs::read_to_string("/proc/uptime")
        .ok()
        .and_then(|s| {
            s.split_whitespace()
                .next()
                .map(|v| v.parse::<f64>().unwrap_or(0.0))
        })
        .unwrap_or(0.0);
    let (mem_total, mem_available) = read_proc_meminfo().unwrap_or((0, 0));
    let cache = state.cache.stats();
    let db_size = state.db.db_size().map_err(http_err)?;
    let threads = match std::fs::read_to_string("/proc/self/status") {
        Ok(s) => s
            .lines()
            .find(|l| l.starts_with("Threads:"))
            .and_then(|l| {
                l.split_whitespace()
                    .last()
                    .map(|v| v.parse::<u64>().unwrap_or(0))
            })
            .unwrap_or(0),
        Err(_) => 0,
    };

    Ok(Json(serde_json::json!({
        "uptime_secs": (uptime * 10.0).round() / 10.0,
        "memory": {
            "total_bytes": mem_total,
            "available_bytes": mem_available,
            "used_pct": if mem_total > 0 { (100.0 - (mem_available as f64 / mem_total as f64 * 100.0)) * 100.0 / 100.0 } else { 0.0 },
        },
        "cpu_usage_pct": cpu_usage_pct(),
        "cache": { "entries": cache.total_entries, "bytes": cache.total_bytes },
        "db_size_bytes": db_size,
        "threads": threads,
        "captured_at": chrono::Utc::now().to_rfc3339(),
    })))
}

/// Daily snapshots computed by the analytics worker.
pub async fn analytics_snapshots(
    auth: AuthUser,
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    require_admin(&auth)?;
    let limit: i64 = params
        .get("limit")
        .and_then(|s| s.parse().ok())
        .unwrap_or(30);
    let rows = state.db.list_analytics_snapshots(limit).map_err(http_err)?;
    Ok(Json(serde_json::json!({ "snapshots": rows })))
}

fn csv_cell(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => format!("\"{}\"", s.replace('"', "\"\"")),
        other => other.to_string(),
    }
}

fn to_csv(rows: &[serde_json::Value]) -> String {
    let mut out = String::new();
    let headers: Vec<&str> = rows
        .first()
        .map(|r| {
            r.as_object()
                .map(|o| o.keys().map(|k| k.as_str()).collect())
                .unwrap_or_default()
        })
        .unwrap_or_default();
    out.push_str(&headers.join(","));
    out.push('\n');
    for row in rows {
        let cells: Vec<String> = headers.iter().map(|h| csv_cell(&row[h])).collect();
        out.push_str(&cells.join(","));
        out.push('\n');
    }
    out
}

/// CSV / JSON export for user, revenue, host and geo datasets.
pub async fn export_analytics(
    auth: AuthUser,
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Response, (StatusCode, String)> {
    require_admin(&auth)?;
    let dataset = params.get("dataset").map(String::as_str).unwrap_or("users");
    let format = params.get("format").map(String::as_str).unwrap_or("json");
    let days = params
        .get("days")
        .and_then(|s| s.parse().ok())
        .unwrap_or(30);

    let rows: Vec<serde_json::Value> = match dataset {
        "users" => state
            .db
            .get_user_analytics(days)
            .map_err(http_err)?
            .get("days")
            .and_then(|d| d.as_array())
            .cloned()
            .unwrap_or_default(),
        "revenue" => vec![state.db.get_revenue_analytics(days).map_err(http_err)?],
        "hosts" => state.db.get_host_leaderboard(50).map_err(http_err)?,
        "geo" => state
            .db
            .get_geo_distribution()
            .map_err(http_err)?
            .get("distribution")
            .and_then(|d| d.as_array())
            .cloned()
            .unwrap_or_default(),
        "snapshots" => state.db.list_analytics_snapshots(90).map_err(http_err)?,
        other => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("Unknown dataset: {}", other),
            ));
        }
    };

    match format {
        "csv" => {
            let body = to_csv(&rows);
            Ok(axum::response::Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "text/csv")
                .header(
                    header::CONTENT_DISPOSITION,
                    format!("attachment; filename=\"{}.csv\"", dataset),
                )
                .body(axum::body::Body::from(body))
                .unwrap())
        }
        "json" => Ok(Json(serde_json::json!({ "dataset": dataset, "rows": rows })).into_response()),
        other => Err((
            StatusCode::BAD_REQUEST,
            format!("Unknown format: {}", other),
        )),
    }
}

/// User-facing: allow a user to set their region for geo analytics.
pub async fn set_my_region(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(region): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    if region.is_empty() || region.len() > 64 {
        return Err((StatusCode::BAD_REQUEST, "Invalid region".into()));
    }
    let user_id = auth
        .user_id
        .parse::<i64>()
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid user".into()))?;
    state
        .db
        .set_user_region(user_id, &region)
        .map_err(http_err)?;
    Ok(Json(serde_json::json!({ "region": region })))
}
