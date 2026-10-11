//! Owner: bijux-dna-bench
//! Generic stage-report derivation from validated case aggregations.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    BenchmarkAdmittedCaseObservationV2, BenchmarkCampaignPlan, BenchmarkCaseExecutionEvidenceV1,
    BenchmarkCaseRepetitionObservationV1, BenchmarkReportDeterminism,
    BenchmarkReportDownstreamProbe, BenchmarkReportStageInterface, BenchmarkReportVerdict,
    BenchmarkScientificValue, BenchmarkStageCampaignSpec, BenchmarkStageNormalizationUnitV1,
    BenchmarkStageNormalizationV1, BenchmarkStageReport, BenchmarkToolReport,
    BENCHMARK_ADMITTED_CASE_OBSERVATION_SCHEMA_V2, BENCHMARK_STAGE_NORMALIZATION_ARTIFACT,
    BENCHMARK_STAGE_NORMALIZATION_SCHEMA_V1,
};

use super::{
    aggregate_benchmark_case_observations, validate_benchmark_case_aggregations,
    BenchmarkCaseAggregationV2, BenchmarkCaseDefinitionV1, BenchmarkCaseMatrixV2,
};
use crate::workflow::stage_report::{
    build_stage_report_for_commit, is_selected_plan_attempt, load_admitted_attempt_evidence,
    read_json, read_resource_observation, require_complete_plan, ComparableAttemptEvidence,
};

/// Project admitted stage-plugin evidence into generic case aggregations without stage dispatch.
///
/// Numeric scientific values are reduced to the per-execution-unit mean for each measured
/// repetition. Equal nonnumeric values are retained exactly; heterogeneous nonnumeric values are
/// retained as canonical JSON rather than silently discarded. Performance values come from the
/// attempt-level resource receipt and exact admitted input/output byte closure.
///
/// # Errors
/// Returns an error when the selected plan, case matrix, admission, normalized stage outputs,
/// scientific metric closure, resource evidence, or immutable files differ from one another.
pub(crate) fn aggregate_admitted_benchmark_case_evidence(
    repository_root: &Path,
    study_root: &Path,
    benchmark_root: &Path,
    plan: &BenchmarkCampaignPlan,
    matrix: &BenchmarkCaseMatrixV2,
    campaign: &BenchmarkStageCampaignSpec,
) -> Result<Vec<BenchmarkCaseAggregationV2>> {
    require_complete_plan(campaign, plan)?;
    let cases =
        matrix.cases.iter().filter(|case| case.run_id == campaign.run_id).collect::<Vec<_>>();
    if cases.is_empty() || cases.len() != campaign.tool_ids.len() {
        bail!("generic report case closure differs from the declared tools");
    }
    let observations = cases
        .into_iter()
        .map(|case| {
            admitted_case_observation(
                repository_root,
                study_root,
                benchmark_root,
                plan,
                campaign,
                case,
            )
        })
        .collect::<Result<Vec<_>>>()?;
    aggregate_benchmark_case_observations(matrix, &observations)
}

