//! Owner: bijux-dna-bench-model
//! Admission of rendered command observations against governed result bindings.

use crate::{
    AllDomainExpectedBenchmarkResultRow, AllDomainRenderedCommandRow, BenchError,
    ReadinessStageToolBinding as BindingKey,
};
use bijux_dna_core::id_catalog;
use std::collections::{BTreeMap, BTreeSet};

/// Admits one executable command row per governed ready binding.
///
/// # Errors
/// Returns [`BenchError::InvalidObservation`] for missing, duplicate or placeholder commands.
pub fn validate_all_domain_rendered_command_rows(
    expected_by_binding: &BTreeMap<BindingKey, AllDomainExpectedBenchmarkResultRow>,
    rows: &[AllDomainRenderedCommandRow],
) -> Result<(), BenchError> {
    if rows.len() != expected_by_binding.len() {
        return Err(BenchError::InvalidObservation { reason: format!(
            "all-domain rendered commands must keep exactly one row per benchmark-ready result binding (expected {}, found {})",
            expected_by_binding.len(),
            rows.len()
        ) });
    }

    validate_rendered_command_observations(rows)?;

    let missing = expected_by_binding
        .iter()
        .filter(|(binding, expected)| {
            !rows.iter().any(|row| {
                row.result_id == expected.result_id
                    && row.domain == binding.domain
                    && row.stage_id == binding.stage_id
                    && row.tool_id == binding.tool_id
            })
        })
        .map(|(_, expected)| expected.result_id.clone())
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(BenchError::InvalidObservation {
            reason: format!(
                "all-domain rendered commands are missing governed result ids: {}",
                missing.join(", ")
            ),
        });
    }

    require_rendered_row(
        rows,
        "fastq:corpus-02-edna-mini:fastq.screen_taxonomy:sample-set:kraken2",
        "fastq",
        id_catalog::FASTQ_SCREEN,
        id_catalog::TOOL_KRAKEN2,
        "fastq_bam_command_adapter",
    )?;
    require_rendered_row(
        rows,
        "bam:corpus-01-kinship-mini:bam.kinship:sample-set:king",
        "bam",
        id_catalog::BAM_KINSHIP,
        id_catalog::TOOL_KING,
        "fastq_bam_command_adapter",
    )?;
    require_rendered_row(
        rows,
        "vcf:vcf_production_regression:vcf.call:bam_bundle:bcftools",
        "vcf",
        id_catalog::VCF_CALL,
        id_catalog::TOOL_BCFTOOLS,
        "vcf_bcftools_adapter",
    )?;

    Ok(())
}

/// Admit intrinsic command identities and executable steps, independently of slice coverage.
///
/// # Errors
/// Rejects missing, duplicate, empty or placeholder command observations.
pub fn validate_rendered_command_observations(
    rows: &[AllDomainRenderedCommandRow],
) -> Result<(), BenchError> {
    if rows.is_empty() {
        return Err(BenchError::InvalidObservation {
            reason: "rendered commands require a nonempty observation set".to_string(),
        });
    }
    let mut seen_result_ids = BTreeSet::<&str>::new();
    let mut seen_bindings = BTreeSet::<BindingKey>::new();
    for row in rows {
        validate_required_columns(row)?;
        if !seen_result_ids.insert(row.result_id.as_str()) {
            return Err(BenchError::InvalidObservation {
                reason: format!(
                    "all-domain rendered commands contain duplicate result_id `{}`",
                    row.result_id
                ),
            });
        }
        if !seen_bindings.insert(binding_key(&row.domain, &row.stage_id, &row.tool_id)) {
            return Err(BenchError::InvalidObservation {
                reason: format!(
                    "all-domain rendered commands contain duplicate `{}` / `{}` / `{}` bindings",
                    row.domain, row.stage_id, row.tool_id
                ),
            });
        }
        validate_rendered_command_row(row)?;
    }

    Ok(())
}

fn validate_required_columns(row: &AllDomainRenderedCommandRow) -> Result<(), BenchError> {
    if row.result_id.trim().is_empty()
        || row.domain.trim().is_empty()
        || row.stage_id.trim().is_empty()
        || row.tool_id.trim().is_empty()
        || row.corpus_id.trim().is_empty()
        || row.asset_profile_id.trim().is_empty()
        || row.readiness_kind.trim().is_empty()
        || row.benchmark_status.trim().is_empty()
        || row.command_source.trim().is_empty()
    {
        return Err(BenchError::InvalidObservation {
            reason: format!(
                "all-domain rendered command row `{}` is missing required columns",
                row.result_id
            ),
        });
    }
    Ok(())
}

