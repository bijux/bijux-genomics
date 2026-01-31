//! Owner: bijux-bench
//! Robust statistics and uncertainty estimates.
//! Owns stable stats computations and bootstrap CI.
//! Must not perform IO or depend on compare/gate logic.
//! Invariants: ordering is deterministic.

#[derive(Debug, Clone)]
pub struct BootstrapResult {
    pub mean: f64,
    pub ci_low: f64,
    pub ci_high: f64,
    pub samples: usize,
}

#[must_use]
pub fn median(mut values: Vec<f64>) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mid = values.len() / 2;
    if values.len() % 2 == 0 {
        values[mid - 1].midpoint(values[mid])
    } else {
        values[mid]
    }
}

#[must_use]
pub fn mad(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let med = median(values.to_vec());
    let deviations: Vec<f64> = values.iter().map(|v| (v - med).abs()).collect();
    median(deviations)
}

#[must_use]
pub fn iqr(values: &[f64]) -> f64 {
    if values.len() < 4 {
        return 0.0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let q1 = sorted[sorted.len() / 4];
    let q3 = sorted[sorted.len() * 3 / 4];
    q3 - q1
}

#[must_use]
pub fn trimmed_mean(values: &[f64], trim_ratio: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let trim = ((sorted.len() as f64) * trim_ratio).floor() as usize;
    let slice = &sorted[trim..sorted.len().saturating_sub(trim)];
    if slice.is_empty() {
        return 0.0;
    }
    slice.iter().sum::<f64>() / slice.len() as f64
}

#[must_use]
pub fn bootstrap_ci(values: &[f64], samples: usize, seed: u64) -> BootstrapResult {
    if values.is_empty() || samples == 0 {
        return BootstrapResult {
            mean: 0.0,
            ci_low: 0.0,
            ci_high: 0.0,
            samples: 0,
        };
    }
    let mut rng = fastrand::Rng::with_seed(seed);
    let mut means = Vec::with_capacity(samples);
    for _ in 0..samples {
        let mut acc = 0.0;
        for _ in 0..values.len() {
            let idx = rng.usize(..values.len());
            acc += values[idx];
        }
        means.push(acc / values.len() as f64);
    }
    means.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mean = means.iter().sum::<f64>() / means.len() as f64;
    let low_idx = (means.len() as f64 * 0.025).floor() as usize;
    let high_idx = ((means.len() as f64 * 0.975).floor() as usize).min(means.len() - 1);
    BootstrapResult {
        mean,
        ci_low: means[low_idx],
        ci_high: means[high_idx],
        samples,
    }
}
