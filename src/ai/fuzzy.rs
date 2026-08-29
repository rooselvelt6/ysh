/// Lightweight fuzzy logic engine: membership functions, fuzzy set
/// evaluation and a simple rule-based inference used for QoS adaptation
/// and user classification.
#[derive(Debug, Clone)]
pub struct Membership {
    pub name: &'static str,
    pub center: f64,
    pub width: f64,
}

impl Membership {
    /// Triangular membership function in [0, 1].
    pub fn mf(&self, x: f64) -> f64 {
        let d = (x - self.center).abs();
        if d >= self.width {
            0.0
        } else {
            1.0 - d / self.width
        }
    }
}

/// Classifies a crisp input into fuzzy linguistic labels with degrees.
#[derive(Debug, Clone)]
pub struct FuzzySet {
    pub labels: Vec<Membership>,
}

impl FuzzySet {
    #[allow(dead_code)]
    pub fn fuzzify(&self, x: f64) -> Vec<(&'static str, f64)> {
        self.labels.iter().map(|m| (m.name, m.mf(x))).collect()
    }

    pub fn degree(&self, x: f64, name: &'static str) -> f64 {
        self.labels
            .iter()
            .find(|m| m.name == name)
            .map(|m| m.mf(x))
            .unwrap_or(0.0)
    }
}

pub fn load_system_config_fuzzy_set() -> FuzzySet {
    FuzzySet {
        labels: vec![
            Membership {
                name: "low",
                center: 0.0,
                width: 0.4,
            },
            Membership {
                name: "medium",
                center: 0.5,
                width: 0.4,
            },
            Membership {
                name: "high",
                center: 1.0,
                width: 0.4,
            },
        ],
    }
}

/// Defuzzify using the centroid over a set of output membership functions.
#[allow(dead_code)]
pub fn defuzzify(pairs: &[(f64, f64)]) -> f64 {
    let mut num = 0.0;
    let mut den = 0.0;
    for (center, degree) in pairs {
        num += center * degree;
        den += degree;
    }
    if den == 0.0 { 0.0 } else { num / den }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn membership_at_center_is_one() {
        let m = Membership {
            name: "high",
            center: 1.0,
            width: 0.4,
        };
        assert!((m.mf(1.0) - 1.0).abs() < 1e-9);
        assert!(m.mf(2.0) < 1e-9);
    }

    #[test]
    fn fuzzify_returns_degrees() {
        let set = load_system_config_fuzzy_set();
        let labels = set.fuzzify(0.5);
        assert_eq!(labels.len(), 3);
        let medium = set.degree(0.5, "medium");
        assert!(medium > 0.9);
    }

    #[test]
    fn centroid_defuzzifies() {
        let out = defuzzify(&[(0.0, 0.0), (1.0, 1.0)]);
        assert!((out - 1.0).abs() < 1e-9);
    }
}
