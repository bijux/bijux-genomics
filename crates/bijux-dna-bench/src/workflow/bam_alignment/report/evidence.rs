//! Owner: bijux-dna-bench
//! Admission projection for alignment results with retained sample strata.

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

use crate::workflow::stage_report::{
    distribution, load_admitted_attempt_evidence, read_resource_observation,
    ComparableAttemptEvidence, ResourceObservation, MAX_REPORT_INPUT_BYTES,
};

const SCIENTIFIC_METRICS: [&str; 7] = [
    "total_reads",
    "mapped_reads",
    "alignment_rate",
    "expected_mapped_reads",
    "exact_placement_reads",
    "placement_precision",
    "placement_recall",
];

struct AlignmentObservation {
    samples: BTreeMap<String, BTreeMap<String, BenchmarkScientificValue>>,
    total_reads: u64,
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
        .map(|repetition| derive_observation(spec, repetition, identity))
        .collect::<Result<Vec<_>>>()?;
    let first = science.first().context("alignment campaign has no measured repetitions")?;
    if science.iter().any(|row| {
        row.input_bytes != first.input_bytes
            || row.total_reads != first.total_reads
            || row.samples.keys().collect::<Vec<_>>() != first.samples.keys().collect::<Vec<_>>()
    }) {
        bail!("alignment repetitions disagree on inputs, reads, or sample strata");
    }
    if science.iter().map(|row| row.result_sha256.as_str()).collect::<BTreeSet<_>>().len() != 1 {
        bail!("alignment normalized results differ across repetitions");
    }
    let performance = performance_distributions(&resources, &science, first)?;
    if performance.keys().collect::<BTreeSet<_>>()
        != campaign.performance_metrics.iter().collect::<BTreeSet<_>>()
    {
        bail!("alignment performance closure differs from the campaign");
    }
    if admitted.interface.accepted_input_contracts != interface.accepted_input_contracts
        || admitted.interface.output_contract != interface.output_contract
        || admitted.interface.normalized_result_contract != interface.normalized_result_contract
    {
        bail!("alignment attempt interface differs from repository authority");
    }
    let (fastq, reference, truth) = input_byte_classes(spec)?;
    Ok(ComparableAttemptEvidence {
        tool_report: BenchmarkToolReport {
            tool: identity.tool_id.clone(),
            verdict: BenchmarkReportVerdict::Passed,
            repository_commit: spec.source.repository_commit.clone(),
            image: admitted.image,
            slurm: admitted.slurm,
            source_evidence_sha256: admitted.source_evidence_sha256,
            input_locks: admitted.input_locks,
            input_bytes: BTreeMap::from([
                ("fastq".to_string(), fastq),
                ("reference".to_string(), reference),
                ("reference_index".to_string(), 0),
                ("truth".to_string(), truth),
                ("total".to_string(), first.input_bytes),
            ]),
            performance,
            scientific_observations: aggregate_observations(campaign, &science)?,
            scientific_strata: stratified_observations(campaign, &science)?,
            determinism: BenchmarkReportDeterminism {
                artifact: campaign.result_artifact.clone(),
                sha256: first.result_sha256.clone(),
                digest_basis: Some(
                    "sample-identified normalized alignment truth TSV bytes".to_string(),
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
    identity: &bijux_dna_bench_model::BenchmarkAttemptIdentity,
) -> Result<AlignmentObservation> {
    let root = Path::new(&spec.outputs.result_root);
    let prefix = format!("repetitions/repetition-{repetition}");
    let result_path = root.join(format!("{prefix}.alignment-truth-summary.tsv"));
    let samples = read_cohort_summary(&result_path)?;
    if samples.len() != super::super::EXPECTED_SAMPLE_COUNT as usize {
        bail!("alignment result lacks the governed sample count");
    }
    let metrics: Value = read_json(&root.join(format!("{prefix}.metrics.json")))?;
    let probe: bijux_dna_bench_model::normalized_result::BenchmarkDownstreamProbeRecord =
        read_json(&root.join(format!("{prefix}.report-qc.json")))?;
    let (fastq_bytes, reference_bytes, truth_bytes) = input_byte_classes(spec)?;
    let input_bytes = fastq_bytes
        .checked_add(reference_bytes)
        .and_then(|value| value.checked_add(truth_bytes))
        .context("alignment report input overflow")?;
    if metrics.get("sample_count").and_then(Value::as_u64)
        != Some(super::super::EXPECTED_SAMPLE_COUNT)
        || metrics.get("input_bytes").and_then(Value::as_u64) != Some(input_bytes)
        || metrics.get("output_bytes").and_then(Value::as_u64).unwrap_or_default() == 0
        || !probe.complete
        || probe.stage_id != "benchmark.bam.align.compare"
    {
        bail!("alignment aggregate metrics or downstream probe are invalid");
    }
    let total_reads = samples.values().try_fold(0_u64, |total, values| {
        total
            .checked_add(scientific_number(&values["total_reads"])? as u64)
            .context("alignment read count overflow")
    })?;
    validate_sample_artifacts(spec, root, &prefix, identity, &samples, reference_bytes)?;
    Ok(AlignmentObservation {
        samples,
        total_reads,
        input_bytes,
        output_bytes: metrics["output_bytes"].as_u64().context("output bytes")?,
        result_sha256: bijux_dna_infra::hash_file_sha256(&result_path)?,
    })
}

fn read_cohort_summary(
    path: &Path,
) -> Result<BTreeMap<String, BTreeMap<String, BenchmarkScientificValue>>> {
    let bytes = bijux_dna_infra::read_to_end_bounded(path, MAX_REPORT_INPUT_BYTES)?;
    let text = std::str::from_utf8(&bytes).context("alignment summary is not UTF-8")?;
    let mut lines = text.lines();
    if lines.next() != Some("sample_id\tmetric\tvalue") {
        bail!("alignment cohort summary header is invalid");
    }
    let mut samples = BTreeMap::<String, BTreeMap<String, BenchmarkScientificValue>>::new();
    for line in lines {
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() != 3 || fields[0].is_empty() || !SCIENTIFIC_METRICS.contains(&fields[1]) {
            bail!("alignment cohort summary row is outside the contract");
        }
        let value = fields[2].parse::<f64>().context("parse alignment metric")?;
        if !value.is_finite()
            || value < 0.0
            || samples
                .entry(fields[0].to_string())
                .or_default()
                .insert(fields[1].to_string(), BenchmarkScientificValue::Number(value))
                .is_some()
        {
            bail!("alignment cohort summary contains invalid or duplicate evidence");
        }
    }
    let expected = SCIENTIFIC_METRICS.into_iter().collect::<BTreeSet<_>>();
    if samples.is_empty()
        || samples
            .values()
            .any(|values| values.keys().map(String::as_str).collect::<BTreeSet<_>>() != expected)
    {
        bail!("alignment cohort summary lacks exact metric closure");
    }
    Ok(samples)
}

fn validate_sample_artifacts(
    spec: &BenchmarkExecutionSpec,
    root: &Path,
    repetition_prefix: &str,
    identity: &bijux_dna_bench_model::BenchmarkAttemptIdentity,
    samples: &BTreeMap<String, BTreeMap<String, BenchmarkScientificValue>>,
    reference_bytes: u64,
) -> Result<()> {
    for (sample_id, expected) in samples {
        let stem = super::super::inputs::normalized_id(sample_id);
        let prefix = format!("{repetition_prefix}.{stem}");
        let bundle: Value =
            read_json(&root.join(format!("{prefix}.alignment-truth-summary.json")))?;
        let metrics: Value = read_json(&root.join(format!("{prefix}.metrics.json")))?;
        let probe: bijux_dna_bench_model::normalized_result::BenchmarkDownstreamProbeRecord =
            read_json(&root.join(format!("{prefix}.report-qc.json")))?;
        if bundle.get("sample_id").and_then(Value::as_str) != Some(sample_id)
            || bundle.get("stage_id").and_then(Value::as_str) != Some("bam.align")
            || bundle.get("tool_id").and_then(Value::as_str) != Some(identity.tool_id.as_str())
            || !probe.complete
        {
            bail!("alignment sample bundle or probe identity is invalid");
        }
        for (metric, value) in expected {
            let expected = scientific_number(value)?;
            if bundle.get(metric).and_then(Value::as_f64) != Some(expected)
                || metrics.get(metric).and_then(Value::as_f64) != Some(expected)
            {
                bail!("alignment sample bundle differs from cohort summary");
            }
        }
        let fastq = input_prefix_size(spec, &format!("fastq_{stem}_"))?;
        let truth = input_size(spec, &format!("truth_{stem}"))?;
        let total = fastq
            .checked_add(reference_bytes)
            .and_then(|value| value.checked_add(truth))
            .context("alignment sample size overflow")?;
        if metrics.get("fastq_input_bytes").and_then(Value::as_u64) != Some(fastq)
            || metrics.get("reference_input_bytes").and_then(Value::as_u64) != Some(reference_bytes)
            || metrics.get("reference_index_input_bytes").and_then(Value::as_u64) != Some(0)
            || metrics.get("truth_input_bytes").and_then(Value::as_u64) != Some(truth)
            || metrics.get("input_bytes").and_then(Value::as_u64) != Some(total)
        {
            bail!("alignment sample byte metrics differ from execution inputs");
        }
    }
    Ok(())
}

fn aggregate_observations(
    campaign: &BenchmarkStageCampaignSpec,
    repetitions: &[AlignmentObservation],
) -> Result<BTreeMap<String, Vec<BenchmarkScientificValue>>> {
    campaign
        .scientific_metrics
        .iter()
        .map(|metric| {
            let values = repetitions
                .iter()
                .map(|repetition| {
                    let sum = repetition
                        .samples
                        .values()
                        .map(|sample| scientific_number(&sample[metric]))
                        .sum::<Result<f64>>()?;
                    Ok(BenchmarkScientificValue::Number(sum / repetition.samples.len() as f64))
                })
                .collect::<Result<Vec<_>>>()?;
            Ok((metric.clone(), values))
        })
        .collect()
}

fn stratified_observations(
    campaign: &BenchmarkStageCampaignSpec,
    repetitions: &[AlignmentObservation],
) -> Result<Vec<BenchmarkScientificStratum>> {
    let first = repetitions.first().context("alignment repetitions are empty")?;
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
                                format!("alignment stratum {sample_id} omits {metric}")
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

fn input_byte_classes(spec: &BenchmarkExecutionSpec) -> Result<(u64, u64, u64)> {
    let fastq = input_prefix_size(spec, "fastq_")?;
    let truth = input_prefix_size(spec, "truth_")?;
    let reference = ["reference_fasta", "reference_fasta_index", "reference_sequence_dictionary"]
        .into_iter()
        .map(|id| input_size(spec, id))
        .sum::<Result<u64>>()?;
    if fastq == 0 || truth == 0 || reference == 0 {
        bail!("alignment execution lacks non-empty input classes");
    }
    Ok((fastq, reference, truth))
}

fn input_prefix_size(spec: &BenchmarkExecutionSpec, prefix: &str) -> Result<u64> {
    spec.inputs.iter().filter(|input| input.id.starts_with(prefix)).try_fold(
        0_u64,
        |total, input| {
            total
                .checked_add(regular_file_size(Path::new(&input.path))?)
                .context("input size overflow")
        },
    )
}

fn input_size(spec: &BenchmarkExecutionSpec, id: &str) -> Result<u64> {
    let input = spec
        .inputs
        .iter()
        .find(|input| input.id == id)
        .with_context(|| format!("alignment execution omits input {id}"))?;
    regular_file_size(Path::new(&input.path))
}

fn performance_distributions(
    resources: &[ResourceObservation],
    science: &[AlignmentObservation],
    first: &AlignmentObservation,
) -> Result<BTreeMap<String, BenchmarkMetricDistribution>> {
    if resources.len() != science.len() || resources.len() < 3 {
        bail!("alignment report requires aligned measured repetitions");
    }
    let wall = resources.iter().map(|row| row.wall_seconds).collect::<Vec<_>>();
    let cpu = resources.iter().map(|row| row.user_seconds + row.system_seconds).collect::<Vec<_>>();
    let input = first.input_bytes as f64;
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
        (
            "throughput_reads_per_second".to_string(),
            distribution(
                &science
                    .iter()
                    .zip(&wall)
                    .map(|(row, wall)| row.total_reads as f64 / wall)
                    .collect::<Vec<_>>(),
            )?,
        ),
    ]))
}

fn scientific_number(value: &BenchmarkScientificValue) -> Result<f64> {
    match value {
        BenchmarkScientificValue::Number(value) => Ok(*value),
        BenchmarkScientificValue::Text(_) => bail!("alignment metric is not numeric"),
    }
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let bytes = bijux_dna_infra::read_to_end_bounded(path, MAX_REPORT_INPUT_BYTES)?;
    serde_json::from_slice(&bytes).with_context(|| format!("parse {}", path.display()))
}

fn regular_file_size(path: &Path) -> Result<u64> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("inspect alignment input {}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("alignment input is not a regular non-symlink file");
    }
    Ok(metadata.len())
}