fn validate_rendered_command_row(row: &AllDomainRenderedCommandRow) -> Result<(), BenchError> {
    if row.benchmark_status != "benchmark_ready" {
        return Err(BenchError::InvalidObservation {
            reason: format!(
                "all-domain rendered command row `{}` drifted from the benchmark-ready slice",
                row.result_id
            ),
        });
    }
    if row.command_steps.is_empty() || row.script_commands.is_empty() {
        return Err(BenchError::InvalidObservation {
            reason: format!(
                "all-domain rendered command row `{}` must keep command steps and shell commands",
                row.result_id
            ),
        });
    }
    for step in &row.command_steps {
        let executable = step
            .argv
            .first()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .ok_or_else(|| BenchError::InvalidObservation {
                reason: format!(
                    "all-domain rendered command row `{}` step `{}` has an empty executable",
                    row.result_id, step.step_id
                ),
            })?;
        let command_lower = step.command.to_ascii_lowercase();
        if command_lower.contains("todo")
            || command_lower.contains("placeholder")
            || (executable.eq_ignore_ascii_case("echo") && command_lower.contains("echo execute"))
        {
            return Err(BenchError::InvalidObservation { reason: format!(
                    "all-domain rendered command row `{}` step `{}` still contains placeholder execution text",
                    row.result_id,
                    step.step_id
                ) });
        }
    }
    Ok(())
}

fn require_rendered_row(
    rows: &[AllDomainRenderedCommandRow],
    result_id: &str,
    domain: &str,
    stage_id: &str,
    tool_id: &str,
    command_source: &str,
) -> Result<(), BenchError> {
    let row = rows.iter().find(|row| row.result_id == result_id).ok_or_else(|| {
        BenchError::InvalidObservation {
            reason: format!("all-domain rendered commands are missing `{result_id}`"),
        }
    })?;
    if row.domain != domain
        || row.stage_id != stage_id
        || row.tool_id != tool_id
        || row.command_source != command_source
        || row.command_steps.is_empty()
        || row.script_commands.is_empty()
    {
        return Err(BenchError::InvalidObservation {
            reason: format!(
                "all-domain rendered command row `{result_id}` drifted from the governed contract"
            ),
        });
    }
    Ok(())
}

