//! Owner: bijux-dna-bench-model
//! Pure admission of all-domain stage/tool coverage against acquired catalog expectations.

use crate::{AllDomainStageToolTableRow, BenchError, ReadinessStageToolBinding};
use std::collections::BTreeSet;

/// Expected coverage derived independently from domain catalogs in the acquisition owner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AllDomainStageToolTableAdmissionContext {
    pub expected_ready_bindings: BTreeSet<ReadinessStageToolBinding>,
    pub expected_vcf_row_count: usize,
}

/// Validates required columns, unique bindings and governed ready/VCF coverage.
///
/// # Errors
/// Returns [`BenchError::InvalidObservation`] when rows do not match catalog expectations.
pub fn validate_all_domain_stage_tool_table_rows(
    rows: &[AllDomainStageToolTableRow],
    context: &AllDomainStageToolTableAdmissionContext,
) -> Result<(), BenchError> {
    let mut seen = BTreeSet::<ReadinessStageToolBinding>::new();
    for row in rows {
        if !seen.insert(binding_key(&row.domain, &row.stage_id, &row.tool_id)) {
            return Err(invalid(format!(
                "all-domain stage-tool table contains duplicate `{}` / `{}` / `{}`",
                row.domain, row.stage_id, row.tool_id
            )));
        }
        if row.corpus_id.trim().is_empty()
            || row.asset_profile_id.trim().is_empty()
            || row.adapter_id.trim().is_empty()
            || row.parser_id.trim().is_empty()
        {
            return Err(invalid(format!(
                "all-domain stage-tool table row `{}` / `{}` / `{}` contains a blank required column",
                row.domain,
                row.stage_id,
                row.tool_id
            )));
        }
    }

    let expected_ready_keys = &context.expected_ready_bindings;

    let observed_ready_keys = rows
        .iter()
        .filter(|row| row.benchmark_status == "benchmark_ready")
        .map(|row| binding_key(&row.domain, &row.stage_id, &row.tool_id))
        .collect::<BTreeSet<_>>();

    if expected_ready_keys != &observed_ready_keys {
        let missing = expected_ready_keys
            .difference(&observed_ready_keys)
            .map(render_binding_key)
            .collect::<Vec<_>>();
        let extra = observed_ready_keys
            .difference(expected_ready_keys)
            .map(render_binding_key)
            .collect::<Vec<_>>();
        return Err(invalid(format!(
            "all-domain stage-tool table drifted from domain-ready bindings; missing={missing:?} extra={extra:?}"
        )));
    }

    let expected_vcf_row_count = context.expected_vcf_row_count;
    let actual_vcf_row_count = rows.iter().filter(|row| row.domain == "vcf").count();
    if actual_vcf_row_count != expected_vcf_row_count {
        return Err(invalid(format!(
            "all-domain stage-tool table must include every governed VCF matrix row (expected {expected_vcf_row_count}, found {actual_vcf_row_count})"
        )));
    }

    Ok(())
}

fn binding_key(domain: &str, stage_id: &str, tool_id: &str) -> ReadinessStageToolBinding {
    ReadinessStageToolBinding {
        domain: domain.to_string(),
        stage_id: stage_id.to_string(),
        tool_id: tool_id.to_string(),
    }
}

fn render_binding_key(key: &ReadinessStageToolBinding) -> String {
    format!("{}:{}:{}", key.domain, key.stage_id, key.tool_id)
}

fn invalid(reason: String) -> BenchError {
    BenchError::InvalidObservation { reason }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(domain: &str, stage: &str, status: &str) -> AllDomainStageToolTableRow {
        AllDomainStageToolTableRow {
            domain: domain.to_string(),
            stage_id: stage.to_string(),
            tool_id: "tool".to_string(),
            corpus_id: "corpus".to_string(),
            asset_profile_id: "asset".to_string(),
            adapter_id: "adapter".to_string(),
            parser_id: "parser".to_string(),
            benchmark_status: status.to_string(),
        }
    }

    fn evidence() -> (Vec<AllDomainStageToolTableRow>, AllDomainStageToolTableAdmissionContext) {
        (
            vec![
                row("fastq", "fastq.stage", "benchmark_ready"),
                row("vcf", "vcf.stage", "blocked"),
            ],
            AllDomainStageToolTableAdmissionContext {
                expected_ready_bindings: BTreeSet::from([binding_key(
                    "fastq",
                    "fastq.stage",
                    "tool",
                )]),
                expected_vcf_row_count: 1,
            },
        )
    }

    #[test]
    fn coverage_admission_preserves_explicit_blocked_rows() -> Result<(), BenchError> {
        let (rows, context) = evidence();
        let before = rows.clone();
        validate_all_domain_stage_tool_table_rows(&rows, &context)?;
        assert_eq!(rows, before);
        Ok(())
    }

    #[test]
    fn coverage_admission_rejects_duplicate_bindings_and_blank_columns() {
        let (rows, context) = evidence();
        let mut duplicate = rows.clone();
        duplicate.push(rows[0].clone());
        assert!(validate_all_domain_stage_tool_table_rows(&duplicate, &context).is_err());
        for field in 0..4 {
            let mut missing = rows.clone();
            match field {
                0 => missing[0].corpus_id = " ".to_string(),
                1 => missing[0].asset_profile_id.clear(),
                2 => missing[0].adapter_id.clear(),
                _ => missing[0].parser_id.clear(),
            }
            assert!(validate_all_domain_stage_tool_table_rows(&missing, &context).is_err());
        }
    }

    #[test]
    fn coverage_admission_rejects_missing_and_extra_ready_bindings() {
        let (rows, context) = evidence();
        assert!(validate_all_domain_stage_tool_table_rows(&rows[1..], &context).is_err());
        let mut extra = rows;
        extra[1].benchmark_status = "benchmark_ready".to_string();
        assert!(validate_all_domain_stage_tool_table_rows(&extra, &context).is_err());
    }

    #[test]
    fn coverage_admission_requires_every_governed_vcf_row() {
        let (rows, mut context) = evidence();
        context.expected_vcf_row_count = 2;
        assert!(validate_all_domain_stage_tool_table_rows(&rows, &context).is_err());
    }
}
