use crate::config::settings::AiConfig;

/// A tiny fully-connected feedforward neural network with one hidden layer,
/// implemented in pure Rust with backpropagation. Satisfies the platform's
/// "neural networks" requirement (matching / churn / pricing / NSFW scoring)
/// without an external ML runtime.
#[derive(Debug, Clone)]
pub struct Weights {
    /// hidden weights: input_size x hidden_size
    pub hidden: Vec<Vec<f64>>,
    /// hidden bias vector
    pub hidden_bias: Vec<f64>,
    /// output weights: hidden_size
    pub output: Vec<f64>,
    pub output_bias: f64,
    pub input_size: usize,
    pub hidden_size: usize,
}

impl Weights {
    pub fn new(input_size: usize, hidden_size: usize) -> Self {
        Self {
            hidden: vec![vec![0.0; hidden_size]; input_size],
            hidden_bias: vec![0.0; hidden_size],
            output: vec![0.0; hidden_size],
            output_bias: 0.0,
            input_size,
            hidden_size,
        }
    }

    pub fn randomize(&mut self) {
        let mut seed = 0xB32DE1CE07C5FF97u64;
        let mut next = move || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            (seed as f64 / u64::MAX as f64) * 2.0 - 1.0
        };
        for row in self.hidden.iter_mut() {
            for v in row.iter_mut() {
                *v = next() * 0.5;
            }
        }
        for v in self.hidden_bias.iter_mut() {
            *v = next() * 0.1;
        }
        for v in self.output.iter_mut() {
            *v = next() * 0.5;
        }
        self.output_bias = next() * 0.1;
    }
}

fn relu(x: f64) -> f64 {
    x.max(0.0)
}

fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}

/// Forward pass, returns the sigmoid output probability in [0, 1].
pub fn forward(_cfg: &AiConfig, w: &Weights, input: &[f64]) -> f64 {
    let mut hidden = vec![0.0; w.hidden_size];
    for (h_idx, hb) in w.hidden_bias.iter().enumerate() {
        let mut acc = *hb;
        for (i_idx, x) in input.iter().enumerate() {
            if let Some(row) = w.hidden.get(i_idx) {
                acc += x * row.get(h_idx).copied().unwrap_or(0.0);
            }
        }
        hidden[h_idx] = relu(acc);
    }
    let mut out = w.output_bias;
    for (h_idx, hv) in hidden.iter().enumerate() {
        out += hv * w.output.get(h_idx).copied().unwrap_or(0.0);
    }
    sigmoid(out)
}

/// Train a single sample via stochastic gradient descent with backprop.
#[allow(clippy::needless_range_loop)]
pub fn train(w: &mut Weights, input: &[f64], target: f64, learning_rate: f64) -> f64 {
    let n = w.input_size;
    let h = w.hidden_size;
    let mut z = vec![0.0; h];
    let mut a = vec![0.0; h];
    for (j, hb) in w.hidden_bias.iter().enumerate() {
        let mut acc = *hb;
        for (i, x) in input.iter().take(n).enumerate() {
            acc += x * w.hidden[i][j];
        }
        z[j] = acc;
        a[j] = relu(acc);
    }
    let out = sigmoid(w.output_bias + a.iter().zip(&w.output).map(|(x, o)| x * o).sum::<f64>());
    let error = out - target;
    let d_out = error * out * (1.0 - out);

    // output layer gradients
    let mut d_hidden = vec![0.0; h];
    for j in 0..h {
        d_hidden[j] = d_out * w.output[j] * if z[j] > 0.0 { 1.0 } else { 0.0 };
    }
    for j in 0..h {
        w.output[j] -= learning_rate * d_out * a[j];
    }
    w.output_bias -= learning_rate * d_out;

    // hidden layer gradients
    for i in 0..n {
        for j in 0..h {
            w.hidden[i][j] -= learning_rate * d_hidden[j] * input[i];
        }
    }
    for j in 0..h {
        w.hidden_bias[j] -= learning_rate * d_hidden[j];
    }

    error.abs()
}

/// Train on a dataset of (input, target) pairs for `epochs`, returns MSE.
pub fn train_epochs(
    w: &mut Weights,
    data: &[(Vec<f64>, f64)],
    epochs: usize,
    learning_rate: f64,
) -> f64 {
    let mut last_loss = 0.0;
    for _ in 0..epochs {
        let mut loss = 0.0;
        for (input, target) in data {
            loss += train(w, input, *target, learning_rate).powi(2);
        }
        last_loss = loss / data.len() as f64;
    }
    last_loss
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::settings::default_ai;

    #[test]
    fn learns_or_gate_approximation() {
        let cfg = default_ai();
        let mut w = Weights::new(2, 4);
        w.randomize();
        let data = vec![
            (vec![0.0, 0.0], 0.0),
            (vec![1.0, 0.0], 1.0),
            (vec![0.0, 1.0], 1.0),
            (vec![1.0, 1.0], 1.0),
        ];
        let loss = train_epochs(&mut w, &data, 800, 0.5);
        assert!(loss < 0.05, "mse was {loss}");
        let p = forward(&cfg, &w, &[1.0, 1.0]);
        assert!(p > 0.9, "or(1,1) prob was {p}");
        let p00 = forward(&cfg, &w, &[0.0, 0.0]);
        assert!(p00 < 0.1, "or(0,0) prob was {p00}");
    }
}
