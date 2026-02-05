//! Owner: bijux-benchmark
//! Outlier detection policies.
//! Must not perform IO or depend on compare/gate logic.

use crate::stats::robust::{mad, median};

#[derive(Debug, Clone)]
pub struct OutlierReport {
    pub outlier_count: usize,
    pub outlier_indices: Vec<usize>,
}

#[must_use]
pub fn mad_outliers(values: &[f64], threshold: f64) -> OutlierReport {
    if values.is_empty() {
        return OutlierReport {
            outlier_count: 0,
            outlier_indices: Vec::new(),
        };
    }
    let med = median(values.to_vec());
    let mad_value = mad(values);
    let mut outlier_indices = Vec::new();
    let scale = if mad_value.abs() < f64::EPSILON {
        1.0
    } else {
        mad_value
    };
    for (idx, value) in values.iter().enumerate() {
        let score = (value - med).abs() / scale;
        if score > threshold {
            outlier_indices.push(idx);
        }
    }
    OutlierReport {
        outlier_count: outlier_indices.len(),
        outlier_indices,
    }
}

#[cfg(test)]
mod tests {
    use super::mad_outliers;

    #[test]
    fn outliers_detected() {
        let values = vec![1.0, 1.1, 0.9, 10.0];
        let report = mad_outliers(&values, 3.5);
        assert_eq!(report.outlier_count, 1);
    }
}
