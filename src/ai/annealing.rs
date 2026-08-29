use crate::config::settings::AiConfig;

/// Simulated annealing minimization over a bounded scalar domain.
/// Used for resource/parameter optimization (e.g. pricing, QoS).
#[allow(dead_code)]
pub fn minimize(cfg: &AiConfig, cost: impl Fn(f64) -> f64, start: f64, min: f64, max: f64) -> f64 {
    let mut rng_seed = 0x2545F4914F6CDD1Du64;
    let mut next_rand = move || {
        rng_seed ^= rng_seed << 13;
        rng_seed ^= rng_seed >> 7;
        rng_seed ^= rng_seed << 17;
        rng_seed as f64 / u64::MAX as f64
    };

    let mut current = start.clamp(min, max);
    let mut best = current;
    let mut best_cost = cost(current);
    let mut current_cost = best_cost;
    let cooling = cfg.annealing_cooling_factor;
    let mut temp = cfg.annealing_start_temp;

    for _ in 0..cfg.annealing_iterations {
        let step = (next_rand() - 0.5) * (max - min) * cfg.annealing_step_size;
        let candidate = (current + step).clamp(min, max);
        let candidate_cost = cost(candidate);
        let delta = candidate_cost - current_cost;

        if delta < 0.0 || next_rand() < (-delta / temp.max(1e-9)).exp() {
            current = candidate;
            current_cost = candidate_cost;
            if candidate_cost < best_cost {
                best = candidate;
                best_cost = candidate_cost;
            }
        }
        temp *= cooling;
    }

    best
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::settings::default_ai;

    #[test]
    fn finds_minimum_of_parabola() {
        let cfg = default_ai();
        // cost minimized at x=0.5 within [0,1]
        let best = minimize(&cfg, |x| (x - 0.5).powi(2), 0.0, 0.0, 1.0);
        assert!((best - 0.5).abs() < 0.1, "best was {best}");
    }
}
