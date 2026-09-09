//! Observed FASTQ scoring evidence. Authored truth fixtures alone do not establish tool outcomes.

use crate::repo::load_optional_fastq_score_report as load_optional_json_value;
use anyhow::Result;
use bijux_dna_bench_model::FastqScoreEvidence as FastqEvidenceAggregate;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::Path;

mod adapters;
mod fields;
mod summaries;

use summaries::{
    merge_case_stage_report, merge_profile_reads_summary, merge_screen_taxonomy_summary,
    merge_validate_reads_summary,
};

/// Load observed report evidence without substituting fixture-reference availability for outputs.
/// Missing optional reports contribute no observations; supported bindings remain caller-owned.
///
/// # Errors
/// Rejects unreadable reports, malformed JSON, or report fields that cannot be interpreted.
pub fn load_fastq_tool_score_evidence(
    repo_root: &Path,
) -> Result<BTreeMap<(String, String), FastqEvidenceAggregate>> {
    let mut rows = BTreeMap::<(String, String), FastqEvidenceAggregate>::new();
    for (path, adapter) in adapters::REPORTS {
        merge_single_stage_report(repo_root, &mut rows, path, *adapter)?;
    }

    merge_case_stage_report(
        repo_root,
        &mut rows,
        "artifacts/runs/bench/local-smoke/fastq.trim_reads/report.json",
        "trim_reads",
    )?;
    merge_case_stage_report(
        repo_root,
        &mut rows,
        "artifacts/runs/bench/local-smoke/fastq.trim_polyg_tails/metrics.json",
        "trim_polyg_tails",
    )?;
    merge_case_stage_report(
        repo_root,
        &mut rows,
        "artifacts/runs/bench/local-smoke/fastq.normalize_primers/report.json",
        "normalize_primers",
    )?;
    merge_profile_reads_summary(repo_root, &mut rows)?;
    merge_validate_reads_summary(repo_root, &mut rows)?;
    merge_screen_taxonomy_summary(repo_root, &mut rows)?;

    Ok(rows)
}

fn merge_single_stage_report<F>(
    repo_root: &Path,
    rows: &mut BTreeMap<(String, String), FastqEvidenceAggregate>,
    relative_path: &str,
    extractor: F,
) -> Result<()>
where
    F: Fn(&Value) -> Result<(String, String, FastqEvidenceAggregate)>,
{
    let absolute_path = repo_root.join(relative_path);
    let Some(root) = load_optional_json_value(&absolute_path)? else {
        return Ok(());
    };
    let (stage_id, tool_id, evidence) = extractor(&root)?;
    merge_evidence_row(rows, (stage_id, tool_id), absolute_path.display().to_string(), evidence);
    Ok(())
}

