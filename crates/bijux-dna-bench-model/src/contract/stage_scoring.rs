//! Pure admission of scientific scoring policy, independent of repository IO.

use crate::{
    StageScoringApplicability, StageScoringConfig, StageScoringCorrectnessSignal,
    StageScoringDecisionMode, StageScoringRow, REQUIRED_ALL_DOMAIN_FAILURE_CLASSES,
    STAGE_SCORING_SCHEMA_VERSION,
};
use anyhow::{bail, Result};
use std::collections::BTreeSet;

/// Admit versioned scoring policy and the scientific and operational meaning of each row.
/// Repository paths and active-stage coverage are admitted by the repository caller.
///
/// # Errors
/// Rejects unsupported versions, missing failure classes, duplicate rows, invalid weights,
/// inconsistent scientific thresholds, completion requirements, or ranking modes.
pub fn validate_stage_scoring_config(config: &StageScoringConfig) -> Result<()> {
    if config.schema_version != STAGE_SCORING_SCHEMA_VERSION {
        bail!(
            "stage scoring schema drift: expected `{STAGE_SCORING_SCHEMA_VERSION}`, found `{}`",
            config.schema_version
        );
    }
    if config.rows.is_empty() {
        bail!("stage scoring config requires at least one scoring row");
    }
    let failure_class_ids =
        config.failure_classes.iter().map(|row| row.class_id.as_str()).collect::<BTreeSet<_>>();
    if failure_class_ids.len() != config.failure_classes.len()
        || failure_class_ids.len() != REQUIRED_ALL_DOMAIN_FAILURE_CLASSES.len()
    {
        bail!("stage scoring config requires each declared failure class exactly once");
    }
    for class_id in REQUIRED_ALL_DOMAIN_FAILURE_CLASSES {
        if !failure_class_ids.contains(class_id) {
            bail!("stage scoring config is missing failure class `{class_id}`");
        }
    }

    let mut seen_stage_ids = BTreeSet::<(&str, &str)>::new();
    for row in &config.rows {
        if !matches!(row.domain.as_str(), "fastq" | "bam" | "vcf")
            || row
                .stage_id
                .split_once('.')
                .is_none_or(|(domain, stage)| domain != row.domain || stage.is_empty())
            || row.stage_id.trim() != row.stage_id
        {
            bail!("stage scoring row has an invalid domain/stage identity");
        }
        let tool_ids =
            row.benchmark_ready_tool_ids.iter().map(String::as_str).collect::<BTreeSet<_>>();
        if tool_ids.is_empty()
            || tool_ids.len() != row.benchmark_ready_tool_ids.len()
            || tool_ids.len() != row.benchmark_ready_tool_count
            || row.active_tool_count < row.benchmark_ready_tool_count
            || tool_ids.iter().any(|tool| tool.trim().is_empty() || tool.trim() != *tool)
        {
            bail!(
                "stage scoring row `{}` has inconsistent benchmark-ready tool identities or counts",
                row.stage_id
            );
        }
        if !seen_stage_ids.insert((row.domain.as_str(), row.stage_id.as_str())) {
            bail!(
                "stage scoring config contains duplicate row `{}` / `{}`",
                row.domain,
                row.stage_id
            );
        }
        if !row.benchmark_ready_tool_ids.iter().any(|tool_id| tool_id == &row.default_tool_id) {
            bail!(
                "stage scoring row `{}` / `{}` keeps default tool `{}` outside benchmark-ready tool ids",
                row.domain,
                row.stage_id,
                row.default_tool_id
            );
        }
        if row.report_section_ids.is_empty() {
            bail!(
                "stage scoring row `{}` / `{}` is missing report_section_ids",
                row.domain,
                row.stage_id
            );
        }
        ensure_weight_contract(row)?;
        ensure_scientific_threshold_contract(row)?;
        ensure_runtime_memory_completion_contract(row)?;
        ensure_failure_class_contract(row, &failure_class_ids)?;
        ensure_decision_mode_contract(row)?;
    }
    Ok(())
}

fn ensure_weight_contract(row: &StageScoringRow) -> Result<()> {
    for weight in [
        row.weights.correctness,
        row.weights.scientific_threshold,
        row.weights.runtime,
        row.weights.memory,
        row.weights.completion,
        row.weights.failure_class,
    ] {
        if !weight.is_finite() || weight < 0.0 {
            bail!(
                "stage scoring row `{}` / `{}` requires finite non-negative weights",
                row.domain,
                row.stage_id
            );
        }
    }
    let sum = row.weights.correctness
        + row.weights.scientific_threshold
        + row.weights.runtime
        + row.weights.memory
        + row.weights.completion
        + row.weights.failure_class;
    if (sum - 1.0).abs() > 1e-9 {
        bail!(
            "stage scoring row `{}` / `{}` has weights summing to {sum} instead of 1.0",
            row.domain,
            row.stage_id
        );
    }
    Ok(())
}

