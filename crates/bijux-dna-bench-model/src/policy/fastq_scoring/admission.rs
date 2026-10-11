//! Numeric admission before scoring or normalization can hide invalid observations.

use crate::FastqToolScoringInput;
use anyhow::{bail, Result};

pub(super) fn validate_input(input: &FastqToolScoringInput) -> Result<()> {
    for ((stage, tool), evidence) in &input.evidence {
        for (field, value) in [
            ("truth_correctness_score", evidence.truth_correctness_score),
            ("contract_correctness_score", evidence.contract_correctness_score),
        ] {
            validate_non_negative(stage, tool, field, value)?;
            if value.is_some_and(|value| value > 1.0) {
                bail!(
                    "FASTQ scoring evidence `{stage}` / `{tool}` field `{field}` must be in [0, 1]"
                );
            }
        }
    }
    for row in input.runtime_rows.iter().filter(|row| row.domain == "fastq") {
        validate_non_negative(&row.stage_id, &row.tool_id, "elapsed_seconds", row.elapsed_seconds)?;
    }
    for row in input.memory_rows.iter().filter(|row| row.domain == "fastq") {
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
        bail!("FASTQ scoring evidence `{stage}` / `{tool}` field `{field}` must be finite and non-negative");
    }
    Ok(())
}
