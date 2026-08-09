//! Owner: bijux-dna-bench
//! Generic aggregation of admitted benchmark case observations.

use std::collections::{BTreeMap, BTreeSet};

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::BenchmarkScientificValue;

use super::{
    validate_benchmark_case_matrix, BenchmarkAdmittedCaseObservationV2, BenchmarkCaseAggregationV2,
    BenchmarkCaseDefinitionV1, BenchmarkCaseMatrixV2,
    BENCHMARK_ADMITTED_CASE_OBSERVATION_SCHEMA_V2, BENCHMARK_CASE_AGGREGATION_SCHEMA_V2,
};

/// Aggregate exact admitted repetition closure without stage or backend dispatch.
///
/// # Errors
/// Returns an error when observations do not close the case matrix exactly, source identities
/// drift, repetitions or metrics are incomplete, values are invalid, or normalized outputs differ.
pub fn aggregate_benchmark_case_observations(
    matrix: &BenchmarkCaseMatrixV2,
    observations: &[BenchmarkAdmittedCaseObservationV2],
) -> Result<Vec<BenchmarkCaseAggregationV2>> {
    validate_benchmark_case_matrix(matrix).context("validate benchmark case matrix")?;
    let mut indexed = BTreeMap::new();
    for observation in observations {
        if observation.schema_version != BENCHMARK_ADMITTED_CASE_OBSERVATION_SCHEMA_V2 {
            bail!("admitted benchmark case observation schema is unsupported");
        }
        require_digest(&observation.admission_sha256, "admission digest")?;
        require_digest(&observation.input_lock_sha256, "input lock digest")?;
        require_digest(&observation.parameter_lock_sha256, "parameter lock digest")?;
        if indexed.insert(observation.case_id.as_str(), observation).is_some() {
            bail!("duplicate admitted benchmark case observation");
        }
    }
    let declared = matrix.cases.iter().map(|case| case.case_id.as_str()).collect::<BTreeSet<_>>();
    if indexed.keys().copied().collect::<BTreeSet<_>>() != declared {
        bail!("admitted observations do not close the benchmark case matrix");
    }
    let aggregations = matrix
        .cases
        .iter()
        .map(|case| {
            aggregate_case(
                case,
                indexed
                    .get(case.case_id.as_str())
                    .copied()
                    .context("case observation is absent")?,
                &matrix.repository_commit,
            )
        })
        .collect::<Result<Vec<_>>>()?;
    validate_benchmark_case_aggregations(matrix, &aggregations)?;
    Ok(aggregations)
}

/// Validate persisted aggregations against their complete canonical case matrix.
///
/// # Errors
/// Returns an error when case closure, source identity, execution provenance, metric closure, or
/// normalized-result determinism differs from the matrix authority.
pub fn validate_benchmark_case_aggregations(
    matrix: &BenchmarkCaseMatrixV2,
    aggregations: &[BenchmarkCaseAggregationV2],
) -> Result<()> {
    validate_benchmark_case_matrix(matrix).context("validate benchmark case matrix")?;
    let cases =
        matrix.cases.iter().map(|case| (case.case_id.as_str(), case)).collect::<BTreeMap<_, _>>();
    let mut observed = BTreeSet::new();
    for aggregation in aggregations {
        if aggregation.schema_version != BENCHMARK_CASE_AGGREGATION_SCHEMA_V2
            || !observed.insert(aggregation.case_id.as_str())
        {
            bail!("benchmark case aggregation identity is unsupported or duplicated");
        }
        let case = cases
            .get(aggregation.case_id.as_str())
            .copied()
            .context("benchmark case aggregation is not declared by the matrix")?;
        validate_aggregation(case, aggregation, &matrix.repository_commit)?;
    }
    if observed != cases.keys().copied().collect::<BTreeSet<_>>() {
        bail!("benchmark case aggregations do not close the benchmark case matrix");
    }
    Ok(())
}