fn ensure_scientific_threshold_contract(row: &StageScoringRow) -> Result<()> {
    match row.scientific_threshold.applicability {
        StageScoringApplicability::Required => {
            if row.scientific_threshold.metric_ids.is_empty() {
                bail!(
                    "stage scoring row `{}` / `{}` requires scientific thresholds but has no metric_ids",
                    row.domain,
                    row.stage_id
                );
            }
            if row.weights.scientific_threshold <= 0.0 {
                bail!(
                    "stage scoring row `{}` / `{}` requires scientific thresholds but gives them zero weight",
                    row.domain,
                    row.stage_id
                );
            }
        }
        StageScoringApplicability::NotApplicable => {
            if !row.scientific_threshold.metric_ids.is_empty() {
                bail!(
                    "stage scoring row `{}` / `{}` marks scientific thresholds not_applicable but still declares metric_ids",
                    row.domain,
                    row.stage_id
                );
            }
            if row.weights.scientific_threshold != 0.0 {
                bail!(
                    "stage scoring row `{}` / `{}` marks scientific thresholds not_applicable but still assigns non-zero weight",
                    row.domain,
                    row.stage_id
                );
            }
        }
    }
    Ok(())
}

fn ensure_runtime_memory_completion_contract(row: &StageScoringRow) -> Result<()> {
    if row.runtime.metric_id != "elapsed_seconds" {
        bail!(
            "stage scoring row `{}` / `{}` must use runtime metric `elapsed_seconds`, found `{}`",
            row.domain,
            row.stage_id,
            row.runtime.metric_id
        );
    }
    if row.memory.metric_id != "observed_memory_mb" {
        bail!(
            "stage scoring row `{}` / `{}` must use memory metric `observed_memory_mb`, found `{}`",
            row.domain,
            row.stage_id,
            row.memory.metric_id
        );
    }
    if row.memory.fallback_metric_id != "declared_memory_mb" {
        bail!(
            "stage scoring row `{}` / `{}` must fall back to `declared_memory_mb`, found `{}`",
            row.domain,
            row.stage_id,
            row.memory.fallback_metric_id
        );
    }
    if row.completion.required_row_statuses != ["present".to_string()] {
        bail!(
            "stage scoring row `{}` / `{}` must require full-benchmark row_status `present`",
            row.domain,
            row.stage_id
        );
    }
    if row.completion.required_execution_statuses != ["succeeded".to_string()] {
        bail!(
            "stage scoring row `{}` / `{}` must require micro execution_status `succeeded`",
            row.domain,
            row.stage_id
        );
    }
    if !row.completion.required_success_ratio.is_finite()
        || (row.completion.required_success_ratio - 1.0).abs() > 1e-9
    {
        bail!(
            "stage scoring row `{}` / `{}` must require success ratio 1.0, found {}",
            row.domain,
            row.stage_id,
            row.completion.required_success_ratio
        );
    }
    Ok(())
}

fn ensure_failure_class_contract(
    row: &StageScoringRow,
    failure_class_ids: &BTreeSet<&str>,
) -> Result<()> {
    if row.failure_class.blocking_class_ids.is_empty() {
        bail!(
            "stage scoring row `{}` / `{}` is missing blocking failure classes",
            row.domain,
            row.stage_id
        );
    }
    for class_id in row
        .failure_class
        .blocking_class_ids
        .iter()
        .chain(row.failure_class.insufficient_class_ids.iter())
    {
        if !failure_class_ids.contains(class_id.as_str()) {
            bail!(
                "stage scoring row `{}` / `{}` references unknown failure class `{}`",
                row.domain,
                row.stage_id,
                class_id
            );
        }
    }
    Ok(())
}