fn merge_evidence_row(
    rows: &mut BTreeMap<(String, String), FastqEvidenceAggregate>,
    key: (String, String),
    source_path: String,
    mut evidence: FastqEvidenceAggregate,
) {
    evidence.source_paths.insert(source_path);
    let entry = rows.entry(key).or_default();
    entry.source_paths.extend(evidence.source_paths);
    if entry.truth_correctness_score.is_none() {
        entry.truth_correctness_score = evidence.truth_correctness_score;
    }
    if entry.truth_correctness_basis.is_none() {
        entry.truth_correctness_basis = evidence.truth_correctness_basis;
    }
    if entry.contract_correctness_score.is_none() {
        entry.contract_correctness_score = evidence.contract_correctness_score;
    }
    if entry.contract_correctness_basis.is_none() {
        entry.contract_correctness_basis = evidence.contract_correctness_basis;
    }
    if entry.retained_reads.is_none() {
        entry.retained_reads = evidence.retained_reads;
    }
    if entry.dropped_reads.is_none() {
        entry.dropped_reads = evidence.dropped_reads;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Context;

    #[test]
    fn empty_repository_never_manufactures_fastq_tool_observations() -> Result<()> {
        let repository = bijux_dna_infra::temp_dir("fastq-observed-evidence-empty")?;
        assert!(load_fastq_tool_score_evidence(repository.path())?.is_empty());
        Ok(())
    }

    #[test]
    fn observed_fastq_reports_preserve_counts_and_reject_corrupt_json() -> Result<()> {
        let repository = bijux_dna_infra::temp_dir("fastq-observed-evidence-report")?;
        let report = repository
            .path()
            .join("artifacts/runs/bench/local-smoke/fastq.filter_reads/report.json");
        bijux_dna_infra::ensure_dir(report.parent().context("report parent")?)?;
        bijux_dna_infra::write_bytes(&report, br#"{"stage_id":"fastq.filter_reads","tool_id":"fastp","input_reads":3,"output_reads":1,"reads_dropped":2}"#)?;
        let observed = load_fastq_tool_score_evidence(repository.path())?;
        assert_eq!(observed.len(), 1);
        let row = observed
            .get(&("fastq.filter_reads".into(), "fastp".into()))
            .context("observed fastp row")?;
        assert_eq!((row.retained_reads, row.dropped_reads), (Some(1), Some(2)));
        assert_eq!(row.contract_correctness_score, Some(1.0));
        assert!((row.truth_correctness_score.unwrap_or(-1.0) - 1.0 / 3.0).abs() < 1e-12);
        assert_eq!(row.source_paths.len(), 1);
        bijux_dna_infra::write_bytes(&report, b"{")?;
        assert!(load_fastq_tool_score_evidence(repository.path()).is_err());
        Ok(())
    }
    #[test]
    fn case_report_count_overflow_is_a_contextual_error() -> Result<()> {
        let repository = bijux_dna_infra::temp_dir("fastq-observed-evidence-overflow")?;
        let report =
            repository.path().join("artifacts/runs/bench/local-smoke/fastq.trim_reads/report.json");
        bijux_dna_infra::ensure_dir(report.parent().context("report parent")?)?;
        for field in ["input_reads", "reads_retained", "reads_dropped"] {
            let mut first = serde_json::json!({"tool_id":"fastp","input_reads":1,"output_reads":0,"reads_retained":0,"reads_dropped":0});
            first[field] = serde_json::json!(u64::MAX);
            let mut second = serde_json::json!({"tool_id":"fastp","input_reads":1,"output_reads":0,"reads_retained":0,"reads_dropped":0});
            second[field] = serde_json::json!(1);
            let value = serde_json::json!({"stage_id":"fastq.trim_reads","cases":[first,second]});
            bijux_dna_infra::write_bytes(&report, &serde_json::to_vec(&value)?)?;
            let error = load_fastq_tool_score_evidence(repository.path())
                .err()
                .context("count overflow must reject")?;
            assert!(error.to_string().contains("overflows"), "{error}");
        }
        Ok(())
    }
    #[test]
    fn aggregate_reports_reject_cross_stage_identity() -> Result<()> {
        for path in [
            "artifacts/runs/bench/local-smoke/fastq.trim_reads/report.json",
            "artifacts/runs/bench/local-smoke/fastq.trim_polyg_tails/metrics.json",
            "artifacts/runs/bench/local-smoke/fastq.normalize_primers/report.json",
            "artifacts/runs/bench/local-smoke/fastq.profile_reads/profile.json",
            "artifacts/runs/bench/local-smoke/fastq.validate_reads/report.json",
            "artifacts/benchmark-smoke/micro/pipelines/edna/artifacts/fastq.screen_taxonomy/report.json",
        ] {
            let repository = bijux_dna_infra::temp_dir("fastq-report-stage-identity")?;
            let report = repository.path().join(path);
            bijux_dna_infra::ensure_dir(report.parent().context("report parent")?)?;
            bijux_dna_infra::write_bytes(&report, br#"{"stage_id":"vcf.filter","tool_id":"fastp","cases":[],"samples":[]}"#)?;
            let error = load_fastq_tool_score_evidence(repository.path()).err().context("cross-stage report must reject")?;
            assert!(error.to_string().contains("expected stage"), "{error}");
        }
        Ok(())
    }
    #[test]
    fn summary_counts_reject_overflow_and_preserve_maximum_count() -> Result<()> {
        for (path, stage, tool, field) in [
            (
                "artifacts/runs/bench/local-smoke/fastq.profile_reads/profile.json",
                "fastq.profile_reads",
                "seqkit_stats",
                "reads_total",
            ),
            (
                "artifacts/runs/bench/local-smoke/fastq.validate_reads/report.json",
                "fastq.validate_reads",
                "fastqvalidator",
                "input_read_count_total",
            ),
        ] {
            let repository = bijux_dna_infra::temp_dir("fastq-summary-count-admission")?;
            let report = repository.path().join(path);
            bijux_dna_infra::ensure_dir(report.parent().context("report parent")?)?;
            let mut value = serde_json::json!({"stage_id":stage,"cases":[{field:u64::MAX,"validation_status":"pass"},{field:1,"validation_status":"pass"}]});
            bijux_dna_infra::write_bytes(&report, &serde_json::to_vec(&value)?)?;
            let error = load_fastq_tool_score_evidence(repository.path())
                .err()
                .context("sum overflow must reject")?;
            assert!(error.to_string().contains("overflows u64"), "{error}");
            value["cases"].as_array_mut().context("cases array")?.truncate(1);
            bijux_dna_infra::write_bytes(&report, &serde_json::to_vec(&value)?)?;
            let rows = load_fastq_tool_score_evidence(repository.path())?;
            assert_eq!(
                rows.get(&(stage.to_string(), tool.to_string()))
                    .context("observed summary")?
                    .retained_reads,
                Some(u64::MAX)
            );
        }
        Ok(())
    }
}
