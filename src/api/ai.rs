use axum::{Json, extract::State, http::StatusCode};

use crate::ai::{
    ModerateRequest, ModerationDecision,
    anomaly::StreamingAnomalyDetector,
    matching::{MatchFeatures, vectorize},
    network::Weights,
};
use crate::auth::jwt::AuthUser;
use crate::server::AppState;

pub async fn moderate_text(
    _auth: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let content = req["content"].as_str().unwrap_or("");
    let author_id = req["author_id"].as_str().unwrap_or("anonymous");

    let moderation = state.ai_engine.moderate_text(ModerateRequest {
        content: content.to_string(),
    });

    if moderation.decision == ModerationDecision::Block {
        let _ = state
            .ai_actor
            .send_message(crate::actors::ai_actor::AIActorMsg::Moderate {
                content_id: author_id.to_string(),
                content: Some(content.to_string()),
            });
    }

    Ok(Json(serde_json::json!({
        "decision": moderation.decision,
        "severity": moderation.severity,
        "categories": moderation.categories,
        "matches": moderation.matches,
    })))
}

pub async fn anomaly_score(
    _auth: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let features: Vec<f64> = req["features"]
        .as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_f64()).collect())
        .unwrap_or_default();

    if features.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "features array required".into()));
    }

    let score = state.ai_engine.anomaly_score(&features);
    let flagged = score >= state.ai_engine.cfg.anomaly_flag_threshold;

    Ok(Json(serde_json::json!({
        "score": score,
        "flagged": flagged,
    })))
}

pub async fn match_score(
    _auth: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let a: Vec<f64> = req["a"]
        .as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_f64()).collect())
        .unwrap_or_default();
    let b: Vec<f64> = req["b"]
        .as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_f64()).collect())
        .unwrap_or_default();

    if a.len() != b.len() || a.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "a and b must be equal-length feature vectors".into(),
        ));
    }

    let score = state.ai_engine.match_score(&a, &b);
    Ok(Json(serde_json::json!({ "score": score })))
}

pub async fn neural_predict(
    _auth: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let input: Vec<f64> = req["input"]
        .as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_f64()).collect())
        .unwrap_or_default();
    if input.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "input array required".into()));
    }

    let n_in = input.len();
    let n_hidden = state.ai_engine.cfg.neural_hidden_size;
    let mut weights = Weights::new(n_in, n_hidden);
    weights.randomize();

    let prediction = state.ai_engine.neural_predict(&weights, &input);
    Ok(Json(serde_json::json!({ "prediction": prediction })))
}

pub async fn optimize_genetic(
    _auth: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let dims = req["dims"].as_u64().map(|d| d as usize).unwrap_or(4);
    if dims == 0 {
        return Err((StatusCode::BAD_REQUEST, "dims must be > 0".into()));
    }

    let best = state
        .ai_engine
        .genetic_optimize(|g| g.iter().sum::<f64>(), dims);

    Ok(Json(serde_json::json!({
        "best_genome": best.genome,
        "best_fitness": best.fitness,
    })))
}

pub async fn match_vectorize(
    _auth: AuthUser,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let interests: Vec<String> = req["interests"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect()
        })
        .unwrap_or_default();
    let region = req["region"].as_str().unwrap_or("").to_string();
    let age = req["age"].as_f64().unwrap_or(25.0);

    let features = MatchFeatures {
        interests,
        region,
        age,
        gender: req["gender"].as_str().unwrap_or("").to_string(),
        rating: req["rating"].as_f64().unwrap_or(0.0),
    };
    let vector = vectorize(&features);
    Ok(Json(serde_json::json!({ "vector": vector })))
}

pub async fn neural_train(
    _auth: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let samples: Vec<(Vec<f64>, f64)> = req["samples"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|s| {
                    let input = s["input"].as_array()?;
                    let target = s["target"].as_f64()?;
                    Some((input.iter().filter_map(|v| v.as_f64()).collect(), target))
                })
                .collect()
        })
        .unwrap_or_default();

    if samples.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "samples array required".into()));
    }
    if let Some((first, _)) = samples.first()
        && samples.iter().any(|(inp, _)| inp.len() != first.len())
    {
        return Err((
            StatusCode::BAD_REQUEST,
            "all sample inputs must have the same length".into(),
        ));
    }

    let n_in = samples[0].0.len();
    let n_hidden = state.ai_engine.cfg.neural_hidden_size;
    let mut weights = Weights::new(n_in, n_hidden);
    weights.randomize();

    let loss = crate::ai::network::train_epochs(&mut weights, &samples, 500, 0.3);

    Ok(Json(serde_json::json!({
        "loss": loss,
        "trained": true,
    })))
}

pub async fn ai_stats(
    _auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    Ok(Json(serde_json::json!({
        "enabled": state.ai_engine.cfg.enabled,
        "models": state.ai_engine.model_kinds(),
        "stats": state.ai_engine.stats(),
    })))
}

pub async fn anomaly_detector_demo(
    _auth: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let series: Vec<Vec<f64>> = req["series"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .map(|row| {
                    row.as_array()
                        .map(|x| x.iter().filter_map(|v| v.as_f64()).collect())
                        .unwrap_or_default()
                })
                .collect()
        })
        .unwrap_or_default();

    if series.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "series array required".into()));
    }
    let dims = series[0].len().max(1);
    let mut detector = StreamingAnomalyDetector::new(dims, state.ai_engine.cfg.anomaly_std_devs);
    let mut anomalies = Vec::new();
    for (i, sample) in series.iter().enumerate() {
        if detector.observe(sample) {
            anomalies.push(i);
        }
    }

    Ok(Json(serde_json::json!({
        "anomalous_indices": anomalies,
        "count": anomalies.len(),
    })))
}