fn admitted_case_observation(
    repository_root: &Path,
    study_root: &Path,
    benchmark_root: &Path,
    plan: &BenchmarkCampaignPlan,
    campaign: &BenchmarkStageCampaignSpec,
    case: &BenchmarkCaseDefinitionV1,
) -> Result<BenchmarkAdmittedCaseObservationV2> {
    let selected = plan
        .cycle
        .attempts
        .iter()
        .filter(|attempt| is_selected_plan_attempt(attempt))
        .find(|attempt| {
            attempt.identity.case_id.as_deref() == Some(case.case_id.as_str())
                && attempt.identity.tool_id == case.tool_id
        })
        .with_context(|| format!("campaign plan omits selected case attempt {}", case.case_id))?;
    let admitted = load_admitted_attempt_evidence(
        repository_root,
        study_root,
        benchmark_root,
        campaign,
        plan,
        &selected.identity,
    )?;
    if admitted.interface.accepted_input_contracts != case.interface.accepted_input_contracts
        || admitted.interface.output_contract != case.interface.output_contract
        || admitted.interface.normalized_result_contract
            != case.interface.normalized_result_contract
    {
        bail!("admitted generic case interface differs from its matrix authority");
    }
    let result_root = Path::new(&admitted.spec.outputs.result_root);
    let normalization_path = result_root.join(BENCHMARK_STAGE_NORMALIZATION_ARTIFACT);
    let normalization: BenchmarkStageNormalizationV1 = read_json(&normalization_path)?;
    if normalization.schema_version != BENCHMARK_STAGE_NORMALIZATION_SCHEMA_V1
        || normalization.identity != selected.identity
        || !normalization.passed
        || normalization.units.is_empty()
        || normalization.units.iter().any(|unit| !unit.passed)
    {
        bail!("generic report stage normalization is not admitted");
    }

    let input_bytes = exact_input_bytes(&admitted.spec)?;
    let execution_input_bytes = case
        .campaign
        .input_size_keys
        .iter()
        .map(|key| (key.clone(), input_bytes))
        .collect::<BTreeMap<_, _>>();
    if execution_input_bytes.len() != 1 {
        bail!("generic report requires one unambiguous input-size authority");
    }
    let repetitions = (1..=case.measurement.measured_repetitions)
        .map(|repetition| {
            case_repetition_observation(
                result_root,
                case,
                &normalization.units,
                repetition,
                input_bytes,
            )
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(BenchmarkAdmittedCaseObservationV2 {
        schema_version: BENCHMARK_ADMITTED_CASE_OBSERVATION_SCHEMA_V2.to_string(),
        case_id: case.case_id.clone(),
        repository_commit: plan.repository_commit.clone(),
        admission_sha256: admitted.source_evidence_sha256.clone(),
        input_lock_sha256: admitted.input_lock_sha256,
        parameter_lock_sha256: admitted.parameter_lock_sha256,
        execution: BenchmarkCaseExecutionEvidenceV1 {
            image: admitted.image,
            slurm: admitted.slurm,
            source_evidence_sha256: admitted.source_evidence_sha256,
            input_locks: admitted.input_locks,
            input_bytes: execution_input_bytes,
        },
        repetitions,
    })
}

fn case_repetition_observation(
    result_root: &Path,
    case: &BenchmarkCaseDefinitionV1,
    units: &[BenchmarkStageNormalizationUnitV1],
    repetition: u32,
    input_bytes: u64,
) -> Result<BenchmarkCaseRepetitionObservationV1> {
    let prefix = format!("measured-repetition-{repetition}--");
    let selected = units
        .iter()
        .filter_map(|unit| unit.stage_instance_id.strip_prefix(&prefix).map(|id| (id, unit)))
        .collect::<Vec<_>>();
    if selected.is_empty() {
        bail!("generic report repetition has no normalized execution units");
    }
    let resource = read_resource_observation(
        &result_root.join(format!("repetitions/repetition-{repetition}.resource.tsv")),
    )?;
    let output_bytes = normalized_output_bytes(selected.iter().map(|(_, unit)| *unit))?;
    let cpu_seconds = resource.user_seconds + resource.system_seconds;
    let available_cpu_seconds = resource.wall_seconds * f64::from(case.measurement.allocated_cpus);
    let values = BTreeMap::from([
        ("wall_seconds", resource.wall_seconds),
        ("cpu_seconds", cpu_seconds),
        (
            "cpu_utilization_percent",
            if available_cpu_seconds > 0.0 {
                100.0 * cpu_seconds / available_cpu_seconds
            } else {
                0.0
            },
        ),
        ("peak_rss_kib", resource.peak_rss_kib),
        ("filesystem_input_operations", resource.filesystem_input_operations),
        ("filesystem_output_operations", resource.filesystem_output_operations),
        ("input_bytes", input_bytes as f64),
        ("output_bytes", output_bytes as f64),
        ("throughput_input_bytes_per_second", input_bytes as f64 / resource.wall_seconds),
    ]);
    let performance = case
        .measurement
        .performance_metrics
        .iter()
        .map(|metric| {
            values.get(metric.as_str()).copied().map(|value| (metric.clone(), value)).with_context(
                || format!("generic report has no performance projection for {metric}"),
            )
        })
        .collect::<Result<BTreeMap<_, _>>>()?;

    let unit_metrics = selected
        .iter()
        .map(|(execution_unit_id, unit)| {
            canonical_scientific_metrics(unit)
                .map(|metrics| ((*execution_unit_id).to_string(), metrics))
        })
        .collect::<Result<Vec<_>>>()?;
    let scientific = case
        .science
        .scientific_metrics
        .iter()
        .map(|metric| {
            let values = unit_metrics
                .iter()
                .map(|(_, metrics)| {
                    metrics.get(metric).cloned().with_context(|| {
                        format!("normalized generic case omits scientific metric {metric}")
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            Ok((metric.clone(), reduce_scientific_values(&values)?))
        })
        .collect::<Result<BTreeMap<_, _>>>()?;
    let normalized_result_sha256 = bijux_dna_infra::hash_bytes_sha256(
        &serde_json::to_vec(&unit_metrics).context("encode generic normalized result closure")?,
    );
    Ok(BenchmarkCaseRepetitionObservationV1 {
        repetition,
        allocated_cpus: case.measurement.allocated_cpus,
        performance,
        scientific,
        normalized_result_sha256,
    })
}

fn exact_input_bytes(spec: &bijux_dna_bench_model::BenchmarkExecutionSpec) -> Result<u64> {
    spec.inputs.iter().try_fold(0_u64, |total, input| {
        let metadata = fs::symlink_metadata(&input.path)
            .with_context(|| format!("inspect generic benchmark input {}", input.path))?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            bail!("generic benchmark input is not a regular file");
        }
        if bijux_dna_infra::hash_file_sha256(Path::new(&input.path))? != input.sha256 {
            bail!("generic benchmark input changed after execution");
        }
        total.checked_add(metadata.len()).context("generic benchmark input bytes overflow")
    })
}

fn normalized_output_bytes<'a>(
    units: impl Iterator<Item = &'a BenchmarkStageNormalizationUnitV1>,
) -> Result<u64> {
    let mut files = BTreeSet::new();
    for unit in units {
        let artifacts = unit
            .normalized_output
            .get("artifacts")
            .and_then(serde_json::Value::as_array)
            .context("normalized generic case omits artifacts")?;
        for artifact in artifacts {
            let path = artifact
                .get("path")
                .and_then(serde_json::Value::as_str)
                .context("normalized generic artifact omits its path")?;
            let metadata = fs::symlink_metadata(path)
                .with_context(|| format!("inspect normalized generic artifact {path}"))?;
            if metadata.file_type().is_symlink() || (!metadata.is_file() && !metadata.is_dir()) {
                bail!("normalized generic artifact is not a regular file or directory");
            }
            if metadata.is_file() {
                files.insert(path.to_string());
                continue;
            }
            for entry in walkdir::WalkDir::new(path).min_depth(1).follow_links(false) {
                let entry = entry.context("walk normalized generic artifact directory")?;
                if entry.file_type().is_symlink()
                    || (!entry.file_type().is_file() && !entry.file_type().is_dir())
                {
                    bail!("normalized generic directory contains an unsafe entry");
                }
                if entry.file_type().is_file() {
                    files.insert(entry.path().to_string_lossy().into_owned());
                }
            }
        }
    }
    files.into_iter().try_fold(0_u64, |total, path| {
        let metadata = fs::symlink_metadata(&path)
            .with_context(|| format!("inspect normalized generic artifact {path}"))?;
        total.checked_add(metadata.len()).context("generic benchmark output bytes overflow")
    })
}

fn canonical_scientific_metrics(
    unit: &BenchmarkStageNormalizationUnitV1,
) -> Result<BTreeMap<String, serde_json::Value>> {
    let parts = unit
        .normalized_output
        .get("report_parts")
        .and_then(serde_json::Value::as_array)
        .context("normalized generic case omits report parts")?;
    let metrics = parts
        .iter()
        .find(|part| {
            part.get("name").and_then(serde_json::Value::as_str)
                == Some("observed_semantic_metrics")
        })
        .and_then(|part| part.pointer("/payload/semantic_metrics"))
        .and_then(serde_json::Value::as_object)
        .context("normalized generic case omits canonical scientific metrics")?;
    Ok(metrics.iter().map(|(key, value)| (key.clone(), value.clone())).collect())
}

fn reduce_scientific_values(values: &[serde_json::Value]) -> Result<BenchmarkScientificValue> {
    if values.is_empty() {
        bail!("generic scientific metric has no execution-unit observations");
    }
    if values.iter().all(serde_json::Value::is_number) {
        let numeric = values
            .iter()
            .map(|value| value.as_f64().context("generic scientific number is not finite"))
            .collect::<Result<Vec<_>>>()?;
        if numeric.iter().any(|value| !value.is_finite()) {
            bail!("generic scientific number is not finite");
        }
        return Ok(BenchmarkScientificValue::Number(
            numeric.iter().sum::<f64>() / numeric.len() as f64,
        ));
    }
    if values.iter().all(|value| value == &values[0]) {
        return Ok(BenchmarkScientificValue::Text(match &values[0] {
            serde_json::Value::String(value) => value.clone(),
            value => serde_json::to_string(value)?,
        }));
    }
    Ok(BenchmarkScientificValue::Text(serde_json::to_string(values)?))
}

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
                scientific_strata: Vec::new(),
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

#[cfg(test)]
mod tests {
    use bijux_dna_bench_model::BenchmarkScientificValue;

    use super::reduce_scientific_values;

    #[test]
    fn generic_scientific_reduction_preserves_numeric_and_nonnumeric_evidence() -> anyhow::Result<()>
    {
        assert_eq!(
            reduce_scientific_values(&[serde_json::json!(2), serde_json::json!(4)])?,
            BenchmarkScientificValue::Number(3.0)
        );
        assert_eq!(
            reduce_scientific_values(&[
                serde_json::json!("report_only"),
                serde_json::json!("report_only"),
            ])?,
            BenchmarkScientificValue::Text("report_only".to_string())
        );
        assert_eq!(
            reduce_scientific_values(&[serde_json::Value::Null, serde_json::Value::Null])?,
            BenchmarkScientificValue::Text("null".to_string())
        );
        assert_eq!(
            reduce_scientific_values(&[serde_json::json!("a"), serde_json::json!("b")])?,
            BenchmarkScientificValue::Text("[\"a\",\"b\"]".to_string())
        );
        Ok(())
    }
}
