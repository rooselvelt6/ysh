//! i18n API: locale listing, detection, translation catalog, and admin CRUD.

use std::collections::HashMap;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::{StatusCode, header},
};

use crate::auth::jwt::AuthUser;
use crate::i18n::locales::{self, meta_for, supported_meta};
use crate::server::AppState;

fn require_admin(auth: &AuthUser) -> Result<(), (StatusCode, String)> {
    if auth.role != "admin" {
        return Err((StatusCode::FORBIDDEN, "Admin access required".into()));
    }
    Ok(())
}

/// Lists all supported locales with metadata (public).
pub async fn list_locales(State(_state): State<AppState>) -> Json<serde_json::Value> {
    let metas = supported_meta();
    Json(serde_json::json!({
        "default": "es",
        "locales": metas,
    }))
}

/// Detects the best locale from the `Accept-Language` header (public).
pub async fn detect(
    State(_state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Json<serde_json::Value> {
    let accept = headers
        .get(header::ACCEPT_LANGUAGE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    let code = locales::negotiate(&accept, "es");
    let meta = meta_for(&code);
    Json(serde_json::json!({
        "code": code,
        "name": meta.as_ref().map(|m| m.name.as_str()).unwrap_or(""),
        "native": meta.as_ref().map(|m| m.native.as_str()).unwrap_or(""),
        "dir": meta.as_ref().map(|m| m.dir).unwrap_or("ltr"),
        "rtl": meta.as_ref().map(|m| m.rtl).unwrap_or(false),
        "accept_language": accept,
    }))
}

/// Returns the full, resolved catalog for a locale (public).
pub async fn translations(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let locale = params.get("locale").cloned().unwrap_or_else(|| "es".into());
    if state.i18n_engine.base_source(&locale).is_none() {
        return Err((StatusCode::BAD_REQUEST, "unsupported locale".into()));
    }
    let catalog = state.i18n_engine.full_catalog(&locale);
    let meta = meta_for(&locale);
    Ok(Json(serde_json::json!({
        "locale": locale,
        "dir": meta.as_ref().map(|m| m.dir).unwrap_or("ltr"),
        "rtl": meta.as_ref().map(|m| m.rtl).unwrap_or(false),
        "messages": catalog,
    })))
}

/// Translates a single key with optional numeric arguments (public).
pub async fn translate(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let locale = params.get("locale").cloned().unwrap_or_else(|| "es".into());
    let key = params
        .get("key")
        .ok_or((StatusCode::BAD_REQUEST, "key required".into()))?;
    if state.i18n_engine.base_source(&locale).is_none() {
        return Err((StatusCode::BAD_REQUEST, "unsupported locale".into()));
    }

    let mut args: Vec<(&str, crate::i18n::Arg)> = Vec::new();
    if let Some(n) = params.get("n") {
        let arg = n
            .parse::<i64>()
            .map(crate::i18n::Arg::Number)
            .unwrap_or_else(|_| crate::i18n::Arg::Text(n.clone()));
        args.push(("n", arg));
    }
    if let Some(year) = params.get("year") {
        let arg = year
            .parse::<i64>()
            .map(crate::i18n::Arg::Number)
            .unwrap_or_else(|_| crate::i18n::Arg::Text(year.clone()));
        args.push(("year", arg));
    }

    let value = state.i18n_engine.translate(&locale, key, &args);
    Ok(Json(serde_json::json!({
        "locale": locale,
        "key": key,
        "value": value,
        "resolved": value.as_str() != key,
    })))
}

// ═══════════════════════════════════════════
// ADMIN CRUD
// ═══════════════════════════════════════════

/// Lists all translation keys plus any runtime overrides (admin).
pub async fn admin_list(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    require_admin(&auth)?;

    let keys = crate::i18n::catalog::all_keys();
    let overrides = state
        .db
        .list_i18n_overrides()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut ov = Vec::new();
    for (k, v) in overrides {
        let (locale, key) = match k.split_once("::") {
            Some((l, k2)) => (l.to_string(), k2.to_string()),
            None => (String::new(), k.clone()),
        };
        ov.push(serde_json::json!({ "locale": locale, "key": key, "value": v }));
    }

    Ok(Json(serde_json::json!({
        "locales": crate::i18n::catalog::SUPPORTED,
        "keys": keys,
        "overrides": ov,
    })))
}

/// Upserts a translation override for a locale/key (admin).
pub async fn admin_upsert(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    require_admin(&auth)?;

    let locale = req["locale"].as_str().unwrap_or("");
    let key = req["key"].as_str().unwrap_or("");
    let value = req["value"].as_str().unwrap_or("");

    if crate::i18n::catalog::source_for(locale).is_none() {
        return Err((StatusCode::BAD_REQUEST, "unsupported locale".into()));
    }
    if key.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "key required".into()));
    }

    state
        .db
        .set_i18n_override(locale, key, value)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    state
        .i18n_engine
        .overrides
        .lock()
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "lock poisoned".into()))?
        .insert(format!("{locale}::{key}"), value.to_string());

    Ok(Json(
        serde_json::json!({ "locale": locale, "key": key, "value": value }),
    ))
}

/// Deletes a translation override (admin).
pub async fn admin_delete(
    auth: AuthUser,
    State(state): State<AppState>,
    Path((locale, key)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    require_admin(&auth)?;

    let removed = state
        .db
        .delete_i18n_override(&locale, &key)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    state
        .i18n_engine
        .overrides
        .lock()
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "lock poisoned".into()))?
        .remove(&format!("{locale}::{key}"));

    Ok(Json(serde_json::json!({ "removed": removed })))
}
