//! Pure admission of the governed local smoke observation subset.

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    BenchError, RealSmokeCoreSubsetExecutionKind, RealSmokeCoreSubsetReport, RealSmokeCoreSubsetRow,
};

/// Admit declared subset counts, required stage/bridge coverage, metric presence and attached manifest outcomes.
///
/// # Errors
/// Returns an invalid-observation error when these governed subset checks fail.
/// This admission does not independently establish execution provenance or validate metric scientific meaning.
pub fn admit_real_smoke_core_subset_report(
    mut report: RealSmokeCoreSubsetReport,
) -> Result<RealSmokeCoreSubsetReport, BenchError> {
    validate_observation_summary(&report)?;
    if report.execution_count != 4 {
        return Err(invalid_observation(format!(
            "real-smoke core subset must keep exactly 4 executions, found {}",
            report.execution_count
        )));
    }
    if report.stage_execution_count != 3 {
        return Err(invalid_observation(format!(
            "real-smoke core subset must keep exactly 3 stage executions, found {}",
            report.stage_execution_count
        )));
    }
    if report.pipeline_bridge_count != 1 {
        return Err(invalid_observation(format!(
            "real-smoke core subset must keep exactly 1 pipeline bridge, found {}",
            report.pipeline_bridge_count
        )));
    }
    ensure_row_exists(
        &report.rows,
        RealSmokeCoreSubsetExecutionKind::Stage,
        "fastq",
        "fastq.validate_reads",
    )?;
    ensure_row_exists(
        &report.rows,
        RealSmokeCoreSubsetExecutionKind::Stage,
        "bam",
        "bam.validate",
    )?;
    ensure_row_exists(&report.rows, RealSmokeCoreSubsetExecutionKind::Stage, "vcf", "vcf.stats")?;
    let bridge_row = ensure_row_exists(
        &report.rows,
        RealSmokeCoreSubsetExecutionKind::PipelineBridge,
        "vcf",
        "vcf.call",
    )?;
    if bridge_row.bridge_source_domain.as_deref() != Some("bam")
        || bridge_row.bridge_target_domain.as_deref() != Some("vcf")
    {
        return Err(invalid_observation(format!(
            "real-smoke core subset bridge must stay bam->vcf, found {:?}->{:?}",
            bridge_row.bridge_source_domain, bridge_row.bridge_target_domain
        )));
    }

    for row in &report.rows {
        if row.normalized_metric_count == 0 || row.normalized_metrics.is_empty() {
            return Err(invalid_observation(format!(
                "real-smoke core subset row `{}` is missing normalized metrics",
                row.execution_id
            )));
        }
        if let Some(_manifest_path) = &row.stage_result_manifest_path {
            if row.manifest_status.as_deref() != Some("succeeded") {
                return Err(invalid_observation(format!(
                    "real-smoke core subset manifest for `{}` must be succeeded, found {:?}",
                    row.execution_id, row.manifest_status
                )));
            }
            if row.manifest_exit_code != Some(0) {
                return Err(invalid_observation(format!(
                    "real-smoke core subset manifest for `{}` must keep exit_code=0, found {:?}",
                    row.execution_id, row.manifest_exit_code
                )));
            }
        }
    }

    report.passes_behavior_test = true;
    Ok(report)
}

fn validate_observation_summary(report: &RealSmokeCoreSubsetReport) -> Result<(), BenchError> {
    if report.schema_version != crate::REAL_SMOKE_CORE_SUBSET_SCHEMA_VERSION {
        return Err(invalid_observation(
            "unsupported local smoke core subset schema version".to_string(),
        ));
    }
    if report.execution_count != report.rows.len() {
        return Err(invalid_observation(
            "real-smoke execution count does not match observed rows".to_string(),
        ));
    }
    let stage_count = report
        .rows
        .iter()
        .filter(|row| row.execution_kind == RealSmokeCoreSubsetExecutionKind::Stage)
        .count();
    let bridge_count = report
        .rows
        .iter()
        .filter(|row| row.execution_kind == RealSmokeCoreSubsetExecutionKind::PipelineBridge)
        .count();
    if report.stage_execution_count != stage_count || report.pipeline_bridge_count != bridge_count {
        return Err(invalid_observation(
            "real-smoke execution-kind counts do not match observed rows".to_string(),
        ));
    }
    let mut domain_counts = BTreeMap::new();
    let mut execution_ids = BTreeSet::new();
    for row in &report.rows {
        if !execution_ids.insert(row.execution_id.as_str()) {
            return Err(invalid_observation(format!(
                "duplicate real-smoke execution identity `{}`",
                row.execution_id
            )));
        }
        *domain_counts.entry(row.domain.clone()).or_insert(0_usize) += 1;
        if row.normalized_metric_count != row.normalized_metrics.len() {
            return Err(invalid_observation(format!(
                "real-smoke metric count for `{}` does not match observed metrics",
                row.execution_id
            )));
        }
    }
    if report.domain_counts != domain_counts {
        return Err(invalid_observation(
            "real-smoke domain counts do not match observed rows".to_string(),
        ));
    }
    Ok(())
}

fn ensure_row_exists<'a>(
    rows: &'a [RealSmokeCoreSubsetRow],
    execution_kind: RealSmokeCoreSubsetExecutionKind,
    domain: &str,
    stage_id: &str,
) -> Result<&'a RealSmokeCoreSubsetRow, BenchError> {
    rows.iter()
        .find(|row| {
            row.execution_kind == execution_kind && row.domain == domain && row.stage_id == stage_id
        })
        .ok_or_else(|| {
            invalid_observation(format!(
                "real-smoke core subset is missing `{}` `{}` `{}`",
                execution_kind_label(execution_kind),
                domain,
                stage_id
            ))
        })
}

