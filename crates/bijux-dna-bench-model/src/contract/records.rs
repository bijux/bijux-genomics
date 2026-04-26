//! Owner: bijux-dna-bench
//! Validation for non-suite benchmark contract records.

use crate::diagnostics::BenchError;
use crate::model::{BenchmarkObservation, BenchmarkSummary, MetricSummary, SummaryRow};
use crate::policy::GateDecision;

use super::{DECISION_SCHEMA_V1, OBSERVATION_SCHEMA_V1, SUMMARY_SCHEMA_V1};

/// # Errors
/// Returns an error if required confounders are missing.
pub fn validate_observation(obs: &BenchmarkObservation) -> Result<(), BenchError> {
    if obs.schema_version != OBSERVATION_SCHEMA_V1 {
        return Err(BenchError::InvalidObservation {
            reason: format!("observation schema mismatch: {}", obs.schema_version),
        });
    }
    obs.validate()?;
    Ok(())
}

/// # Errors
/// Returns an error if summary schema is invalid.
pub fn validate_summary(summary: &BenchmarkSummary) -> Result<(), BenchError> {
    if summary.schema_version != SUMMARY_SCHEMA_V1 {
        return Err(BenchError::InvalidPolicy(format!(
            "summary schema mismatch: {}",
            summary.schema_version
        )));
    }
    required_text(&summary.suite_id, "summary.suite_id")?;
    for row in &summary.rows {
        validate_summary_row(row)?;
    }
    Ok(())
}

/// # Errors
/// Returns an error if decision schema is invalid.
pub fn validate_decision(decision: &GateDecision) -> Result<(), BenchError> {
    if decision.schema_version != DECISION_SCHEMA_V1 {
        return Err(BenchError::InvalidPolicy(format!(
            "decision schema mismatch: {}",
            decision.schema_version
        )));
    }
    Ok(())
}

fn validate_summary_row(row: &SummaryRow) -> Result<(), BenchError> {
    required_text(&row.dataset_id, "summary row dataset_id")?;
    required_text(&row.dataset_class, "summary row dataset_class")?;
    required_text(&row.read_layout, "summary row read_layout")?;
    required_text(&row.stage_id, "summary row stage_id")?;
    required_text(&row.tool_id, "summary row tool_id")?;
    required_text(&row.params_hash, "summary row params_hash")?;
    finite_ratio(row.failure_rate, "summary row failure_rate")?;
    finite_ratio(row.completeness, "summary row completeness")?;
    if row.n_effective > row.runtime.n {
        return Err(BenchError::InvalidPolicy(format!(
            "summary row n_effective {} exceeds runtime n {}",
            row.n_effective, row.runtime.n
        )));
    }
    validate_metric_summary(&row.runtime)?;
    validate_metric_summary(&row.memory)?;
    for metric in &row.metrics {
        validate_metric_summary(metric)?;
    }
    Ok(())
}

fn validate_metric_summary(metric: &MetricSummary) -> Result<(), BenchError> {
    required_text(&metric.metric_id, "summary metric_id")?;
    if metric.stats.n != metric.n {
        return Err(BenchError::InvalidPolicy(format!(
            "summary metric {} stats n {} does not match n {}",
            metric.metric_id, metric.stats.n, metric.n
        )));
    }
    finite_value(metric.stats.median, "summary metric median")?;
    finite_value(metric.stats.mad, "summary metric mad")?;
    finite_value(metric.stats.iqr, "summary metric iqr")?;
    finite_value(metric.stats.trimmed_mean, "summary metric trimmed_mean")?;
    if let Some(ci_low) = metric.ci_low {
        finite_value(ci_low, "summary metric ci_low")?;
    }
    if let Some(ci_high) = metric.ci_high {
        finite_value(ci_high, "summary metric ci_high")?;
    }
    if let (Some(ci_low), Some(ci_high)) = (metric.ci_low, metric.ci_high) {
        if ci_low > ci_high {
            return Err(BenchError::InvalidPolicy(format!(
                "summary metric {} ci_low exceeds ci_high",
                metric.metric_id
            )));
        }
    }
    if metric.outlier_count != metric.outlier_replicates.len() {
        return Err(BenchError::InvalidPolicy(format!(
            "summary metric {} outlier_count does not match outlier_replicates",
            metric.metric_id
        )));
    }
    if let Some(threshold) = metric.practical_threshold {
        finite_value(threshold, "summary metric practical_threshold")?;
        if threshold < 0.0 {
            return Err(BenchError::InvalidPolicy(format!(
                "summary metric {} practical_threshold must be non-negative",
                metric.metric_id
            )));
        }
    }
    Ok(())
}

fn required_text(value: &str, field: &str) -> Result<(), BenchError> {
    if value.trim().is_empty() {
        return Err(BenchError::InvalidPolicy(format!("missing {field}")));
    }
    Ok(())
}

