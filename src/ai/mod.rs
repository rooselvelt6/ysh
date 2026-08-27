pub mod anomaly;
pub mod annealing;
pub mod fuzzy;
pub mod genetic;
pub mod matching;
pub mod network;
pub mod text;

use std::sync::atomic::{AtomicU64, Ordering};

use serde::Serialize;

use crate::config::settings::AiConfig;

pub use text::{ModerateRequest, Moderation, ModerationDecision};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AiModelKind {
    #[default]
    TextModeration,
    Anomaly,
    Matching,
    Fuzzy,
    Genetic,
    Annealing,
    Neural,
}

/// The YSH AI engine. Pure Rust — no external ML runtime.
/// Implements: tiny neural nets, genetic algorithms, fuzzy logic,
/// simulated annealing, text moderation and anomaly detection.
#[derive(Debug)]
pub struct AIEngine {
    pub cfg: AiConfig,
    metrics: AICounters,
}

#[derive(Debug, Default)]
pub struct AICounters {
    moderation_checks: AtomicU64,
    anomalies_flagged: AtomicU64,
    matches_scored: AtomicU64,
    neural_inferences: AtomicU64,
    genetic_generations: AtomicU64,
    annealing_runs: AtomicU64,
}

impl AIEngine {
    pub fn new(cfg: AiConfig) -> Self {
        Self {
            cfg,
            metrics: AICounters::default(),
        }
    }

    pub fn moderate_text(&self, req: ModerateRequest) -> Moderation {
        self.metrics.moderation_checks.fetch_add(1, Ordering::Relaxed);
        text::moderate_text(&self.cfg, req)
    }

    pub fn anomaly_score(&self, features: &[f64]) -> f64 {
        let score = anomaly::anomaly_score(&self.cfg, features);
        if score >= self.cfg.anomaly_flag_threshold {
            self.metrics.anomalies_flagged.fetch_add(1, Ordering::Relaxed);
        }
        score
    }

    pub fn match_score(&self, a: &[f64], b: &[f64]) -> f64 {
        self.metrics.matches_scored.fetch_add(1, Ordering::Relaxed);
        matching::compatibility_score(&self.cfg, a, b)
    }

    pub fn neural_predict(&self, weights: &network::Weights, input: &[f64]) -> f64 {
        self.metrics.neural_inferences.fetch_add(1, Ordering::Relaxed);
        network::forward(&self.cfg, weights, input)
    }

    pub fn genetic_optimize(
        &self,
        fitness: impl Fn(&[f64]) -> f64 + Send + Sync,
        dims: usize,
    ) -> genetic::Individual {
        self.metrics.genetic_generations.fetch_add(1, Ordering::Relaxed);
        genetic::optimize(&self.cfg, fitness, dims)
    }

    pub fn stats(&self) -> serde_json::Value {
        serde_json::json!({
            "moderation_checks": self.metrics.moderation_checks.load(Ordering::Relaxed),
            "anomalies_flagged": self.metrics.anomalies_flagged.load(Ordering::Relaxed),
            "matches_scored": self.metrics.matches_scored.load(Ordering::Relaxed),
            "neural_inferences": self.metrics.neural_inferences.load(Ordering::Relaxed),
            "genetic_generations": self.metrics.genetic_generations.load(Ordering::Relaxed),
            "annealing_runs": self.metrics.annealing_runs.load(Ordering::Relaxed),
        })
    }

    pub fn model_kinds(&self) -> Vec<AiModelKind> {
        vec![
            AiModelKind::TextModeration,
            AiModelKind::Anomaly,
            AiModelKind::Matching,
            AiModelKind::Fuzzy,
            AiModelKind::Genetic,
            AiModelKind::Annealing,
            AiModelKind::Neural,
        ]
    }
}
