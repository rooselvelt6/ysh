use serde::{Deserialize, Serialize};

use crate::config::settings::AiConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModerationDecision {
    Allow,
    Flag,
    Block,
}

#[derive(Debug, Clone, Serialize)]
pub struct Moderation {
    pub decision: ModerationDecision,
    pub severity: f64,
    pub categories: Vec<String>,
    pub matches: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModerateRequest {
    pub content: String,
}

const HARDCORE_BLOCKED: &[&str] = &[
    "scam",
    "scammer",
    "fraud",
    "phishing",
    "money laundering",
    "carding",
    "crack",
    "keygen",
    "warez",
    "cp ",
    "child porn",
    "underage",
    "minors",
    "buy followers",
    "nudes for cash",
    "chargeback scam",
    "blackmail",
];

const FLAG_WORDS: &[&str] = &[
    "sexually explicit",
    "nude",
    "leak",
    "pirate",
    "drugs",
    "weapons",
    "violence",
    "harassment",
    "doxx",
    "spam",
    "adult site",
    "casino",
];

fn has_term(content: &str, terms: &[&str]) -> Vec<String> {
    let lower = content.to_lowercase();
    terms
        .iter()
        .filter(|t| lower.contains(**t))
        .map(|t| (*t).to_string())
        .collect()
}

pub fn moderate_text(cfg: &AiConfig, req: ModerateRequest) -> Moderation {
    let blocked = has_term(&req.content, HARDCORE_BLOCKED);
    let flagged = has_term(&req.content, FLAG_WORDS);

    let mut categories = Vec::new();
    if !blocked.is_empty() {
        categories.push("blocked".to_string());
    }
    if !flagged.is_empty() {
        categories.push("flagged".to_string());
    }

    let base = blocked.len() as f64 * 0.5 + flagged.len() as f64 * 0.2;
    let length_penalty = (req.content.chars().count() as f64).min(1000.0) / 1000.0;
    let severity = (base + length_penalty * cfg.text_moderation_sensitivity).clamp(0.0, 1.0);

    let mut all_matches = blocked.clone();
    all_matches.extend(flagged);

    let decision = if !blocked.is_empty() {
        ModerationDecision::Block
    } else if severity >= cfg.text_moderation_flag_threshold {
        ModerationDecision::Flag
    } else {
        ModerationDecision::Allow
    };

    Moderation {
        decision,
        severity,
        categories,
        matches: all_matches,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::settings::default_ai;

    #[test]
    fn allows_clean_content() {
        let m = moderate_text(
            &default_ai(),
            ModerateRequest {
                content: "hello nice to meet you".into(),
            },
        );
        assert_eq!(m.decision, ModerationDecision::Allow);
    }

    #[test]
    fn blocks_hardcore_content() {
        let m = moderate_text(
            &default_ai(),
            ModerateRequest {
                content: "buy cheap carding tools here".into(),
            },
        );
        assert_eq!(m.decision, ModerationDecision::Block);
        assert!(!m.matches.is_empty());
    }

    #[test]
    fn flags_sensitive_content() {
        let m = moderate_text(
            &default_ai(),
            ModerateRequest {
                content: "discussing drug policy".into(),
            },
        );
        assert_ne!(m.decision, ModerationDecision::Block);
    }
}
