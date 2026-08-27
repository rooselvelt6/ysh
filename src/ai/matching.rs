use crate::config::settings::AiConfig;

#[derive(Debug, Clone)]
pub struct MatchFeatures {
    pub interests: Vec<String>,
    pub region: String,
    pub age: f64,
    pub gender: String,
    pub rating: f64,
}

/// Vectorize a user's text features (interests/region) into a normalized
/// bag-of-words vector. Deterministic per feature set.
pub fn vectorize(features: &MatchFeatures) -> Vec<f64> {
    let keywords: Vec<&str> = vec![
        "music", "games", "sports", "travel", "tech", "food", "art",
        "movies", "fitness", "fashion", "business", "education",
    ];
    let mut vec = vec![0.0; keywords.len() + 4];
    for interest in &features.interests {
        for (i, kw) in keywords.iter().enumerate() {
            if interest.to_lowercase().contains(kw) {
                vec[i] = 1.0;
            }
        }
    }
    let base = keywords.len();
    vec[base] = features.region.bytes().map(|b| b as f64).sum::<f64>().rem_euclid(2.0);
    vec[base + 1] = if features.gender.to_lowercase() == "f" { 1.0 } else { 0.0 };
    vec[base + 2] = (features.age / 100.0).clamp(0.0, 1.0);
    vec[base + 3] = (features.rating / 5.0).clamp(0.0, 1.0);
    vec
}

/// Cosine similarity between two vectors.
pub fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let mut dot = 0.0;
    let mut na = 0.0;
    let mut nb = 0.0;
    for (x, y) in a.iter().zip(b.iter()) {
        dot += x * y;
        na += x * x;
        nb += y * y;
    }
    let denom = na.sqrt() * nb.sqrt();
    if denom == 0.0 {
        0.0
    } else {
        dot / denom
    }
}

/// Combines raw similarity with fuzzy classification to produce a final
/// compatibility score in [0, 1].
pub fn compatibility_score(cfg: &AiConfig, a: &[f64], b: &[f64]) -> f64 {
    let raw = cosine_similarity(a, b);
    let set = crate::ai::fuzzy::load_system_config_fuzzy_set();
    let degree = set.degree(raw, "high");
    (raw * 0.7 + degree * 0.3).clamp(0.0, 1.0) * cfg.matching_score_scale
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::settings::default_ai;

    fn user(interests: &[&str], age: f64) -> MatchFeatures {
        MatchFeatures {
            interests: interests.iter().map(|s| s.to_string()).collect(),
            region: "mx".into(),
            age,
            gender: "f".into(),
            rating: 4.0,
        }
    }

    #[test]
    fn similar_users_score_higher() {
        let cfg = default_ai();
        let a = vectorize(&user(&["music", "sports"], 25.0));
        let b = vectorize(&user(&["music", "sports", "games"], 27.0));
        let c = vectorize(&user(&["business", "education"], 40.0));
        let score_ab = compatibility_score(&cfg, &a, &b);
        let score_ac = compatibility_score(&cfg, &a, &c);
        assert!(score_ab > score_ac, "ab={score_ab} ac={score_ac}");
    }

    #[test]
    fn cosine_identical_is_one() {
        let a = vec![1.0, 0.0, 1.0];
        let s = cosine_similarity(&a, &a);
        assert!((s - 1.0).abs() < 1e-9);
    }
}
