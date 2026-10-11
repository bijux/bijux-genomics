//! Owner: bijux-dna-bench-model
//! Numeric admission before scoring or normalization can hide invalid observations.

use crate::VcfToolScoringInput;
use anyhow::{bail, Result};

pub(super) fn validate_input(input: &VcfToolScoringInput) -> Result<()> {
    for ((stage, tool), evidence) in &input.evidence {
        for (field, value) in [
            ("truth_correctness_score", evidence.truth_correctness_score),
            ("contract_correctness_score", evidence.contract_correctness_score),
        ] {
            validate_non_negative(stage, tool, field, value)?;
            if value.is_some_and(|value| value > 1.0) {
                bail!(
                    "VCF scoring evidence `{stage}` / `{tool}` field `{field}` must be in [0, 1]"
                );
            }
        }
        for (field, value) in [
            ("genotype_truth_metric_value", evidence.genotype_truth_metric_value),
            ("missingness_metric_value", evidence.missingness_metric_value),
            ("phasing_imputation_metric_value", evidence.phasing_imputation_metric_value),
            ("population_metric_value", evidence.population_metric_value),
        ] {
            if value.is_some_and(|value| !value.is_finite()) {
                bail!("VCF scoring evidence `{stage}` / `{tool}` field `{field}` must be finite");
            }
        }
    }
    for ((stage, tool), evidence) in &input.evidence {
        validate_non_negative(stage, tool, "runtime_seconds", evidence.runtime_seconds)?;
    }
    for row in input.runtime_rows.iter().filter(|row| row.domain == "vcf") {
        validate_non_negative(&row.stage_id, &row.tool_id, "elapsed_seconds", row.elapsed_seconds)?;
    }
    for row in input.memory_rows.iter().filter(|row| row.domain == "vcf") {
        validate_non_negative(
            &row.stage_id,
            &row.tool_id,
            "observed_memory_mb",
            row.observed_memory_mb,
        )?;
        validate_non_negative(
            &row.stage_id,
            &row.tool_id,
            "declared_memory_mb",
            row.declared_memory_mb,
        )?;
    }
    Ok(())
}

fn validate_non_negative(stage: &str, tool: &str, field: &str, value: Option<f64>) -> Result<()> {
    if value.is_some_and(|value| !value.is_finite() || value < 0.0) {
        bail!("VCF scoring evidence `{stage}` / `{tool}` field `{field}` must be finite and non-negative");
    }
    Ok(())
}
