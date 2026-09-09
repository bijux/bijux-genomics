//! Evaluate declared output contracts against explicit adapter observations without IO.

use crate::{
    AllDomainOutputDeclarationRow, AllDomainOutputDeclarationStatus, OutputContractProof,
    OutputContractRuntimePaths, OutputContractTestRow, OutputContractTestsReport,
    OUTPUT_CONTRACT_TESTS_SCHEMA_VERSION,
};
use std::collections::{BTreeMap, BTreeSet};

/// Summarize an evaluated output-audit inventory in deterministic order.
#[must_use]
pub fn build_output_contract_tests_report(
    output_path: String,
    mut rows: Vec<OutputContractTestRow>,
) -> OutputContractTestsReport {
    rows.sort_by(|left, right| {
        left.domain
            .cmp(&right.domain)
            .then_with(|| left.stage_id.cmp(&right.stage_id))
            .then_with(|| left.tool_id.cmp(&right.tool_id))
            .then_with(|| left.result_id.cmp(&right.result_id))
    });

    let passed_row_count = rows.iter().filter(|row| row.passed).count();
    let blocked_row_count = rows.iter().filter(|row| row.execution_blocker.is_some()).count();
    let failed_row_count =
        rows.iter().filter(|row| !row.passed && row.execution_blocker.is_none()).count();
    let mut domain_counts = BTreeMap::<String, usize>::new();
    let mut output_proof_surface_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *domain_counts.entry(row.domain.clone()).or_default() += 1;
        *output_proof_surface_counts.entry(row.output_proof_surface.clone()).or_default() += 1;
    }
    let blocked_stage_ids = rows
        .iter()
        .filter(|row| row.execution_blocker.is_some())
        .map(|row| row.stage_id.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let blocked_tool_ids = rows
        .iter()
        .filter(|row| row.execution_blocker.is_some())
        .map(|row| row.tool_id.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let failed_stage_ids = rows
        .iter()
        .filter(|row| !row.passed && row.execution_blocker.is_none())
        .map(|row| row.stage_id.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let failed_tool_ids = rows
        .iter()
        .filter(|row| !row.passed && row.execution_blocker.is_none())
        .map(|row| row.tool_id.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    OutputContractTestsReport {
        schema_version: OUTPUT_CONTRACT_TESTS_SCHEMA_VERSION.to_string(),
        output_path,
        row_count: rows.len(),
        passed_row_count,
        blocked_row_count,
        failed_row_count,
        domain_counts,
        output_proof_surface_counts,
        blocked_stage_ids,
        blocked_tool_ids,
        failed_stage_ids,
        failed_tool_ids,
        rows,
    }
}

/// Evaluate one declaration using the supplied observation and runtime path provenance.
#[must_use]
pub fn evaluate_output_contract_row(
    output_row: &AllDomainOutputDeclarationRow,
    runtime_path_proof: &OutputContractRuntimePaths,
    proof: OutputContractProof,
) -> OutputContractTestRow {
    let OutputContractProof {
        output_proof_surface,
        output_proof_path,
        proof_tool_id,
        observed_ids,
        observed_paths_by_id,
        stage_undeclared_output_ids,
        independent_execution_proven,
        execution_blocker,
    } = proof;
    let declared_raw = output_row.raw_outputs.clone();
    let declared_norm = output_row.normalized_metrics.clone();
    let declared_index = output_row.index_outputs.clone();
    let raw_outputs_matched = declared_raw.iter().all(|id| observed_ids.contains(id));
    let normalized_metrics_matched = declared_norm.iter().all(|id| observed_ids.contains(id));
    let index_outputs_matched = declared_index.iter().all(|id| observed_ids.contains(id));
    let logs_matched = string_set(&output_row.logs)
        == string_set(&[
            format!("stdout={}", runtime_path_proof.stdout_path),
            format!("stderr={}", runtime_path_proof.stderr_path),
        ]);
    let manifest_matched = output_row.manifest == runtime_path_proof.stage_result_path;
    let no_undeclared_outputs = stage_undeclared_output_ids.is_empty();

    let observed_raw_output_ids = filter_declared_ids(&observed_ids, &declared_raw);
    let observed_normalized_metric_ids = filter_declared_ids(&observed_ids, &declared_norm);
    let observed_index_output_ids = filter_declared_ids(&observed_ids, &declared_index);
    let observed_output_paths = declared_raw
        .iter()
        .chain(declared_norm.iter())
        .chain(declared_index.iter())
        .flat_map(|artifact_id| {
            observed_paths_by_id.get(artifact_id).cloned().unwrap_or_default().into_iter()
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let observed_logs = vec![
        format!("stdout={}", runtime_path_proof.stdout_path),
        format!("stderr={}", runtime_path_proof.stderr_path),
    ];
    let observed_manifest = runtime_path_proof.stage_result_path.clone();
    let passed = output_row.status == AllDomainOutputDeclarationStatus::Complete
        && execution_blocker.is_none()
        && proof_tool_id.as_deref() == Some(output_row.tool_id.as_str())
        && independent_execution_proven
        && raw_outputs_matched
        && normalized_metrics_matched
        && index_outputs_matched
        && logs_matched
        && manifest_matched
        && no_undeclared_outputs;

    let mut row = OutputContractTestRow {
        result_id: output_row.result_id.clone(),
        domain: output_row.domain.clone(),
        stage_id: output_row.stage_id.clone(),
        tool_id: output_row.tool_id.clone(),
        corpus_id: output_row.corpus_id.clone(),
        asset_profile_id: output_row.asset_profile_id.clone(),
        output_proof_surface,
        output_proof_path,
        proof_tool_id,
        runtime_path_proof_surface: runtime_path_proof.runtime_path_proof_surface.clone(),
        runtime_path_proof_path: runtime_path_proof.runtime_path_proof_path.clone(),
        declared_raw_output_ids: declared_raw,
        observed_raw_output_ids,
        declared_normalized_metric_ids: declared_norm,
        observed_normalized_metric_ids,
        declared_index_output_ids: declared_index,
        observed_index_output_ids,
        observed_output_paths,
        stage_undeclared_output_ids,
        declared_logs: output_row.logs.clone(),
        observed_logs,
        declared_manifest: output_row.manifest.clone(),
        observed_manifest,
        raw_outputs_matched,
        normalized_metrics_matched,
        index_outputs_matched,
        logs_matched,
        manifest_matched,
        no_undeclared_outputs,
        independent_execution_proven,
        execution_blocker,
        passed,
        reason: String::new(),
    };
    row.reason = output_audit_reason(output_row, &row);
    row
}

fn output_audit_reason(
    output_row: &AllDomainOutputDeclarationRow,
    row: &OutputContractTestRow,
) -> String {
    if row.passed {
        format!(
            "binding `{}` / `{}` keeps governed raw outputs, normalized metrics, index outputs, logs, and manifest paths under `{}` plus `{}`",
            output_row.stage_id,
            output_row.tool_id,
            row.output_proof_surface,
            row.runtime_path_proof_surface
        )
    } else if let Some(blocker) = &row.execution_blocker {
        format!("execution blocked: {blocker}")
    } else {
        let mut failures = Vec::<String>::new();
        if output_row.status != AllDomainOutputDeclarationStatus::Complete {
            failures.push("output declaration is not complete".to_string());
        }
        if row.proof_tool_id.as_deref() != Some(output_row.tool_id.as_str()) {
            failures.push(format!(
                "observed tool {:?} does not match declared tool `{}`",
                row.proof_tool_id, output_row.tool_id
            ));
        }
        if !row.independent_execution_proven {
            failures.push(format!(
                "independent execution proof is missing from `{}`",
                row.output_proof_surface
            ));
        }
        if !row.raw_outputs_matched {
            failures.push("declared raw outputs are not fully observed".to_string());
        }
        if !row.normalized_metrics_matched {
            failures.push("declared normalized metrics are not fully observed".to_string());
        }
        if !row.index_outputs_matched {
            failures.push("declared index outputs are not fully observed".to_string());
        }
        if !row.logs_matched {
            failures.push(
                "declared stdout/stderr paths drifted from the fake-run runtime paths".to_string(),
            );
        }
        if !row.manifest_matched {
            failures.push(
                "declared stage-result manifest path drifted from the fake-run runtime path"
                    .to_string(),
            );
        }
        if !row.no_undeclared_outputs {
            failures.push("observed outputs include undeclared artifact ids".to_string());
        }
        failures.join("; ")
    }
}

fn filter_declared_ids(observed_ids: &BTreeSet<String>, declared_ids: &[String]) -> Vec<String> {
    declared_ids
        .iter()
        .filter(|artifact_id| observed_ids.contains(*artifact_id))
        .cloned()
        .collect::<Vec<_>>()
}

fn string_set(values: &[String]) -> BTreeSet<String> {
    values.iter().cloned().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    fn inputs() -> (AllDomainOutputDeclarationRow, OutputContractRuntimePaths, OutputContractProof)
    {
        let declaration = AllDomainOutputDeclarationRow {
            result_id: "vcf:corpus:vcf.call:profile:bcftools".into(),
            domain: "vcf".into(),
            stage_id: "vcf.call".into(),
            tool_id: "bcftools".into(),
            corpus_id: "corpus".into(),
            asset_profile_id: "profile".into(),
            raw_outputs: vec!["variants".into()],
            normalized_metrics: vec!["metrics".into()],
            index_outputs: vec![],
            logs: vec!["stdout=run/stdout.log".into(), "stderr=run/stderr.log".into()],
            manifest: "run/stage-result.json".into(),
            status: AllDomainOutputDeclarationStatus::Complete,
        };
        let runtime = OutputContractRuntimePaths {
            runtime_path_proof_surface: "declared_vcf_result_paths".into(),
            runtime_path_proof_path: "runtime.json".into(),
            stdout_path: "run/stdout.log".into(),
            stderr_path: "run/stderr.log".into(),
            stage_result_path: "run/stage-result.json".into(),
        };
        let proof = OutputContractProof {
            output_proof_surface: "direct_tool_smoke".into(),
            output_proof_path: "observed.json".into(),
            proof_tool_id: Some("bcftools".into()),
            observed_ids: BTreeSet::from(["variants".into(), "metrics".into()]),
            observed_paths_by_id: BTreeMap::from([(
                "variants".into(),
                vec!["variants.vcf".into()],
            )]),
            stage_undeclared_output_ids: vec![],
            independent_execution_proven: true,
            execution_blocker: None,
        };
        (declaration, runtime, proof)
    }

    #[test]
    fn output_audit_requires_independent_complete_observations() {
        let (declaration, runtime, proof) = inputs();
        let admitted = evaluate_output_contract_row(&declaration, &runtime, proof.clone());
        assert!(admitted.passed);
        assert_eq!(admitted.observed_output_paths, vec!["variants.vcf"]);
        let mut missing = proof.clone();
        missing.observed_ids.remove("metrics");
        let failed = evaluate_output_contract_row(&declaration, &runtime, missing);
        assert!(!failed.passed);
        assert!(!failed.normalized_metrics_matched);
        let mut planned = proof;
        planned.independent_execution_proven = false;
        let unproven = evaluate_output_contract_row(&declaration, &runtime, planned);
        assert!(!unproven.passed);
        assert!(unproven.reason.contains("independent execution proof is missing"));
    }

    #[test]
    fn output_audit_preserves_blocked_failed_counts_and_wire_shape() -> Result<()> {
        let (declaration, runtime, proof) = inputs();
        let passed = evaluate_output_contract_row(&declaration, &runtime, proof.clone());
        let mut unavailable = proof.clone();
        unavailable.independent_execution_proven = false;
        unavailable.execution_blocker = Some("tool unavailable".into());
        let blocked = evaluate_output_contract_row(&declaration, &runtime, unavailable);
        let mut extra = proof;
        extra.stage_undeclared_output_ids.push("undeclared".into());
        let failed = evaluate_output_contract_row(&declaration, &runtime, extra);
        let report =
            build_output_contract_tests_report("report.json".into(), vec![failed, passed, blocked]);
        assert_eq!(
            (
                report.row_count,
                report.passed_row_count,
                report.blocked_row_count,
                report.failed_row_count
            ),
            (3, 1, 1, 1)
        );
        let mut wire = serde_json::to_value(&report)?;
        assert_eq!(wire["schema_version"], OUTPUT_CONTRACT_TESTS_SCHEMA_VERSION);
        assert_eq!(
            serde_json::to_value(serde_json::from_value::<OutputContractTestsReport>(
                wire.clone()
            )?)?,
            wire
        );
        wire["rows"][0]["undeclared_field"] = serde_json::json!(true);
        assert!(serde_json::from_value::<OutputContractTestsReport>(wire).is_err());
        Ok(())
    }
    #[test]
    fn output_audit_rejects_foreign_or_missing_tool_proof() {
        let (declaration, runtime, proof) = inputs();
        for tool in [None, Some("freebayes".to_string()), Some(" bcftools".to_string())] {
            let mut observed = proof.clone();
            observed.proof_tool_id = tool;
            let row = evaluate_output_contract_row(&declaration, &runtime, observed);
            assert!(!row.passed);
            assert!(row.reason.contains("does not match declared tool"));
        }
    }

    #[test]
    fn output_audit_blocker_prevents_positive_proof_and_count_overlap() {
        let (declaration, runtime, mut proof) = inputs();
        proof.execution_blocker = Some("execution unavailable".into());
        let row = evaluate_output_contract_row(&declaration, &runtime, proof);
        assert!(row.independent_execution_proven);
        assert!(!row.passed);
        let report = build_output_contract_tests_report("report.json".into(), vec![row]);
        assert_eq!(
            (
                report.row_count,
                report.passed_row_count,
                report.blocked_row_count,
                report.failed_row_count
            ),
            (1, 0, 1, 0)
        );
    }
}
