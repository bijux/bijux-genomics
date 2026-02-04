use std::path::Path;

use anyhow::{Context, Result};
use serde::Serialize;

use bijux_core::FactsRowV1;
use bijux_core::ObjectiveSpec;

use crate::aggregate::stats::{robust_summary, RobustSummary};
use crate::decision::effect::{default_thresholds, effect_size};
use crate::decision::{DecisionMetricTrace, DecisionTrace};
use crate::model::JsonBlob;
use crate::semantics::resolve_semantics;

#[derive(Debug, Serialize)]
pub struct RunComparison {
    pub metrics_a: JsonBlob,
    pub metrics_b: JsonBlob,
    pub objective: String,
    pub run_a: String,
    pub run_b: String,
    pub uncertainty: CompareUncertainty,
    pub baseline: Option<String>,
    pub baseline_metrics: Option<JsonBlob>,
    pub deltas_vs_baseline: Option<CompareBaseline>,
    pub regression_risk: Option<RegressionRiskSummary>,
}

#[derive(Debug, Serialize)]
pub struct CompareUncertainty {
    pub runtime_ci: Option<(f64, f64)>,
    pub memory_ci: Option<(f64, f64)>,
    pub read_retention_ci: Option<(f64, f64)>,
    pub note: String,
}

#[derive(Debug, Serialize)]
pub struct CompareBaseline {
    pub deltas_a: JsonBlob,
    pub deltas_b: JsonBlob,
}

