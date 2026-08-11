//! Owner: bijux-dna-bench
//! Admission projection for Picard cohort results with retained sample strata.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    BenchmarkCampaignPlan, BenchmarkExecutionSpec, BenchmarkMetricDistribution,
    BenchmarkReportDeterminism, BenchmarkReportVerdict, BenchmarkScientificStratum,
    BenchmarkScientificValue, BenchmarkStageCampaignSpec, BenchmarkStageInterfaceSpec,
    BenchmarkToolReport,
};
use serde_json::Value;

use super::ReportPolicy;
use crate::workflow::stage_report::{
    distribution, load_admitted_attempt_evidence, read_resource_observation,
    ComparableAttemptEvidence, ResourceObservation, MAX_REPORT_INPUT_BYTES,
};

struct CohortObservation {
    samples: BTreeMap<String, BTreeMap<String, BenchmarkScientificValue>>,
    input_bytes: u64,
    output_bytes: u64,
    result_sha256: String,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn load_attempt_evidence(
    repository_root: &Path,
    study_root: &Path,
    benchmark_root: &Path,
    campaign: &BenchmarkStageCampaignSpec,
    interface: &BenchmarkStageInterfaceSpec,
    plan: &BenchmarkCampaignPlan,
    identity: &bijux_dna_bench_model::BenchmarkAttemptIdentity,
    policy: &ReportPolicy,
) -> Result<ComparableAttemptEvidence> {
    let admitted = load_admitted_attempt_evidence(
        repository_root,
        study_root,
        benchmark_root,
        campaign,
        plan,
        identity,
    )?;
    let spec = &admitted.spec;
    let resources = (1..=campaign.measured_repetitions)
        .map(|repetition| {
            read_resource_observation(
                &Path::new(&spec.outputs.result_root)
                    .join(format!("repetitions/repetition-{repetition}.resource.tsv")),
            )
        })
        .collect::<Result<Vec<_>>>()?;
    let science = (1..=campaign.measured_repetitions)
        .map(|repetition| derive_observation(spec, repetition, policy))
        .collect::<Result<Vec<_>>>()?;
    let first = science.first().context("Picard cohort has no measured repetitions")?;
    if science.iter().any(|row| {
        row.input_bytes != first.input_bytes
            || row.samples.keys().collect::<Vec<_>>() != first.samples.keys().collect::<Vec<_>>()
    }) {
        bail!("Picard cohort repetitions disagree on inputs or sample strata");
    }
    let digests = science.iter().map(|row| row.result_sha256.as_str()).collect::<BTreeSet<_>>();
    if digests.len() != 1 {
        bail!("Picard cohort normalized results differ across repetitions");
    }
    let scientific_observations = aggregate_observations(campaign, &science)?;
    let scientific_strata = stratified_observations(campaign, &science)?;
    let performance = performance_distributions(&resources, &science, first.input_bytes)?;
    if performance.keys().collect::<BTreeSet<_>>()
        != campaign.performance_metrics.iter().collect::<BTreeSet<_>>()
    {
        bail!("Picard cohort performance closure differs from the campaign");
    }
    if admitted.interface.accepted_input_contracts != interface.accepted_input_contracts
        || admitted.interface.output_contract != interface.output_contract
        || admitted.interface.normalized_result_contract != interface.normalized_result_contract
    {
        bail!("attempt stage interface differs from repository authority");
    }
    let (bam_bytes, bam_index_bytes, reference_bytes) = input_byte_classes(spec)?;
    let mut input_bytes = BTreeMap::from([
        ("bam".to_string(), bam_bytes),
        ("bam_index".to_string(), bam_index_bytes),
        ("total".to_string(), first.input_bytes),
    ]);
    if reference_bytes > 0 {
        input_bytes.insert("reference".to_string(), reference_bytes);
    }
    Ok(ComparableAttemptEvidence {
        tool_report: BenchmarkToolReport {
            tool: identity.tool_id.clone(),
            verdict: BenchmarkReportVerdict::Passed,
            repository_commit: spec.source.repository_commit.clone(),
            image: admitted.image,
            slurm: admitted.slurm,
            source_evidence_sha256: admitted.source_evidence_sha256,
            input_locks: admitted.input_locks,
            input_bytes,
            performance,
            scientific_observations,
            scientific_strata,
            determinism: BenchmarkReportDeterminism {
                artifact: campaign.result_artifact.clone(),
                sha256: first.result_sha256.clone(),
                digest_basis: Some(
                    "sample-identified normalized cohort summary TSV bytes".to_string(),
                ),
                identical_across_repetitions: true,
            },
        },
        input_lock_sha256: admitted.input_lock_sha256,
        parameter_lock_sha256: admitted.parameter_lock_sha256,
        interface: admitted.interface,
    })
}

fn derive_observation(
    spec: &BenchmarkExecutionSpec,
    repetition: u32,
    policy: &ReportPolicy,
) -> Result<CohortObservation> {
    let root = Path::new(&spec.outputs.result_root);
    let prefix = format!("repetitions/repetition-{repetition}");
    let result_path = root.join(format!("{prefix}.{}.tsv", policy.result_stem));
    let samples = read_cohort_summary(&result_path, policy.scientific_metrics)?;
    if samples.len() != super::super::EXPECTED_SAMPLE_COUNT {
        bail!("Picard cohort result lacks the governed sample count");
    }
    let metrics: Value = read_json(&root.join(format!("{prefix}.metrics.json")))?;
    let probe: bijux_dna_bench_model::normalized_result::BenchmarkDownstreamProbeRecord =
        read_json(&root.join(format!("{prefix}.report-qc.json")))?;
    let (bam_bytes, bam_index_bytes, reference_bytes) = input_byte_classes(spec)?;
    let input_bytes = bam_bytes
        .checked_add(bam_index_bytes)
        .and_then(|value| value.checked_add(reference_bytes))
        .context("Picard cohort report input overflow")?;
    if metrics.get("sample_count").and_then(Value::as_u64)
        != Some(super::super::EXPECTED_SAMPLE_COUNT as u64)
        || metrics.get("input_bytes").and_then(Value::as_u64) != Some(input_bytes)
        || metrics.get("output_bytes").and_then(Value::as_u64).unwrap_or_default() == 0
        || !probe.complete
        || probe.stage_id != format!("benchmark.{}.consume", policy.stage_id)
    {
        bail!("Picard cohort aggregate metrics or downstream probe are invalid");
    }
    for metric in policy.scientific_metrics {
        let expected = samples
            .values()
            .map(|sample| scientific_number(&sample[*metric]))
            .sum::<Result<f64>>()?
            / samples.len() as f64;
        let observed = metrics
            .get(*metric)
            .and_then(Value::as_f64)
            .with_context(|| format!("Picard cohort aggregate metrics omit {metric}"))?;
        if (observed - expected).abs() > 1e-9_f64.max(expected.abs() * 1e-12) {
            bail!("Picard cohort aggregate metric differs from retained sample strata");
        }
    }
    validate_sample_artifacts(spec, repetition, policy, &samples, reference_bytes)?;
    Ok(CohortObservation {
        samples,
        input_bytes,
        output_bytes: metrics["output_bytes"].as_u64().context("output bytes")?,
        result_sha256: bijux_dna_infra::hash_file_sha256(&result_path)?,
    })
}

fn read_cohort_summary(
    path: &Path,
    expected_metrics: &[&str],
) -> Result<BTreeMap<String, BTreeMap<String, BenchmarkScientificValue>>> {
    let bytes = bijux_dna_infra::read_to_end_bounded(path, MAX_REPORT_INPUT_BYTES)?;
    let text = std::str::from_utf8(&bytes).context("Picard cohort summary is not UTF-8")?;
    let mut lines = text.lines();
    if lines.next() != Some("sample_id\tmetric\tvalue") {
        bail!("Picard cohort summary header is invalid");
    }
    let mut samples = BTreeMap::<String, BTreeMap<String, BenchmarkScientificValue>>::new();
    for line in lines {
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() != 3
            || fields[0].trim().is_empty()
            || !expected_metrics.contains(&fields[1])
        {
            bail!("Picard cohort summary row is outside the contract");
        }
        let value = fields[2].parse::<f64>().context("parse Picard cohort value")?;
        if !value.is_finite()
            || value < 0.0
            || samples
                .entry(fields[0].to_string())
                .or_default()
                .insert(fields[1].to_string(), BenchmarkScientificValue::Number(value))
                .is_some()
        {
            bail!("Picard cohort summary contains invalid or duplicate evidence");
        }
    }
    let expected = expected_metrics.iter().copied().collect::<BTreeSet<_>>();
    if samples.is_empty()
        || samples
            .values()
            .any(|values| values.keys().map(String::as_str).collect::<BTreeSet<_>>() != expected)
    {
        bail!("Picard cohort summary lacks exact sample metric closure");
    }
    Ok(samples)
}

fn aggregate_observations(
    campaign: &BenchmarkStageCampaignSpec,
    repetitions: &[CohortObservation],
) -> Result<BTreeMap<String, Vec<BenchmarkScientificValue>>> {
    campaign
        .scientific_metrics
        .iter()
        .map(|metric| {
            let values = repetitions
                .iter()
                .map(|repetition| {
                    let total = repetition
                        .samples
                        .values()
                        .map(|sample| scientific_number(&sample[metric]))
                        .sum::<Result<f64>>()?;
                    Ok(BenchmarkScientificValue::Number(total / repetition.samples.len() as f64))
                })
                .collect::<Result<Vec<_>>>()?;
            Ok((metric.clone(), values))
        })
        .collect()
}

fn stratified_observations(
    campaign: &BenchmarkStageCampaignSpec,
    repetitions: &[CohortObservation],
) -> Result<Vec<BenchmarkScientificStratum>> {
    let first = repetitions.first().context("Picard cohort repetitions are empty")?;
    first
        .samples
        .keys()
        .map(|sample_id| {
            let observations = campaign
                .scientific_metrics
                .iter()
                .map(|metric| {
                    let values = repetitions
                        .iter()
                        .map(|repetition| {
                            repetition.samples[sample_id].get(metric).cloned().with_context(|| {
                                format!("Picard stratum {sample_id} omits {metric}")
                            })
                        })
                        .collect::<Result<Vec<_>>>()?;
                    Ok((metric.clone(), values))
                })
                .collect::<Result<BTreeMap<_, _>>>()?;
            Ok(BenchmarkScientificStratum { sample_id: sample_id.clone(), observations })
        })
        .collect()
}

fn validate_sample_artifacts(
    spec: &BenchmarkExecutionSpec,
    repetition: u32,
    policy: &ReportPolicy,
    samples: &BTreeMap<String, BTreeMap<String, BenchmarkScientificValue>>,
    reference_bytes: u64,
) -> Result<()> {
    let root = Path::new(&spec.outputs.result_root);
    for (sample_id, expected) in samples {
        let stem = normalized_id(sample_id);
        let prefix = format!("repetitions/repetition-{repetition}.{stem}");
        let bundle: Value = read_json(&root.join(format!("{prefix}.{}.json", policy.result_stem)))?;
        let metrics: Value = read_json(&root.join(format!("{prefix}.metrics.json")))?;
        let probe: bijux_dna_bench_model::normalized_result::BenchmarkDownstreamProbeRecord =
            read_json(&root.join(format!("{prefix}.report-qc.json")))?;
        if bundle.get("sample_id").and_then(Value::as_str) != Some(sample_id)
            || bundle.get("stage_id").and_then(Value::as_str) != Some(policy.stage_id)
            || bundle.get("tool_id").and_then(Value::as_str) != Some("picard")
            || !probe.complete
        {
            bail!("Picard sample bundle or downstream probe identity is invalid");
        }
        for (metric, value) in expected {
            let expected = scientific_number(value)?;
            if bundle.get(metric).and_then(Value::as_f64) != Some(expected)
                || metrics.get(metric).and_then(Value::as_f64) != Some(expected)
            {
                bail!("Picard sample bundle differs from normalized cohort summary");
            }
        }
        let normalized = normalized_id(sample_id);
        let bam_bytes = input_size(spec, &format!("bam_{normalized}"))?;
        let bam_index_bytes = input_size(spec, &format!("bam_index_{normalized}"))?;
        let expected_input = bam_bytes
            .checked_add(bam_index_bytes)
            .and_then(|value| value.checked_add(reference_bytes))
            .context("Picard sample report input overflow")?;
        if metrics.get("bam_input_bytes").and_then(Value::as_u64) != Some(bam_bytes)
            || metrics.get("bam_index_input_bytes").and_then(Value::as_u64) != Some(bam_index_bytes)
            || metrics.get("input_bytes").and_then(Value::as_u64) != Some(expected_input)
            || metrics.get("output_bytes").and_then(Value::as_u64).unwrap_or_default() == 0
        {
            bail!("Picard sample byte metrics differ from execution inputs");
        }
    }
    Ok(())
}

fn input_byte_classes(spec: &BenchmarkExecutionSpec) -> Result<(u64, u64, u64)> {
    let mut bam = 0_u64;
    let mut bam_index = 0_u64;
    let mut reference = 0_u64;
    for input in &spec.inputs {
        let size = regular_file_size(Path::new(&input.path))?;
        if input.id.starts_with("bam_index_") {
            bam_index = bam_index.checked_add(size).context("BAM index size overflow")?;
        } else if input.id.starts_with("bam_") {
            bam = bam.checked_add(size).context("BAM size overflow")?;
        } else if matches!(
            input.id.as_str(),
            "reference_fasta" | "reference_fasta_index" | "reference_sequence_dictionary"
        ) {
            reference = reference.checked_add(size).context("reference size overflow")?;
        }
    }
    if bam == 0 || bam_index == 0 {
        bail!("Picard cohort execution lacks BAM or index bytes");
    }
    Ok((bam, bam_index, reference))
}

fn input_size(spec: &BenchmarkExecutionSpec, id: &str) -> Result<u64> {
    let input = spec
        .inputs
        .iter()
        .find(|input| input.id == id)
        .with_context(|| format!("Picard cohort execution omits input {id}"))?;
    regular_file_size(Path::new(&input.path))
}

fn performance_distributions(
    resources: &[ResourceObservation],
    science: &[CohortObservation],
    input_bytes: u64,
) -> Result<BTreeMap<String, BenchmarkMetricDistribution>> {
    if resources.len() != science.len() || resources.len() < 3 {
        bail!("Picard cohort report requires aligned measured repetitions");
    }
    let wall = resources.iter().map(|row| row.wall_seconds).collect::<Vec<_>>();
    let cpu = resources.iter().map(|row| row.user_seconds + row.system_seconds).collect::<Vec<_>>();
    let input = input_bytes as f64;
    Ok(BTreeMap::from([
        ("wall_seconds".to_string(), distribution(&wall)?),
        ("cpu_seconds".to_string(), distribution(&cpu)?),
        (
            "cpu_utilization_percent".to_string(),
            distribution(
                &cpu.iter().zip(&wall).map(|(cpu, wall)| cpu / wall * 100.0).collect::<Vec<_>>(),
            )?,
        ),
        (
            "peak_rss_kib".to_string(),
            distribution(&resources.iter().map(|row| row.peak_rss_kib).collect::<Vec<_>>())?,
        ),
        (
            "filesystem_input_operations".to_string(),
            distribution(
                &resources.iter().map(|row| row.filesystem_input_operations).collect::<Vec<_>>(),
            )?,
        ),
        (
            "filesystem_output_operations".to_string(),
            distribution(
                &resources.iter().map(|row| row.filesystem_output_operations).collect::<Vec<_>>(),
            )?,
        ),
        ("input_bytes".to_string(), distribution(&vec![input; resources.len()])?),
        (
            "output_bytes".to_string(),
            distribution(&science.iter().map(|row| row.output_bytes as f64).collect::<Vec<_>>())?,
        ),
        (
            "throughput_input_bytes_per_second".to_string(),
            distribution(&wall.iter().map(|wall| input / wall).collect::<Vec<_>>())?,
        ),
    ]))
}

fn scientific_number(value: &BenchmarkScientificValue) -> Result<f64> {
    match value {
        BenchmarkScientificValue::Number(value) => Ok(*value),
        BenchmarkScientificValue::Text(_) => bail!("Picard cohort metric is not numeric"),
    }
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let bytes = bijux_dna_infra::read_to_end_bounded(path, MAX_REPORT_INPUT_BYTES)?;
    serde_json::from_slice(&bytes).with_context(|| format!("parse {}", path.display()))
}

fn regular_file_size(path: &Path) -> Result<u64> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("inspect Picard cohort input {}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("Picard cohort input is not a regular non-symlink file");
    }
    Ok(metadata.len())
}

fn normalized_id(value: &str) -> String {
    value
        .to_ascii_lowercase()
        .chars()
        .map(|character| if character.is_ascii_alphanumeric() { character } else { '_' })
        .collect()
}
