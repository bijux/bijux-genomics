use anyhow::{anyhow, Result};

use crate::aggregate::{metric_kind_for_stage, metric_spec, stage_metric_spec};

/// Print the benchmark schema for a stage.
///
/// # Errors
/// Returns an error if the schema cannot be rendered.
pub fn print_bench_schema(stage: &str) -> Result<()> {
    let json = bench_schema_json(stage)?;
    println!("{}", serde_json::to_string_pretty(&json)?);
    Ok(())
}

/// Build the benchmark schema as JSON for a stage.
///
/// # Errors
/// Returns an error if the stage is unknown or serialization fails.
pub fn bench_schema_json(stage: &str) -> Result<serde_json::Value> {
    let kind = metric_kind_for_stage(stage).ok_or_else(|| anyhow!("unknown stage {stage}"))?;
    let spec = stage_metric_spec(kind);
    let metrics: Vec<_> = spec
        .metrics
        .iter()
        .map(|metric_id| {
            let metric = metric_spec(*metric_id);
            serde_json::json!({
                "name": metric.name,
                "meaning": metric.meaning,
                "direction": format!("{:?}", metric.direction),
                "range": metric.range.map(|range| serde_json::json!({
                    "min": range.min,
                    "max": range.max
                })),
                "measured": metric.measured,
                "derived": metric.derived,
            })
        })
        .collect();
    let derived: Vec<_> = spec
        .metrics
        .iter()
        .filter_map(|metric_id| {
            let metric = metric_spec(*metric_id);
            if metric.derived {
                Some(metric.name.to_string())
            } else {
                None
            }
        })
        .collect();
    Ok(serde_json::json!({
        "stage": stage,
        "schema_version": format!("{}_v{}", stage.replace('.', "_"), spec.version),
        "metrics": metrics,
        "derived": derived,
        "invariants": spec.invariants,
    }))
}
