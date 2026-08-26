use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: String,
    pub version: String,
    pub uptime_secs: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReadinessStatus {
    pub database: bool,
    pub cache: bool,
    pub actors: bool,
}

pub fn health_check() -> HealthStatus {
    HealthStatus {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_secs: 0,
    }
}

pub fn readiness_check() -> ReadinessStatus {
    ReadinessStatus {
        database: true,
        cache: true,
        actors: true,
    }
}
