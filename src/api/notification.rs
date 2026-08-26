use axum::{extract::State, http::StatusCode, Json};

use crate::actors::notification_actor::NotificationMsg;
use crate::auth::jwt::AuthUser;
use crate::server::AppState;

pub async fn list_notifications(
    auth: AuthUser,
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id: i64 = auth.user_id.parse().map_err(|_| {
        (StatusCode::UNAUTHORIZED, "Invalid token".into())
    })?;

    let limit: i64 = params
        .get("limit")
        .and_then(|s| s.parse().ok())
        .unwrap_or(50);

    let notifications = state
        .db
        .get_notifications(user_id, limit)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let unread = state
        .db
        .get_unread_count(user_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "notifications": notifications,
        "count": notifications.len(),
        "unread_count": unread,
    })))
}

pub async fn mark_read(
    auth: AuthUser,
    State(state): State<AppState>,
    axum::extract::Path(notification_id): axum::extract::Path<i64>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id: i64 = auth.user_id.parse().map_err(|_| {
        (StatusCode::UNAUTHORIZED, "Invalid token".into())
    })?;

    let updated = state
        .db
        .mark_notification_read(user_id, notification_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if updated {
        Ok(Json(serde_json::json!({"message": "Marked as read"})))
    } else {
        Err((StatusCode::NOT_FOUND, "Notification not found".into()))
    }
}

pub async fn mark_all_read(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id: i64 = auth.user_id.parse().map_err(|_| {
        (StatusCode::UNAUTHORIZED, "Invalid token".into())
    })?;

    let count = state
        .db
        .mark_all_read(user_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "message": "All marked as read",
        "count": count,
    })))
}

pub async fn get_preferences(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id: i64 = auth.user_id.parse().map_err(|_| {
        (StatusCode::UNAUTHORIZED, "Invalid token".into())
    })?;

    let prefs = state
        .db
        .get_notification_preference(user_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(prefs))
}

pub async fn update_preference(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id: i64 = auth.user_id.parse().map_err(|_| {
        (StatusCode::UNAUTHORIZED, "Invalid token".into())
    })?;

    let allowed_fields = [
        "email_enabled",
        "push_enabled",
        "in_app_enabled",
        "email_gifts",
        "email_calls",
        "email_moments",
        "email_marketing",
        "push_gifts",
        "push_calls",
        "push_moments",
    ];

    for field in &allowed_fields {
        if let Some(val) = req.get(*field).and_then(|v| v.as_bool()) {
            state
                .db
                .update_notification_preference(user_id, field, val)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        }
    }

    Ok(Json(serde_json::json!({"message": "Preferences updated"})))
}

pub async fn update_quiet_hours(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id: i64 = auth.user_id.parse().map_err(|_| {
        (StatusCode::UNAUTHORIZED, "Invalid token".into())
    })?;

    let start = req["start"].as_str().unwrap_or("22:00");
    let end = req["end"].as_str().unwrap_or("08:00");

    state
        .db
        .update_quiet_hours(user_id, start, end)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "message": "Quiet hours updated",
        "quiet_hours_start": start,
        "quiet_hours_end": end,
    })))
}

pub async fn register_push_token(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id: i64 = auth.user_id.parse().map_err(|_| {
        (StatusCode::UNAUTHORIZED, "Invalid token".into())
    })?;

    let token = req["token"]
        .as_str()
        .ok_or((StatusCode::BAD_REQUEST, "token required".into()))?;
    let platform = req["platform"].as_str().unwrap_or("web");

    state
        .db
        .register_push_token(user_id, token, platform)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "message": "Push token registered",
        "platform": platform,
    })))
}

pub async fn remove_push_token(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id: i64 = auth.user_id.parse().map_err(|_| {
        (StatusCode::UNAUTHORIZED, "Invalid token".into())
    })?;

    let token = req["token"]
        .as_str()
        .ok_or((StatusCode::BAD_REQUEST, "token required".into()))?;

    let removed = state
        .db
        .deactivate_push_token(user_id, token)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if removed {
        Ok(Json(serde_json::json!({"message": "Token removed"})))
    } else {
        Err((StatusCode::NOT_FOUND, "Token not found".into()))
    }
}

pub async fn get_push_tokens(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id: i64 = auth.user_id.parse().map_err(|_| {
        (StatusCode::UNAUTHORIZED, "Invalid token".into())
    })?;

    let tokens = state
        .db
        .get_push_tokens(user_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "tokens": tokens,
        "count": tokens.len(),
    })))
}

pub async fn send_test_notification(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id: i64 = auth.user_id.parse().map_err(|_| {
        (StatusCode::UNAUTHORIZED, "Invalid token".into())
    })?;

    let notif_id = state
        .db
        .create_notification(
            user_id,
            "test",
            "Test Notification",
            "This is a test notification from YSH.",
            "{}",
            "in_app",
        )
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let _ = state.notification_actor.send_message(NotificationMsg::InApp {
        notification_id: notif_id,
        user_id,
        title: "Test Notification".into(),
        body: "This is a test notification from YSH.".into(),
    });

    Ok(Json(serde_json::json!({
        "message": "Test notification sent",
        "notification_id": notif_id,
    })))
}