fn aggregate_case(
    case: &BenchmarkCaseDefinitionV1,
    observation: &BenchmarkAdmittedCaseObservationV2,
    repository_commit: &str,
) -> Result<BenchmarkCaseAggregationV2> {
    if observation.case_id != case.case_id || observation.repository_commit != repository_commit {
        bail!("admitted case identity differs from the case matrix");
    }
    validate_execution_evidence(case, &observation.execution)?;
    let expected_repetitions = case.measurement.measured_repetitions;
    if observation.repetitions.len() != expected_repetitions as usize {
        bail!("admitted case repetition closure is incomplete");
    }
    let expected_performance =
        case.measurement.performance_metrics.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let expected_science =
        case.science.scientific_metrics.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let mut performance = BTreeMap::<String, Vec<f64>>::new();
    let mut scientific = BTreeMap::<String, Vec<BenchmarkScientificValue>>::new();
    let mut result_digests = BTreeSet::new();
    for (offset, repetition) in observation.repetitions.iter().enumerate() {
        if repetition.repetition != offset as u32 + 1
            || repetition.allocated_cpus != case.measurement.allocated_cpus
        {
            bail!("admitted case repetition identity or CPU allocation is invalid");
        }
        if repetition.performance.keys().map(String::as_str).collect::<BTreeSet<_>>()
            != expected_performance
            || repetition.scientific.keys().map(String::as_str).collect::<BTreeSet<_>>()
                != expected_science
        {
            bail!("admitted case metric closure differs from the case matrix");
        }
        for (metric, value) in &repetition.performance {
            if !value.is_finite() || *value < 0.0 {
                bail!("admitted case performance observation is invalid");
            }
            performance.entry(metric.clone()).or_default().push(*value);
        }
        for (metric, value) in &repetition.scientific {
            validate_scientific_value(value)?;
            scientific.entry(metric.clone()).or_default().push(value.clone());
        }
        require_digest(&repetition.normalized_result_sha256, "normalized result digest")?;
        result_digests.insert(repetition.normalized_result_sha256.as_str());
    }
    if result_digests.len() != 1 {
        bail!("normalized benchmark results differ across measured repetitions");
    }
    let performance = performance
        .into_iter()
        .map(|(metric, values)| {
            super::super::stage_report::distribution(&values).map(|summary| (metric, summary))
        })
        .collect::<Result<BTreeMap<_, _>>>()?;
    Ok(BenchmarkCaseAggregationV2 {
        schema_version: BENCHMARK_CASE_AGGREGATION_SCHEMA_V2.to_string(),
        case_id: case.case_id.clone(),
        run_id: case.run_id.clone(),
        domain: case.domain.clone(),
        stage_id: case.stage_id.clone(),
        tool_id: case.tool_id.clone(),
        corpus: case.corpus.clone(),
        repository_commit: repository_commit.to_string(),
        admission_sha256: observation.admission_sha256.clone(),
        input_lock_sha256: observation.input_lock_sha256.clone(),
        parameter_lock_sha256: observation.parameter_lock_sha256.clone(),
        execution: observation.execution.clone(),
        repetition_count: expected_repetitions,
        performance,
        scientific,
        normalized_result_sha256: result_digests
            .into_iter()
            .next()
            .context("normalized result digest is absent")?
            .to_string(),
    })
}

