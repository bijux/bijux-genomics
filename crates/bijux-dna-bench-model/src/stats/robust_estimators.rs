//! Owner: bijux-dna-bench
//! Robust statistics: median, MAD, IQR, and trimmed mean.
//! These estimators reduce sensitivity to outliers compared to mean/stddev.
//! Must not perform IO or depend on compare/gate logic.
//! Invariants: ordering is deterministic.

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RobustStats {
    pub n: usize,
    pub median: f64,
    pub mad: f64,
    pub iqr: f64,
    pub trimmed_mean: f64,
}

#[must_use]
pub fn median(mut values: Vec<f64>) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mid = values.len() / 2;
    if values.len() % 2 == 0 {
        (values[mid - 1] + values[mid]) * 0.5
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

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]
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
pub fn robust_stats(values: &[f64]) -> RobustStats {
    if values.is_empty() {
        return RobustStats {
            n: 0,
            median: 0.0,
            mad: 0.0,
            iqr: 0.0,
            trimmed_mean: 0.0,
        };
    }
    RobustStats {
        n: values.len(),
        median: median(values.to_vec()),
        mad: mad(values),
        iqr: iqr(values),
        trimmed_mean: trimmed_mean(values, 0.1),
    }
}

#[cfg(test)]
mod tests {
    use super::{iqr, mad, median, robust_stats, trimmed_mean};

    #[test]
    fn robust_stats_basic() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 100.0];
        assert!((median(values.clone()) - 3.0).abs() < 1e-6);
        assert!((mad(&values) - 1.0).abs() < 1e-6);
        assert!((iqr(&values) - 2.0).abs() < 1e-6);
        assert!((trimmed_mean(&values, 0.2) - 3.0).abs() < 1e-6);
        let stats = robust_stats(&values);
        assert_eq!(stats.n, 5);
    }
}