fn execution_kind_label(kind: RealSmokeCoreSubsetExecutionKind) -> &'static str {
    match kind {
        RealSmokeCoreSubsetExecutionKind::Stage => "stage",
        RealSmokeCoreSubsetExecutionKind::PipelineBridge => "pipeline_bridge",
    }
}

fn invalid_observation(reason: String) -> BenchError {
    BenchError::InvalidObservation { reason }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn observations() -> RealSmokeCoreSubsetReport {
        let rows = [
            (RealSmokeCoreSubsetExecutionKind::Stage, "fastq", "fastq.validate_reads"),
            (RealSmokeCoreSubsetExecutionKind::Stage, "bam", "bam.validate"),
            (RealSmokeCoreSubsetExecutionKind::Stage, "vcf", "vcf.stats"),
            (RealSmokeCoreSubsetExecutionKind::PipelineBridge, "vcf", "vcf.call"),
        ]
        .into_iter()
        .map(|(kind, domain, stage)| RealSmokeCoreSubsetRow {
            execution_id: stage.to_string(),
            execution_kind: kind,
            domain: domain.to_string(),
            bridge_source_domain: (kind == RealSmokeCoreSubsetExecutionKind::PipelineBridge)
                .then(|| "bam".to_string()),
            bridge_target_domain: (kind == RealSmokeCoreSubsetExecutionKind::PipelineBridge)
                .then(|| "vcf".to_string()),
            stage_id: stage.to_string(),
            tool_id: "reported-producer".to_string(),
            corpus_id: "corpus".to_string(),
            asset_profile_id: "asset".to_string(),
            evidence_path: "artifacts/evidence.json".to_string(),
            parsed_schema_version: "report.v1".to_string(),
            stage_result_manifest_path: Some("artifacts/manifest.json".to_string()),
            manifest_status: Some("succeeded".to_string()),
            manifest_exit_code: Some(0),
            normalized_metric_count: 1,
            normalized_metrics: BTreeMap::from([("count".to_string(), serde_json::json!(3))]),
        })
        .collect();
        RealSmokeCoreSubsetReport {
            schema_version: crate::REAL_SMOKE_CORE_SUBSET_SCHEMA_VERSION.to_string(),
            output_path: "artifacts/smoke.json".to_string(),
            execution_count: 4,
            stage_execution_count: 3,
            pipeline_bridge_count: 1,
            domain_counts: BTreeMap::from([
                ("fastq".to_string(), 1),
                ("bam".to_string(), 1),
                ("vcf".to_string(), 2),
            ]),
            passes_behavior_test: false,
            rows,
        }
    }

    #[test]
    fn admitted_subset_keeps_original_rows_and_marks_the_behavior_check() -> Result<(), BenchError>
    {
        let original = observations();
        let mut expected = original.clone();
        expected.passes_behavior_test = true;
        assert_eq!(admit_real_smoke_core_subset_report(original)?, expected);
        Ok(())
    }

    #[test]
    fn subset_admission_rejects_missing_required_bindings_and_declared_counts() {
        for index in 0..4 {
            let mut report = observations();
            report.rows.remove(index);
            assert!(admit_real_smoke_core_subset_report(report).is_err());
        }
        let mut report = observations();
        report.execution_count = 3;
        assert!(admit_real_smoke_core_subset_report(report).is_err());
        let mut report = observations();
        report.stage_execution_count = 2;
        assert!(admit_real_smoke_core_subset_report(report).is_err());
        let mut report = observations();
        report.pipeline_bridge_count = 2;
        assert!(admit_real_smoke_core_subset_report(report).is_err());
    }

    #[test]
    fn subset_admission_preserves_bridge_metric_and_manifest_rejection() {
        let mut report = observations();
        report.rows[3].bridge_source_domain = Some("fastq".to_string());
        assert!(admit_real_smoke_core_subset_report(report).is_err());
        let mut report = observations();
        report.rows[0].normalized_metric_count = 0;
        assert!(admit_real_smoke_core_subset_report(report).is_err());
        let mut report = observations();
        report.rows[0].normalized_metrics.clear();
        assert!(admit_real_smoke_core_subset_report(report).is_err());
        let mut report = observations();
        report.rows[0].manifest_status = Some("failed".to_string());
        assert!(admit_real_smoke_core_subset_report(report).is_err());
        let mut report = observations();
        report.rows[0].manifest_exit_code = Some(1);
        assert!(admit_real_smoke_core_subset_report(report).is_err());
    }
    #[test]
    fn subset_admission_rejects_extra_rows_hidden_by_declared_counts() {
        let mut report = observations();
        report.rows.push(report.rows[0].clone());
        assert!(admit_real_smoke_core_subset_report(report).is_err());
    }

    #[test]
    fn subset_admission_rejects_duplicate_execution_identity() {
        let mut report = observations();
        report.rows[1].execution_id = report.rows[0].execution_id.clone();
        assert!(admit_real_smoke_core_subset_report(report).is_err());
    }

    #[test]
    fn subset_admission_requires_exact_domain_summaries() {
        let mut report = observations();
        report.domain_counts.insert("vcf".to_string(), 99);
        assert!(admit_real_smoke_core_subset_report(report).is_err());
    }

    #[test]
    fn subset_admission_requires_exact_metric_cardinality() {
        let mut report = observations();
        report.rows[0].normalized_metric_count = 2;
        assert!(admit_real_smoke_core_subset_report(report).is_err());
    }

    #[test]
    fn subset_admission_rejects_unsupported_schema_identity() {
        let mut report = observations();
        report.schema_version = "unknown.v1".to_string();
        assert!(admit_real_smoke_core_subset_report(report).is_err());
    }
}