fn ensure_decision_mode_contract(row: &StageScoringRow) -> Result<()> {
    match row.decision_mode {
        StageScoringDecisionMode::MultiToolRanking => {
            if row.benchmark_ready_tool_count < 2 {
                bail!(
                    "stage scoring row `{}` / `{}` claims multi_tool_ranking with only {} benchmark-ready tools",
                    row.domain,
                    row.stage_id,
                    row.benchmark_ready_tool_count
                );
            }
            let expected_signal =
                if row.scientific_threshold.applicability == StageScoringApplicability::Required {
                    StageScoringCorrectnessSignal::ScientificComparableMetrics
                } else {
                    StageScoringCorrectnessSignal::OutputContract
                };
            if row.correctness.signal != expected_signal {
                bail!(
                    "stage scoring row `{}` / `{}` must use correctness signal `{:?}` for its multi-tool ranking contract",
                    row.domain,
                    row.stage_id,
                    expected_signal
                );
            }
        }
        StageScoringDecisionMode::SingleToolAcceptance => {
            if row.benchmark_ready_tool_count != 1 {
                bail!(
                    "stage scoring row `{}` / `{}` claims single_tool_acceptance with {} benchmark-ready tools",
                    row.domain,
                    row.stage_id,
                    row.benchmark_ready_tool_count
                );
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy() -> Result<StageScoringConfig> {
        Ok(serde_json::from_str(include_str!("../../tests/fixtures/stage_scoring/policy.json"))?)
    }

    #[test]
    fn stage_scoring_policy_preserves_authored_wire_fields() -> Result<()> {
        let config = policy()?;
        validate_stage_scoring_config(&config)?;
        let original: serde_json::Value =
            serde_json::from_str(include_str!("../../tests/fixtures/stage_scoring/policy.json"))?;
        assert_eq!(serde_json::to_value(&config)?, original);
        let mut unknown = original;
        unknown["rows"][0]["weights"]["speed"] = serde_json::json!(0.1);
        assert!(serde_json::from_value::<StageScoringConfig>(unknown).is_err());
        Ok(())
    }

    #[test]
    fn stage_scoring_policy_rejects_non_finite_and_negative_weights() -> Result<()> {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -0.1] {
            let mut config = policy()?;
            config.rows[0].weights.runtime = value;
            assert!(validate_stage_scoring_config(&config).is_err(), "{value}");
        }
        let mut balanced = policy()?;
        balanced.rows[0].weights.correctness += balanced.rows[0].weights.runtime + 0.1;
        balanced.rows[0].weights.runtime = -0.1;
        assert!(validate_stage_scoring_config(&balanced).is_err());
        for value in [f64::NAN, f64::INFINITY, 0.5] {
            let mut config = policy()?;
            config.rows[0].completion.required_success_ratio = value;
            assert!(validate_stage_scoring_config(&config).is_err());
        }
        Ok(())
    }

    #[test]
    fn stage_scoring_policy_rejects_inconsistent_scientific_decisions() -> Result<()> {
        let mutations: [fn(&mut StageScoringConfig); 7] = [
            |c| c.schema_version = "unknown".into(),
            |c| c.failure_classes.clear(),
            |c| c.rows.push(c.rows[0].clone()),
            |c| c.rows[0].default_tool_id = "unregistered".into(),
            |c| c.rows[0].report_section_ids.clear(),
            |c| c.rows[0].benchmark_ready_tool_count = 0,
            |c| c.rows[0].runtime.metric_id = "milliseconds".into(),
        ];
        for mutate in mutations {
            let mut config = policy()?;
            mutate(&mut config);
            assert!(validate_stage_scoring_config(&config).is_err());
        }
        Ok(())
    }
    #[test]
    fn stage_scoring_policy_rejects_ambiguous_inventory_and_tool_cardinality() -> Result<()> {
        let mutations: [fn(&mut StageScoringConfig); 10] = [
            |c| c.rows.clear(),
            |c| c.failure_classes.push(c.failure_classes[0].clone()),
            |c| c.failure_classes[0].class_id = "undeclared".into(),
            |c| c.rows[0].stage_id = "fastq.align".into(),
            |c| c.rows[0].stage_id = "bam.".into(),
            |c| c.rows[0].benchmark_ready_tool_count += 1,
            |c| c.rows[0].active_tool_count = 0,
            |c| {
                let duplicate = c.rows[0].default_tool_id.clone();
                c.rows[0].benchmark_ready_tool_ids.push(duplicate);
            },
            |c| c.rows[0].benchmark_ready_tool_ids[0] = String::new(),
            |c| c.rows[0].benchmark_ready_tool_ids[0].push(' '),
        ];
        for mutate in mutations {
            let mut config = policy()?;
            mutate(&mut config);
            assert!(validate_stage_scoring_config(&config).is_err());
        }
        Ok(())
    }

    #[test]
    fn stage_scoring_policy_admits_the_complete_authored_inventory() -> Result<()> {
        let config: StageScoringConfig =
            toml::from_str(include_str!("../../../../configs/bench/local/stage-scoring.toml"))?;
        validate_stage_scoring_config(&config)?;
        Ok(())
    }
}