fn binding_key(domain: &str, stage_id: &str, tool_id: &str) -> BindingKey {
    BindingKey {
        domain: domain.to_string(),
        stage_id: stage_id.to_string(),
        tool_id: tool_id.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AllDomainRenderedCommandStep;

    fn evidence(
    ) -> (BTreeMap<BindingKey, AllDomainExpectedBenchmarkResultRow>, Vec<AllDomainRenderedCommandRow>)
    {
        let bindings = [
            (
                "fastq:corpus-02-edna-mini:fastq.screen_taxonomy:sample-set:kraken2",
                "fastq",
                id_catalog::FASTQ_SCREEN,
                id_catalog::TOOL_KRAKEN2,
                "fastq_bam_command_adapter",
            ),
            (
                "bam:corpus-01-kinship-mini:bam.kinship:sample-set:king",
                "bam",
                id_catalog::BAM_KINSHIP,
                id_catalog::TOOL_KING,
                "fastq_bam_command_adapter",
            ),
            (
                "vcf:vcf_production_regression:vcf.call:bam_bundle:bcftools",
                "vcf",
                id_catalog::VCF_CALL,
                id_catalog::TOOL_BCFTOOLS,
                "vcf_bcftools_adapter",
            ),
        ];
        let mut expected = BTreeMap::new();
        let mut rows = Vec::new();
        for (result, domain, stage, tool, source) in bindings {
            expected.insert(
                binding_key(domain, stage, tool),
                AllDomainExpectedBenchmarkResultRow {
                    result_id: result.to_string(),
                    domain: domain.to_string(),
                    stage_id: stage.to_string(),
                    tool_id: tool.to_string(),
                    corpus_id: "corpus".to_string(),
                    asset_profile_id: "asset".to_string(),
                    expected_outputs: vec![],
                    expected_metrics: vec![],
                    report_section: "section".to_string(),
                },
            );
            rows.push(AllDomainRenderedCommandRow {
                result_id: result.to_string(),
                domain: domain.to_string(),
                stage_id: stage.to_string(),
                tool_id: tool.to_string(),
                corpus_id: "corpus".to_string(),
                asset_profile_id: "asset".to_string(),
                readiness_kind: "adapter".to_string(),
                benchmark_status: "benchmark_ready".to_string(),
                command_source: source.to_string(),
                command_steps: vec![AllDomainRenderedCommandStep {
                    step_id: "invoke".to_string(),
                    step_kind: "exec".to_string(),
                    consumes_previous_stdout: false,
                    argv: vec![tool.to_string(), "input with spaces".to_string()],
                    command: format!("{tool} 'input with spaces'"),
                }],
                script_commands: vec![format!("{tool} 'input with spaces'")],
            });
        }
        (expected, rows)
    }

    #[test]
    fn admission_preserves_command_tokens_and_pipeline_handoff() -> Result<(), BenchError> {
        let (expected, mut rows) = evidence();
        let mut second = rows[2].command_steps[0].clone();
        second.step_id = "consume".to_string();
        second.consumes_previous_stdout = true;
        rows[2].command_steps.push(second);
        let before = rows.clone();
        validate_all_domain_rendered_command_rows(&expected, &rows)?;
        assert_eq!(rows, before);
        Ok(())
    }

    #[test]
    fn admission_rejects_missing_duplicate_and_misbound_results() {
        let (expected, rows) = evidence();
        assert!(validate_all_domain_rendered_command_rows(&expected, &rows[1..]).is_err());
        let mut duplicate = rows.clone();
        duplicate[1].result_id = rows[0].result_id.clone();
        assert!(validate_all_domain_rendered_command_rows(&expected, &duplicate).is_err());
        let mut duplicate_binding = rows.clone();
        duplicate_binding[1].domain = rows[0].domain.clone();
        duplicate_binding[1].stage_id = rows[0].stage_id.clone();
        duplicate_binding[1].tool_id = rows[0].tool_id.clone();
        assert!(validate_all_domain_rendered_command_rows(&expected, &duplicate_binding).is_err());
        let mut misbound = rows.clone();
        misbound[0].result_id = rows[1].result_id.clone();
        misbound[1].result_id = rows[0].result_id.clone();
        assert!(validate_all_domain_rendered_command_rows(&expected, &misbound).is_err());
    }

    #[test]
    fn admission_rejects_empty_and_placeholder_execution() {
        let (expected, rows) = evidence();
        for mutation in 0..7 {
            let mut invalid = rows.clone();
            match mutation {
                0 => invalid[0].command_steps.clear(),
                1 => invalid[0].script_commands.clear(),
                2 => invalid[0].command_steps[0].argv.clear(),
                3 => invalid[0].command_steps[0].argv[0] = " ".to_string(),
                4 => invalid[0].command_steps[0].command = "TODO run tool".to_string(),
                5 => invalid[0].command_steps[0].command = "tool placeholder".to_string(),
                _ => {
                    invalid[0].command_steps[0].argv[0] = "echo".to_string();
                    invalid[0].command_steps[0].command = "echo execute tool".to_string();
                }
            }
            assert!(validate_all_domain_rendered_command_rows(&expected, &invalid).is_err());
        }
    }

    #[test]
    fn admission_rejects_blank_identity_blocked_status_and_wrong_adapter_source() {
        let (expected, rows) = evidence();
        for field in 0..11 {
            let mut invalid = rows.clone();
            let row = &mut invalid[0];
            match field {
                0 => row.result_id.clear(),
                1 => row.domain.clear(),
                2 => row.stage_id.clear(),
                3 => row.tool_id.clear(),
                4 => row.corpus_id.clear(),
                5 => row.asset_profile_id.clear(),
                6 => row.readiness_kind.clear(),
                7 => row.benchmark_status.clear(),
                8 => row.command_source.clear(),
                9 => row.benchmark_status = "blocked".to_string(),
                _ => row.command_source = "unrelated_adapter".to_string(),
            }
            assert!(validate_all_domain_rendered_command_rows(&expected, &invalid).is_err());
        }
    }
}