fn finite_ratio(value: f64, field: &str) -> Result<(), BenchError> {
    if !value.is_finite() {
        return Err(BenchError::InvalidPolicy(format!("{field} must be finite")));
    }
    if !(0.0..=1.0).contains(&value) {
        return Err(BenchError::InvalidPolicy(format!("{field} must be between 0 and 1")));
    }
    Ok(())
}

fn finite_value(value: f64, field: &str) -> Result<(), BenchError> {
    if !value.is_finite() {
        return Err(BenchError::InvalidPolicy(format!("{field} must be finite")));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use anyhow::bail;

    use crate::contract::SUMMARY_SCHEMA_V1;
    use crate::model::{BenchmarkSummary, MetricSummary, SummaryRow};
    use crate::stats::robust_estimators::RobustStats;

    use super::validate_summary;

    fn metric_summary(metric_id: &str, n: usize) -> MetricSummary {
        MetricSummary {
            metric_id: metric_id.to_string(),
            n,
            stats: RobustStats { n, median: 1.0, mad: 0.0, iqr: 0.0, trimmed_mean: 1.0 },
            ci_low: Some(1.0),
            ci_high: Some(1.0),
            outlier_count: 0,
            outlier_replicates: Vec::new(),
            practical_threshold: None,
            power_warning: false,
        }
    }

    fn valid_summary() -> BenchmarkSummary {
        BenchmarkSummary {
            schema_version: SUMMARY_SCHEMA_V1.to_string(),
            suite_id: "suite-1".to_string(),
            rows: vec![SummaryRow {
                dataset_id: "dataset-1".to_string(),
                dataset_class: "trueseq".to_string(),
                read_layout: "paired".to_string(),
                stage_id: "fastq.trim_reads".to_string(),
                stage_instance_id: None,
                lineage_id: None,
                tool_id: "fastp".to_string(),
                params_hash: "params-a".to_string(),
                runtime: metric_summary("runtime_s", 3),
                memory: metric_summary("memory_mb", 3),
                metrics: Vec::new(),
                failure_rate: 0.0,
                completeness: 1.0,
                n_effective: 3,
                low_power: false,
            }],
            strata: Vec::new(),
            warnings: Vec::new(),
            scientifically_invalid: false,
            invalid_reasons: Vec::new(),
        }
    }

    #[test]
    fn summary_rejects_missing_row_identifier() -> anyhow::Result<()> {
        let mut summary = valid_summary();
        summary.rows[0].dataset_id.clear();

        let Err(err) = validate_summary(&summary) else {
            bail!("summary without dataset_id should fail");
        };

        assert!(err.to_string().contains("missing summary row dataset_id"));
        Ok(())
    }

    #[test]
    fn summary_rejects_invalid_ratio() -> anyhow::Result<()> {
        let mut summary = valid_summary();
        summary.rows[0].completeness = 1.1;

        let Err(err) = validate_summary(&summary) else {
            bail!("summary with invalid completeness should fail");
        };

        assert!(err.to_string().contains("summary row completeness"));
        Ok(())
    }

    #[test]
    fn summary_rejects_effective_n_above_runtime_n() -> anyhow::Result<()> {
        let mut summary = valid_summary();
        summary.rows[0].n_effective = 4;

        let Err(err) = validate_summary(&summary) else {
            bail!("summary with impossible n_effective should fail");
        };

        assert!(err.to_string().contains("n_effective 4 exceeds runtime n 3"));
        Ok(())
    }

    #[test]
    fn summary_rejects_metric_count_mismatch() -> anyhow::Result<()> {
        let mut summary = valid_summary();
        summary.rows[0].runtime.stats.n = 2;

        let Err(err) = validate_summary(&summary) else {
            bail!("summary with mismatched metric count should fail");
        };

        assert!(err.to_string().contains("stats n 2 does not match n 3"));
        Ok(())
    }

    #[test]
    fn summary_rejects_unordered_metric_ci() -> anyhow::Result<()> {
        let mut summary = valid_summary();
        summary.rows[0].memory.ci_low = Some(2.0);
        summary.rows[0].memory.ci_high = Some(1.0);

        let Err(err) = validate_summary(&summary) else {
            bail!("summary with unordered CI should fail");
        };

        assert!(err.to_string().contains("ci_low exceeds ci_high"));
        Ok(())
    }

    #[test]
    fn summary_rejects_outlier_metadata_mismatch() -> anyhow::Result<()> {
        let mut summary = valid_summary();
        summary.rows[0].runtime.outlier_count = 1;

        let Err(err) = validate_summary(&summary) else {
            bail!("summary with mismatched outlier metadata should fail");
        };

        assert!(err.to_string().contains("outlier_count does not match"));
        Ok(())
    }
}
