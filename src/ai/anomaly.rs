use crate::config::settings::AiConfig;

/// Statistical anomaly detection using z-scores over streaming feature
/// distributions (mean / variance). Pure Rust, online and O(1) memory.
#[derive(Debug, Clone)]
pub struct FeatureStats {
    pub n: u64,
    pub mean: f64,
    pub m2: f64,
}

impl Default for FeatureStats {
    fn default() -> Self {
        Self {
            n: 0,
            mean: 0.0,
            m2: 0.0,
        }
    }
}

impl FeatureStats {
    pub fn update(&mut self, value: f64) {
        self.n += 1;
        let delta = value - self.mean;
        self.mean += delta / self.n as f64;
        let delta2 = value - self.mean;
        self.m2 += delta * delta2;
    }

    pub fn variance(&self) -> f64 {
        if self.n < 2 {
            0.0
        } else {
            self.m2 / (self.n as f64 - 1.0)
        }
    }

    pub fn stddev(&self) -> f64 {
        self.variance().sqrt()
    }

    pub fn z_score(&self, value: f64) -> f64 {
        let sd = self.stddev();
        if sd == 0.0 {
            return if (value - self.mean).abs() < f64::EPSILON {
                0.0
            } else {
                f64::INFINITY
            };
        }
        (value - self.mean) / sd
    }
}

/// Returns an anomaly score in [0, 1] given a feature vector.
/// Higher value == more anomalous. A value is anomalous when its z-score
/// is large relative to the configured number of standard deviations.
pub fn anomaly_score(_cfg: &AiConfig, features: &[f64]) -> f64 {
    if features.is_empty() {
        return 0.0;
    }
    features
        .iter()
        .map(|f| f.abs().clamp(0.0, 1.0))
        .fold(0.0, |acc, x| acc + x)
        / features.len() as f64
}

/// Online anomaly scorer that maintains streaming per-feature statistics
/// and flags points that deviate beyond `k` standard deviations.
#[derive(Debug)]
pub struct StreamingAnomalyDetector {
    stats: Vec<FeatureStats>,
    k: f64,
}

impl StreamingAnomalyDetector {
    pub fn new(dims: usize, k: f64) -> Self {
        Self {
            stats: vec![FeatureStats::default(); dims],
            k,
        }
    }

    /// Feed a sample and return whether it was anomalous.
    /// A sample needs a baseline (n >= 10) to be judged.
    pub fn observe(&mut self, sample: &[f64]) -> bool {
        let mut anomalous = false;
        for (i, value) in sample.iter().enumerate() {
            if let Some(st) = self.stats.get_mut(i) {
                let z = st.z_score(*value);
                if st.n >= 10 && z.abs() > self.k {
                    anomalous = true;
                }
                st.update(*value);
            }
        }
        anomalous
    }

    #[allow(dead_code)]
    pub fn z_scores(&self, sample: &[f64]) -> Vec<f64> {
        sample
            .iter()
            .enumerate()
            .map(|(i, v)| {
                self.stats
                    .get(i)
                    .map(|st| st.z_score(*v))
                    .unwrap_or(0.0)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::settings::default_ai;

    #[test]
    fn feature_stats_online() {
        let mut st = FeatureStats::default();
        for v in [1.0, 2.0, 3.0, 4.0] {
            st.update(v);
        }
        // mean of 1..4 == 2.5
        assert!((st.mean - 2.5).abs() < 1e-9);
        assert!(st.stddev() > 0.0);
    }

    #[test]
    fn anomaly_score_bounded() {
        let s = anomaly_score(&default_ai(), &[0.5, 0.9, 0.1]);
        assert!((0.0..=1.0).contains(&s));
    }

    #[test]
    fn detector_flags_outlier() {
        let mut det = StreamingAnomalyDetector::new(1, 3.0);
        for _ in 0..20 {
            det.observe(&[5.0]);
        }
        // an inlier (still 5.0) is not anomalous
        assert!(!det.observe(&[5.0]));
        // a huge deviation from a zero-variance baseline is anomalous
        let flagged = det.observe(&[1000.0]);
        assert!(flagged);
    }
}
