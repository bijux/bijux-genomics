//! Owner: bijux-benchmark
//! Deterministic bootstrap confidence intervals using seeded resampling.
//! We prefer bootstrap CIs to preserve distribution shape without assuming normality.
//! Must not perform IO or depend on compare/gate logic.

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BootstrapResult {
    pub mean: f64,
    pub ci_low: f64,
    pub ci_high: f64,
    pub samples: usize,
}

#[allow(
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation
)]
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

#[must_use]
pub fn seed_from_ids(suite_id: &str, metric_id: &str, stage_id: &str, tool_id: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    suite_id.hash(&mut hasher);
    metric_id.hash(&mut hasher);
    stage_id.hash(&mut hasher);
    tool_id.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::{bootstrap_ci, seed_from_ids};

    #[test]
    #[allow(clippy::float_cmp)]
    fn bootstrap_is_deterministic() {
        let values = vec![1.0, 2.0, 3.0, 4.0];
        let seed = seed_from_ids("suite", "runtime_s", "stage", "tool");
        let a = bootstrap_ci(&values, 100, seed);
        let b = bootstrap_ci(&values, 100, seed);
        assert_eq!(a.ci_low, b.ci_low);
        assert_eq!(a.ci_high, b.ci_high);
    }
}