#[derive(Debug, Serialize)]
pub struct RegressionRisk {
    pub worsened: Vec<String>,
    pub improved: Vec<String>,
    pub unchanged: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct RegressionRiskSummary {
    pub run_a: RegressionRisk,
    pub run_b: RegressionRisk,
    pub note: String,
}

#[derive(Debug, Serialize)]
pub struct CompareRobustStats {
    pub runtime_s: RobustSummary,
    pub memory_mb: RobustSummary,
    pub read_retention: RobustSummary,
    pub flags: Vec<String>,
}

fn load_json(path: &Path) -> Result<JsonBlob> {
    let data = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let value: JsonBlob = serde_json::from_str(&data)?;
    Ok(value)
}

/// Compare two runs using their summary metrics.
///
/// # Errors
/// Returns an error if metrics cannot be loaded.
pub fn compare_runs(
    run_a: &Path,
    run_b: &Path,
    objective: &ObjectiveSpec,
) -> Result<RunComparison> {
    let metrics_a = load_json(&run_a.join("summary").join("metrics_deltas.json"))?;
    let metrics_b = load_json(&run_b.join("summary").join("metrics_deltas.json"))?;
    Ok(RunComparison {
        metrics_a,
        metrics_b,
        objective: objective.name.clone(),
        run_a: run_a.display().to_string(),
        run_b: run_b.display().to_string(),
        uncertainty: CompareUncertainty {
            runtime_ci: None,
            memory_ci: None,
            read_retention_ci: None,
            note: "ci_not_computed".to_string(),
        },
        baseline: None,
        baseline_metrics: None,
        deltas_vs_baseline: None,
        regression_risk: None,
    })
}

fn regression_risk_for(deltas: &JsonBlob) -> RegressionRisk {
    let mut worsened = Vec::new();
    let mut improved = Vec::new();
    let mut unchanged = Vec::new();
    let Some(map) = deltas.as_value().as_object() else {
        return RegressionRisk {
            worsened,
            improved,
            unchanged,
        };
    };
    for (metric, value) in map {
        let Some(delta) = value.as_f64() else {
            continue;
        };
        let semantics = resolve_semantics(metric);
        let direction = semantics
            .as_ref()
            .map(|spec| spec.direction.as_str())
            .unwrap_or("HigherBetter");
        let epsilon = 1e-9_f64;
        if delta.abs() <= epsilon {
            unchanged.push(metric.clone());
            continue;
        }
        let is_worse = match direction {
            "LowerBetter" => delta > 0.0,
            _ => delta < 0.0,
        };
        if is_worse {
            worsened.push(metric.clone());
        } else {
            improved.push(metric.clone());
        }
    }
    worsened.sort();
    improved.sort();
    unchanged.sort();
    RegressionRisk {
        worsened,
        improved,
        unchanged,
    }
}

/// Compare two runs against a shared baseline.
///
/// # Errors
/// Returns an error if metrics cannot be loaded.
pub fn compare_runs_with_baseline(
    run_a: &Path,
    run_b: &Path,
    baseline: &Path,
    objective: &ObjectiveSpec,
) -> Result<RunComparison> {
    let metrics_a = load_json(&run_a.join("summary").join("metrics_deltas.json"))?;
    let metrics_b = load_json(&run_b.join("summary").join("metrics_deltas.json"))?;
    let baseline_metrics = load_json(&baseline.join("summary").join("metrics_deltas.json"))?;
    let deltas_a = JsonBlob::numeric_deltas(&metrics_a, &baseline_metrics);
    let deltas_b = JsonBlob::numeric_deltas(&metrics_b, &baseline_metrics);
    Ok(RunComparison {
        metrics_a,
        metrics_b,
        objective: objective.name.clone(),
        run_a: run_a.display().to_string(),
        run_b: run_b.display().to_string(),
        uncertainty: CompareUncertainty {
            runtime_ci: None,
            memory_ci: None,
            read_retention_ci: None,
            note: "ci_not_computed".to_string(),
        },
        baseline: Some(baseline.display().to_string()),
        baseline_metrics: Some(baseline_metrics),
        deltas_vs_baseline: Some(CompareBaseline {
            deltas_a: deltas_a.clone(),
            deltas_b: deltas_b.clone(),
        }),
        regression_risk: Some(RegressionRiskSummary {
            run_a: regression_risk_for(&deltas_a),
            run_b: regression_risk_for(&deltas_b),
            note: "risk_vs_baseline".to_string(),
        }),
    })
}

/// Compute robust summary stats for runtime/memory/retention.
///
/// # Errors
/// Returns an error if required metric semantics are missing.
pub fn compare_robust_stats(rows: &[FactsRowV1]) -> Result<CompareRobustStats> {
    assert_metric_semantics(&["runtime_s", "memory_mb", "read_retention"])?;
    let runtime_values: Vec<f64> = rows.iter().map(|row| row.runtime_s).collect();
    let memory_values: Vec<f64> = rows.iter().map(|row| row.memory_mb).collect();
    let retention_values: Vec<f64> = rows
        .iter()
        .filter_map(|row| match (row.reads_in, row.reads_out) {
            #[allow(clippy::cast_precision_loss)]
            (Some(ri), Some(ro)) if ri > 0 => Some(ro as f64 / ri as f64),
            _ => None,
        })
        .collect();
    let runtime = robust_summary(&runtime_values);
    let memory = robust_summary(&memory_values);
    let retention = robust_summary(&retention_values);
    let mut flags = Vec::new();
    if runtime.n < 3 || memory.n < 3 || retention.n < 3 {
        flags.push("sample_size_too_small".to_string());
    }
    if runtime.high_variance || memory.high_variance || retention.high_variance {
        flags.push("high_variance".to_string());
    }
    if runtime.outlier_count > 0 || memory.outlier_count > 0 || retention.outlier_count > 0 {
        flags.push("outliers_detected".to_string());
    }
    Ok(CompareRobustStats {
        runtime_s: runtime,
        memory_mb: memory,
        read_retention: retention,
        flags,
    })
}

#[must_use]
pub fn trace_for_robust_stats(stats: &CompareRobustStats) -> DecisionTrace {
    let mut trace = DecisionTrace::empty();
    let thresholds = default_thresholds();
    trace.per_metric = vec![
        DecisionMetricTrace {
            metric_id: "runtime_s".to_string(),
            value: Some(stats.runtime_s.median),
            weight: 1.0,
            contribution: stats.runtime_s.median,
            effect: Some(effect_size(
                stats.runtime_s.trimmed_mean,
                stats.runtime_s.median,
                thresholds,
            )),
        },
        DecisionMetricTrace {
            metric_id: "memory_mb".to_string(),
            value: Some(stats.memory_mb.median),
            weight: 1.0,
            contribution: stats.memory_mb.median,
            effect: Some(effect_size(
                stats.memory_mb.trimmed_mean,
                stats.memory_mb.median,
                thresholds,
            )),
        },
        DecisionMetricTrace {
            metric_id: "read_retention".to_string(),
            value: Some(stats.read_retention.median),
            weight: 1.0,
            contribution: stats.read_retention.median,
            effect: Some(effect_size(
                stats.read_retention.trimmed_mean,
                stats.read_retention.median,
                thresholds,
            )),
        },
    ];
    trace.penalties.clone_from(&stats.flags);
    trace
}

fn assert_metric_semantics(metric_ids: &[&str]) -> Result<()> {
    for metric_id in metric_ids {
        resolve_semantics(metric_id).with_context(|| {
            format!(
                "missing metric semantics for {metric_id}; remediation: register in bijux-core metrics_registry"
            )
        })?;
    }
    Ok(())
}
