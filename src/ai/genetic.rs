use crate::config::settings::AiConfig;

#[derive(Debug, Clone)]
pub struct Individual {
    pub genome: Vec<f64>,
    pub fitness: f64,
}

fn random_genome(rng: &mut impl FnMut() -> f64, dims: usize) -> Vec<f64> {
    (0..dims).map(|_| rng()).collect()
}

/// Simple genetic algorithm optimizing a real-valued genome vector.
/// Uses tournament selection, uniform crossover and Gaussian mutation.
pub fn optimize(
    cfg: &AiConfig,
    fitness: impl Fn(&[f64]) -> f64 + Send + Sync,
    dims: usize,
) -> Individual {
    let mut rng_seed = 0x9E3779B97F4A7C15u64;
    let mut next_rand = move || {
        rng_seed ^= rng_seed << 13;
        rng_seed ^= rng_seed >> 7;
        rng_seed ^= rng_seed << 17;
        rng_seed as f64 / u64::MAX as f64
    };

    let mut population: Vec<Individual> = (0..cfg.genetic_population_size)
        .map(|_| {
            let genome = random_genome(&mut next_rand, dims);
            Individual {
                fitness: fitness(&genome),
                genome,
            }
        })
        .collect();

    for _ in 0..cfg.genetic_generations {
        population.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap_or(std::cmp::Ordering::Equal));

        let mut new_pop = Vec::with_capacity(cfg.genetic_population_size);
        while new_pop.len() < cfg.genetic_population_size {
            // tournament selection
            let p1 = population[next_rand() as usize % cfg.genetic_population_size.min(population.len())].clone();
            let idx = next_rand() as usize % cfg.genetic_population_size.min(population.len());
            let p2 = population[idx].clone();

            // uniform crossover
            let mut child = Vec::with_capacity(dims);
            for i in 0..dims {
                let base = if next_rand() < 0.5 {
                    p1.genome[i]
                } else {
                    p2.genome[i]
                };
                // Gaussian-ish mutation
                let mutation = (next_rand() - 0.5) * cfg.genetic_mutation_rate * 2.0;
                child.push((base + mutation).clamp(0.0, 1.0));
            }
            new_pop.push(Individual {
                fitness: fitness(&child),
                genome: child,
            });
        }
        population = new_pop;
    }

    population
        .into_iter()
        .max_by(|a, b| a.fitness.partial_cmp(&b.fitness).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap_or(Individual {
            genome: vec![0.0; dims],
            fitness: 0.0,
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::settings::default_ai;

    #[test]
    fn finds_better_than_average() {
        let cfg = default_ai();
        let best = optimize(&cfg, |g| g.iter().sum::<f64>(), 4);
        let avg: f64 = best.genome.iter().sum();
        // maximizing sum -> genome should be pushed toward 1.0
        assert!(avg > 2.0);
    }
}
