//! Owner: bijux-dna-bench
//! Generic aggregation of admitted benchmark case observations.

use std::collections::{BTreeMap, BTreeSet};

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::BenchmarkScientificValue;

use super::{
    validate_benchmark_case_matrix, BenchmarkAdmittedCaseObservationV1, BenchmarkCaseAggregationV1,
    BenchmarkCaseDefinitionV1, BenchmarkCaseMatrixV1,
    BENCHMARK_ADMITTED_CASE_OBSERVATION_SCHEMA_V1, BENCHMARK_CASE_AGGREGATION_SCHEMA_V1,
};

/// Aggregate exact admitted repetition closure without stage or backend dispatch.
///
/// # Errors
/// Returns an error when observations do not close the case matrix exactly, source identities
/// drift, repetitions or metrics are incomplete, values are invalid, or normalized outputs differ.
pub fn aggregate_benchmark_case_observations(
    matrix: &BenchmarkCaseMatrixV1,
    observations: &[BenchmarkAdmittedCaseObservationV1],
) -> Result<Vec<BenchmarkCaseAggregationV1>> {
    validate_benchmark_case_matrix(matrix).context("validate benchmark case matrix")?;
    let mut indexed = BTreeMap::new();
    for observation in observations {
        if observation.schema_version != BENCHMARK_ADMITTED_CASE_OBSERVATION_SCHEMA_V1 {
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
    matrix
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
        .collect()
}

fn aggregate_case(
    case: &BenchmarkCaseDefinitionV1,
    observation: &BenchmarkAdmittedCaseObservationV1,
    repository_commit: &str,
) -> Result<BenchmarkCaseAggregationV1> {
    if observation.case_id != case.case_id || observation.repository_commit != repository_commit {
        bail!("admitted case identity differs from the case matrix");
    }
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
    Ok(BenchmarkCaseAggregationV1 {
        schema_version: BENCHMARK_CASE_AGGREGATION_SCHEMA_V1.to_string(),
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
