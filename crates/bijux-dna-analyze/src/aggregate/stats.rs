use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct RobustSummary {
    pub n: usize,
    pub median: f64,
    pub mad: f64,
    pub iqr: f64,
    pub trimmed_mean: f64,
    pub outlier_count: usize,
    pub high_variance: bool,
}

fn median(sorted: &[f64]) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let mid = sorted.len() / 2;
    if sorted.len() % 2 == 0 {
        (sorted[mid - 1] + sorted[mid]) * 0.5
    } else {
        sorted[mid]
    }
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]
fn percentile(sorted: &[f64], pct: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((sorted.len() - 1) as f64 * pct).round() as usize;
    sorted[idx]
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]
fn trimmed_mean(sorted: &[f64], trim_ratio: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let n = sorted.len();
    let trim = (n as f64 * trim_ratio).round() as usize;
    let start = trim.min(n);
    let end = n.saturating_sub(trim);
    if start >= end {
        return median(sorted);
    }
    let slice = &sorted[start..end];
    let sum: f64 = slice.iter().sum();
    let denom = f64::from(u32::try_from(slice.len()).unwrap_or(u32::MAX));
    sum / denom
}

#[must_use]
pub fn robust_summary(values: &[f64]) -> RobustSummary {
    if values.is_empty() {
        return RobustSummary {
            n: 0,
            median: 0.0,
            mad: 0.0,
            iqr: 0.0,
            trimmed_mean: 0.0,
            outlier_count: 0,
            high_variance: false,
        };
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let median_value = median(&sorted);
    let mut deviations: Vec<f64> = sorted.iter().map(|v| (v - median_value).abs()).collect();
    deviations.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mad_value = median(&deviations);
    let q1 = percentile(&sorted, 0.25);
    let q3 = percentile(&sorted, 0.75);
    let iqr = (q3 - q1).max(0.0);
    let trimmed = trimmed_mean(&sorted, 0.1);
    let upper = q3 + 1.5 * iqr;
    let lower = q1 - 1.5 * iqr;
    let outlier_count = sorted
        .iter()
        .filter(|value| **value > upper || **value < lower)
        .count();
    let high_variance = iqr > median_value.abs().max(1e-6) * 0.5;
    RobustSummary {
        n: sorted.len(),
        median: median_value,
        mad: mad_value,
        iqr,
        trimmed_mean: trimmed,
        outlier_count,
        high_variance,
    }
}