fn validate_aggregation(
    case: &BenchmarkCaseDefinitionV1,
    aggregation: &BenchmarkCaseAggregationV2,
    repository_commit: &str,
) -> Result<()> {
    if aggregation.case_id != case.case_id
        || aggregation.run_id != case.run_id
        || aggregation.domain != case.domain
        || aggregation.stage_id != case.stage_id
        || aggregation.tool_id != case.tool_id
        || aggregation.corpus != case.corpus
        || aggregation.repository_commit != repository_commit
        || aggregation.repetition_count != case.measurement.measured_repetitions
    {
        bail!("benchmark case aggregation differs from its matrix case authority");
    }
    for (value, label) in [
        (&aggregation.admission_sha256, "admission digest"),
        (&aggregation.input_lock_sha256, "input lock digest"),
        (&aggregation.parameter_lock_sha256, "parameter lock digest"),
        (&aggregation.normalized_result_sha256, "normalized result digest"),
    ] {
        require_digest(value, label)?;
    }
    validate_execution_evidence(case, &aggregation.execution)?;
    let expected_performance =
        case.measurement.performance_metrics.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let expected_science =
        case.science.scientific_metrics.iter().map(String::as_str).collect::<BTreeSet<_>>();
    if aggregation.performance.keys().map(String::as_str).collect::<BTreeSet<_>>()
        != expected_performance
        || aggregation.scientific.keys().map(String::as_str).collect::<BTreeSet<_>>()
            != expected_science
    {
        bail!("benchmark case aggregation metric closure differs from its matrix case");
    }
    for distribution in aggregation.performance.values() {
        if !distribution.minimum.is_finite()
            || !distribution.maximum.is_finite()
            || !distribution.mean.is_finite()
            || !distribution.sample_standard_deviation.is_finite()
            || distribution.minimum < 0.0
            || distribution.minimum > distribution.mean
            || distribution.mean > distribution.maximum
            || distribution.sample_standard_deviation < 0.0
        {
            bail!("benchmark case aggregation performance distribution is invalid");
        }
    }
    for value in aggregation.scientific.values().flatten() {
        validate_scientific_value(value)?;
    }
    if aggregation.scientific.values().any(Vec::is_empty) {
        bail!("benchmark case aggregation scientific observation is empty");
    }
    Ok(())
}

fn validate_execution_evidence(
    case: &BenchmarkCaseDefinitionV1,
    execution: &super::BenchmarkCaseExecutionEvidenceV1,
) -> Result<()> {
    if execution.image.name.trim().is_empty() || execution.image.size_bytes == 0 {
        bail!("admitted case image evidence is invalid");
    }
    require_digest(&execution.image.sha256, "image digest")?;
    require_digest(&execution.source_evidence_sha256, "source evidence digest")?;
    if [
        execution.slurm.job_id.as_str(),
        execution.slurm.host.as_str(),
        execution.slurm.elapsed.as_str(),
        execution.slurm.allocated_cpus.as_str(),
    ]
    .iter()
    .any(|value| value.trim().is_empty())
    {
        bail!("admitted case scheduler evidence is invalid");
    }
    if execution.slurm.allocated_cpus.parse::<u32>().ok() != Some(case.measurement.allocated_cpus) {
        bail!("admitted case scheduler CPU evidence differs from the case allocation");
    }
    let expected_input_bytes =
        case.campaign.input_size_keys.iter().map(String::as_str).collect::<BTreeSet<_>>();
    if execution.input_bytes.keys().map(String::as_str).collect::<BTreeSet<_>>()
        != expected_input_bytes
        || execution.input_bytes.values().any(|value| *value == 0)
    {
        bail!("admitted case input-byte closure differs from the campaign");
    }
    let mut lock_ids = BTreeSet::new();
    for lock in &execution.input_locks {
        if lock.id.trim().is_empty()
            || lock.kind.trim().is_empty()
            || lock.mount_path.trim().is_empty()
            || !lock_ids.insert(lock.id.as_str())
        {
            bail!("admitted case input-lock evidence is invalid");
        }
        require_digest(&lock.sha256, "input lock digest")?;
    }
    if execution.input_locks.is_empty() {
        bail!("admitted case input-lock evidence is empty");
    }
    Ok(())
}

fn validate_scientific_value(value: &BenchmarkScientificValue) -> Result<()> {
    match value {
        BenchmarkScientificValue::Number(value) if value.is_finite() => Ok(()),
        BenchmarkScientificValue::Text(value) if !value.trim().is_empty() => Ok(()),
        _ => bail!("admitted case scientific observation is invalid"),
    }
}

fn require_digest(value: &str, label: &str) -> Result<()> {
    if value.len() != 64
        || !value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        bail!("{label} is malformed");
    }
    Ok(())
}
