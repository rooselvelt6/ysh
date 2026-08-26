use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

use crate::auth::jwt::AuthUser;
use crate::server::AppState;

pub async fn create_agency(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id: i64 = auth.user_id.parse().map_err(|_| {
        (StatusCode::UNAUTHORIZED, "Invalid token".into())
    })?;

    let name = req["name"]
        .as_str()
        .ok_or((StatusCode::BAD_REQUEST, "name required".into()))?;
    let description = req["description"].as_str().unwrap_or("");

    let agency_id = state
        .db
        .create_agency(user_id, name, description)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    state
        .db
        .add_agency_member(agency_id, user_id, "owner")
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "id": agency_id,
        "name": name,
        "description": description,
        "message": "Agency created",
    })))
}

pub async fn get_agency(
    State(state): State<AppState>,
    Path(agency_id): Path<i64>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let agency = state
        .db
        .get_agency(agency_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    match agency {
        Some(a) => Ok(Json(a)),
        None => Err((StatusCode::NOT_FOUND, "Agency not found".into())),
    }
}

pub async fn list_agencies(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let agencies = state
        .db
        .list_agencies()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "agencies": agencies,
        "count": agencies.len(),
    })))
}

pub async fn add_member(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(agency_id): Path<i64>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let _owner_id: i64 = auth.user_id.parse().map_err(|_| {
        (StatusCode::UNAUTHORIZED, "Invalid token".into())
    })?;

    let target_user_id = req["user_id"]
        .as_i64()
        .ok_or((StatusCode::BAD_REQUEST, "user_id required".into()))?;
    let role = req["role"].as_str().unwrap_or("host");

    state
        .db
        .add_agency_member(agency_id, target_user_id, role)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "message": "Member added",
        "user_id": target_user_id,
        "role": role,
    })))
}

pub async fn get_members(
    State(state): State<AppState>,
    Path(agency_id): Path<i64>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let members = state
        .db
        .get_agency_members(agency_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "members": members,
        "count": members.len(),
    })))
}
