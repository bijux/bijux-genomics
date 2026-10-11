use super::model::{VcfMissingResultReportRow, VcfMissingResultStatus};
use super::paths::{fixture_manifest_path, path_relative_to_repo, result_row_id};
use anyhow::Context;
use anyhow::{anyhow, Result};
use bijux_dna_bench_artifacts::results::load_benchmark_stage_result_manifest as load_validated_stage_result_manifest_path;
use bijux_dna_bench_vcf_evidence::model::expectations::VcfExpectedBenchmarkResultRow;
use std::path::Path;
pub(super) fn collect_vcf_missing_result_report_rows(
    repo_root: &Path,
    fake_result_root: &Path,
    expected_rows: &[VcfExpectedBenchmarkResultRow],
) -> Result<Vec<VcfMissingResultReportRow>> {
    let mut rows = Vec::with_capacity(expected_rows.len());
    for row in expected_rows {
        let manifest_path = fixture_manifest_path(fake_result_root, row);
        let manifest_label = path_relative_to_repo(repo_root, &manifest_path);
        if !manifest_path.is_file() {
            rows.push(VcfMissingResultReportRow {
                result_row_id: result_row_id(row),
                domain: row.domain.clone(),
                stage_id: row.stage_id.clone(),
                tool_id: row.tool_id.clone(),
                corpus_id: row.corpus_id.clone(),
                asset_profile_id: row.asset_profile_id.clone(),
                expected_manifest_path: manifest_label.clone(),
                audit_manifest_path: manifest_label,
                result_status: VcfMissingResultStatus::MissingResult,
                expected_output_artifact_ids: row.expected_outputs.clone(),
                observed_output_artifact_ids: Vec::new(),
                expected_metrics: row.expected_metrics.clone(),
                report_section: row.report_section.clone(),
                reason: format!(
                    "expected VCF benchmark row `{}` remains visible even though its manifest is missing",
                    result_row_id(row)
                ),
            });
            continue;
        }

        let manifest = load_validated_stage_result_manifest_path(&manifest_path)
            .with_context(|| format!("load {}", manifest_path.display()))?;
        if manifest.stage_id != row.stage_id || manifest.tool.id != row.tool_id {
            return Err(anyhow!(
                "fake VCF benchmark result manifest `{}` drifted from `{}` / `{}`",
                manifest_path.display(),
                row.stage_id,
                row.tool_id
            ));
        }
        let observed_output_artifact_ids = manifest
            .outputs
            .iter()
            .map(|artifact| artifact.artifact_id.clone())
            .collect::<Vec<_>>();
        let observed_output_set =
            observed_output_artifact_ids.iter().cloned().collect::<std::collections::BTreeSet<_>>();
        let expected_output_set =
            row.expected_outputs.iter().cloned().collect::<std::collections::BTreeSet<_>>();
        if observed_output_set != expected_output_set {
            return Err(anyhow!(
                "fake VCF benchmark result manifest `{}` drifted from expected output ids for `{}`",
                manifest_path.display(),
                result_row_id(row)
            ));
        }

        rows.push(VcfMissingResultReportRow {
            result_row_id: result_row_id(row),
            domain: row.domain.clone(),
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            corpus_id: row.corpus_id.clone(),
            asset_profile_id: row.asset_profile_id.clone(),
            expected_manifest_path: manifest_label.clone(),
            audit_manifest_path: manifest_label,
            result_status: VcfMissingResultStatus::Present,
            expected_output_artifact_ids: row.expected_outputs.clone(),
            observed_output_artifact_ids,
            expected_metrics: row.expected_metrics.clone(),
            report_section: row.report_section.clone(),
            reason: format!(
                "expected VCF benchmark row `{}` remains present with a validated stage-result manifest",
                result_row_id(row)
            ),
        });
    }
    Ok(rows)
}
