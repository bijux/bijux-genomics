//! Owner: bijux-dna-bench
//! Generic stage-report derivation from validated case aggregations.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    BenchmarkReportDeterminism, BenchmarkReportDownstreamProbe, BenchmarkReportStageInterface,
    BenchmarkReportVerdict, BenchmarkStageReport, BenchmarkToolReport,
};

use super::{
    validate_benchmark_case_aggregations, BenchmarkCaseAggregationV2, BenchmarkCaseMatrixV2,
};
use crate::workflow::stage_report::{build_stage_report_for_commit, ComparableAttemptEvidence};

/// Build one stage report from generic case aggregations without stage or tool dispatch.
///
/// The current report contract represents one corpus case per tool. Campaigns with multiple
/// corpus cases remain fail-closed until the report model can preserve per-corpus distributions.
///
/// # Errors
/// Returns an error when matrix/aggregation closure is invalid, the run is absent or mixed, a
/// tool has more than one corpus case, or report comparability and scientific tolerances fail.
pub fn build_benchmark_stage_report_from_case_aggregations(
    repository_root: &Path,
    matrix: &BenchmarkCaseMatrixV2,
    aggregations: &[BenchmarkCaseAggregationV2],
    run_id: &str,
) -> Result<BenchmarkStageReport> {
    let cases = matrix.cases.iter().filter(|case| case.run_id == run_id).collect::<Vec<_>>();
    let first = cases.first().copied().context("benchmark run has no case definitions")?;
    if cases.iter().any(|case| {
        case.campaign != first.campaign
            || case.interface != first.interface
            || case.stage_id != first.stage_id
    }) {
        bail!("benchmark run cases mix campaign or interface authorities");
    }
    let case_ids = cases.iter().map(|case| case.case_id.as_str()).collect::<BTreeSet<_>>();
    let selected = aggregations
        .iter()
        .filter(|aggregation| case_ids.contains(aggregation.case_id.as_str()))
        .collect::<Vec<_>>();
    if selected.len() != cases.len() {
        bail!("benchmark run aggregation closure is incomplete");
    }
    let run_data_ids = first
        .campaign
        .corpus_catalog_ids
        .iter()
        .chain(&first.campaign.reference_catalog_ids)
        .chain(&first.campaign.database_catalog_ids)
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let run_matrix = BenchmarkCaseMatrixV2 {
        schema_version: matrix.schema_version.clone(),
        repository_commit: matrix.repository_commit.clone(),
        campaign_registry_sha256: matrix.campaign_registry_sha256.clone(),
        interface_catalog_sha256: matrix.interface_catalog_sha256.clone(),
        tool_binding_registry_sha256: matrix.tool_binding_registry_sha256.clone(),
        data_snapshots: matrix
            .data_snapshots
            .iter()
            .filter(|snapshot| run_data_ids.contains(snapshot.catalog_id.as_str()))
            .cloned()
            .collect(),
        cases: cases.iter().map(|case| (*case).clone()).collect(),
    };
    let selected = selected.into_iter().cloned().collect::<Vec<_>>();
    validate_benchmark_case_aggregations(&run_matrix, &selected)?;

    let interface = report_interface(first);
    let mut evidence_by_tool = BTreeMap::new();
    for aggregation in &selected {
        let input_bytes = report_input_bytes(&aggregation.execution.input_bytes)?;
        let evidence = ComparableAttemptEvidence {
            tool_report: BenchmarkToolReport {
                tool: aggregation.tool_id.clone(),
                verdict: BenchmarkReportVerdict::Passed,
                repository_commit: aggregation.repository_commit.clone(),
                image: aggregation.execution.image.clone(),
                slurm: aggregation.execution.slurm.clone(),
                source_evidence_sha256: aggregation.execution.source_evidence_sha256.clone(),
                input_locks: aggregation.execution.input_locks.clone(),
                input_bytes,
                performance: aggregation.performance.clone(),
                scientific_observations: aggregation.scientific.clone(),
                determinism: BenchmarkReportDeterminism {
                    artifact: first.campaign.result_artifact.clone(),
                    sha256: aggregation.normalized_result_sha256.clone(),
                    digest_basis: Some(
                        "ResultNormalizer canonical output bytes recorded by the admitted case"
                            .to_string(),
                    ),
                    identical_across_repetitions: true,
                },
            },
            input_lock_sha256: aggregation.input_lock_sha256.clone(),
            parameter_lock_sha256: aggregation.parameter_lock_sha256.clone(),
            interface: interface.clone(),
        };
        if evidence_by_tool.insert(aggregation.tool_id.clone(), evidence).is_some() {
            bail!("benchmark stage report cannot collapse multiple corpus cases for one tool");
        }
    }
    build_stage_report_for_commit(
        repository_root,
        &matrix.repository_commit,
        &first.campaign,
        &first.interface,
        evidence_by_tool,
    )
}

fn report_input_bytes(input_bytes: &BTreeMap<String, u64>) -> Result<BTreeMap<String, u64>> {
    let mut report = input_bytes.clone();
    if !report.contains_key("total") {
        let total = input_bytes.values().try_fold(0_u64, |total, value| {
            total.checked_add(*value).context("benchmark input-byte total overflows u64")
        })?;
        report.insert("total".to_string(), total);
    }
    Ok(report)
}

fn report_interface(
    case: &bijux_dna_bench_model::BenchmarkCaseDefinitionV1,
) -> BenchmarkReportStageInterface {
    BenchmarkReportStageInterface {
        accepted_input_contracts: case.interface.accepted_input_contracts.clone(),
        output_contract: case.interface.output_contract.clone(),
        normalized_result_contract: case.interface.normalized_result_contract.clone(),
        downstream_probe: BenchmarkReportDownstreamProbe {
            stage_id: case.interface.downstream_probe.consumer_stage_id.clone(),
            input_contract: case.interface.downstream_probe.input_contract.clone(),
            source: case.interface.downstream_probe.source.clone(),
            artifact: case.campaign.result_artifact.clone(),
        },
    }
}
