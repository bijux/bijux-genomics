//! Owner: bijux-dna-bench
//! Admission projection for Picard cohort results with retained sample strata.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::normalized_result::bam::{
    BenchmarkBamGcBiasCohortMetrics, BenchmarkBamGcBiasMetrics, BenchmarkBamGcBiasResult,
    BenchmarkBamGcBiasValues, BenchmarkBamInsertSizeCohortMetrics, BenchmarkBamInsertSizeMetrics,
    BenchmarkBamInsertSizeResult, BenchmarkBamInsertSizeValues,
};
use bijux_dna_bench_model::{
    BenchmarkCampaignPlan, BenchmarkExecutionSpec, BenchmarkMetricDistribution,
    BenchmarkReportDeterminism, BenchmarkReportVerdict, BenchmarkScientificStratum,
    BenchmarkScientificValue, BenchmarkStageCampaignSpec, BenchmarkStageInterfaceSpec,
    BenchmarkToolReport,
};

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

enum PicardCohortMetrics {
    GcBias(BenchmarkBamGcBiasCohortMetrics),
    InsertSize(BenchmarkBamInsertSizeCohortMetrics),
}

impl PicardCohortMetrics {
    fn sample_count(&self) -> u64 {
        match self {
            Self::GcBias(value) => value.sample_count,
            Self::InsertSize(value) => value.sample_count,
        }
    }

    fn input_bytes(&self) -> u64 {
        match self {
            Self::GcBias(value) => value.input_bytes,
            Self::InsertSize(value) => value.input_bytes,
        }
    }

    fn output_bytes(&self) -> u64 {
        match self {
            Self::GcBias(value) => value.output_bytes,
            Self::InsertSize(value) => value.output_bytes,
        }
    }

    fn scientific_metric(&self, metric: &str) -> Result<f64> {
        match self {
            Self::GcBias(value) => gc_bias_cohort_metric(value, metric),
            Self::InsertSize(value) => insert_size_cohort_metric(value, metric),
        }
    }
}

enum PicardSampleResult {
    GcBias(BenchmarkBamGcBiasResult),
    InsertSize(BenchmarkBamInsertSizeResult),
}

impl PicardSampleResult {
    fn identity(&self) -> (&str, &str) {
        match self {
            Self::GcBias(value) => (&value.sample_id, &value.tool_id),
            Self::InsertSize(value) => (&value.sample_id, &value.tool_id),
        }
    }

    fn scientific_metric(&self, metric: &str) -> Result<f64> {
        match self {
            Self::GcBias(value) => gc_bias_metric(&value.values, metric),
            Self::InsertSize(value) => insert_size_metric(&value.values, metric),
        }
    }
}

enum PicardSampleMetrics {
    GcBias(BenchmarkBamGcBiasMetrics),
    InsertSize(BenchmarkBamInsertSizeMetrics),
}

impl PicardSampleMetrics {
    fn scientific_metric(&self, metric: &str) -> Result<f64> {
        match self {
            Self::GcBias(value) => gc_bias_metric(&value.values, metric),
            Self::InsertSize(value) => insert_size_metric(&value.values, metric),
        }
    }

    fn input_bytes(&self) -> (u64, u64, u64, u64, u64) {
        match self {
            Self::GcBias(value) => (
                value.bam_input_bytes,
                value.bam_index_input_bytes,
                value.reference_input_bytes,
                value.input_bytes,
                value.output_bytes,
            ),
            Self::InsertSize(value) => (
                value.bam_input_bytes,
                value.bam_index_input_bytes,
                0,
                value.input_bytes,
                value.output_bytes,
            ),
        }
    }
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
    let metrics = read_cohort_metrics(&root.join(format!("{prefix}.metrics.json")), policy)?;
    let probe: bijux_dna_bench_model::normalized_result::BenchmarkDownstreamProbeRecord =
        read_json(&root.join(format!("{prefix}.report-qc.json")))?;
    let (bam_bytes, bam_index_bytes, reference_bytes) = input_byte_classes(spec)?;
    let input_bytes = bam_bytes
        .checked_add(bam_index_bytes)
        .and_then(|value| value.checked_add(reference_bytes))
        .context("Picard cohort report input overflow")?;
    if metrics.sample_count() != super::super::EXPECTED_SAMPLE_COUNT as u64
        || metrics.input_bytes() != input_bytes
        || metrics.output_bytes() == 0
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
        let observed = metrics.scientific_metric(metric)?;
        if (observed - expected).abs() > 1e-9_f64.max(expected.abs() * 1e-12) {
            bail!("Picard cohort aggregate metric differs from retained sample strata");
        }
    }
    validate_sample_artifacts(spec, repetition, policy, &samples, reference_bytes)?;
    Ok(CohortObservation {
        samples,
        input_bytes,
        output_bytes: metrics.output_bytes(),
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
        let bundle = read_sample_result(
            &root.join(format!("{prefix}.{}.json", policy.result_stem)),
            policy,
        )?;
        let metrics = read_sample_metrics(&root.join(format!("{prefix}.metrics.json")), policy)?;
        let probe: bijux_dna_bench_model::normalized_result::BenchmarkDownstreamProbeRecord =
            read_json(&root.join(format!("{prefix}.report-qc.json")))?;
        let (bundle_sample_id, bundle_tool_id) = bundle.identity();
        if bundle_sample_id != sample_id || bundle_tool_id != "picard" || !probe.complete {
            bail!("Picard sample bundle or downstream probe identity is invalid");
        }
        for (metric, value) in expected {
            let expected = scientific_number(value)?;
            if bundle.scientific_metric(metric)? != expected
                || metrics.scientific_metric(metric)? != expected
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
        let (observed_bam, observed_index, observed_reference, observed_input, observed_output) =
            metrics.input_bytes();
        if observed_bam != bam_bytes
            || observed_index != bam_index_bytes
            || observed_reference != reference_bytes
            || observed_input != expected_input
            || observed_output == 0
        {
            bail!("Picard sample byte metrics differ from execution inputs");
        }
    }
    Ok(())
}

fn read_cohort_metrics(path: &Path, policy: &ReportPolicy) -> Result<PicardCohortMetrics> {
    match policy.stage_id {
        "bam.gc_bias" => read_json(path).map(PicardCohortMetrics::GcBias),
        "bam.insert_size" => read_json(path).map(PicardCohortMetrics::InsertSize),
        stage_id => bail!("unsupported Picard cohort metrics stage: {stage_id}"),
    }
}

fn read_sample_result(path: &Path, policy: &ReportPolicy) -> Result<PicardSampleResult> {
    match policy.stage_id {
        "bam.gc_bias" => read_json(path).map(PicardSampleResult::GcBias),
        "bam.insert_size" => read_json(path).map(PicardSampleResult::InsertSize),
        stage_id => bail!("unsupported Picard result stage: {stage_id}"),
    }
}

fn read_sample_metrics(path: &Path, policy: &ReportPolicy) -> Result<PicardSampleMetrics> {
    match policy.stage_id {
        "bam.gc_bias" => read_json(path).map(PicardSampleMetrics::GcBias),
        "bam.insert_size" => read_json(path).map(PicardSampleMetrics::InsertSize),
        stage_id => bail!("unsupported Picard measurement stage: {stage_id}"),
    }
}

fn gc_bias_metric(values: &BenchmarkBamGcBiasValues, metric: &str) -> Result<f64> {
    match metric {
        "gc_bias_score" => Ok(values.gc_bias_score),
        "at_dropout" => Ok(values.at_dropout),
        "gc_dropout" => Ok(values.gc_dropout),
        "total_clusters" => Ok(values.total_clusters as f64),
        "aligned_reads" => Ok(values.aligned_reads as f64),
        "windows" => Ok(values.windows as f64),
        "read_starts" => Ok(values.read_starts as f64),
        _ => bail!("GC-bias metric is outside the normalized result contract: {metric}"),
    }
}

fn gc_bias_cohort_metric(values: &BenchmarkBamGcBiasCohortMetrics, metric: &str) -> Result<f64> {
    match metric {
        "gc_bias_score" => Ok(values.gc_bias_score),
        "at_dropout" => Ok(values.at_dropout),
        "gc_dropout" => Ok(values.gc_dropout),
        "total_clusters" => Ok(values.total_clusters),
        "aligned_reads" => Ok(values.aligned_reads),
        "windows" => Ok(values.windows),
        "read_starts" => Ok(values.read_starts),
        _ => bail!("GC-bias metric is outside the cohort measurement contract: {metric}"),
    }
}

fn insert_size_metric(values: &BenchmarkBamInsertSizeValues, metric: &str) -> Result<f64> {
    match metric {
        "median_insert_size" => Ok(values.median_insert_size),
        "mean_insert_size" => Ok(values.mean_insert_size),
        "standard_deviation" => Ok(values.standard_deviation),
        "median_absolute_deviation" => Ok(values.median_absolute_deviation),
        "minimum_insert_size" => Ok(values.minimum_insert_size as f64),
        "maximum_insert_size" => Ok(values.maximum_insert_size as f64),
        "read_pairs" => Ok(values.read_pairs as f64),
        "pair_orientation_fr_fraction" => Ok(values.pair_orientation_fr_fraction),
        _ => bail!("insert-size metric is outside the normalized result contract: {metric}"),
    }
}

fn insert_size_cohort_metric(
    values: &BenchmarkBamInsertSizeCohortMetrics,
    metric: &str,
) -> Result<f64> {
    match metric {
        "median_insert_size" => Ok(values.median_insert_size),
        "mean_insert_size" => Ok(values.mean_insert_size),
        "standard_deviation" => Ok(values.standard_deviation),
        "median_absolute_deviation" => Ok(values.median_absolute_deviation),
        "minimum_insert_size" => Ok(values.minimum_insert_size),
        "maximum_insert_size" => Ok(values.maximum_insert_size),
        "read_pairs" => Ok(values.read_pairs),
        "pair_orientation_fr_fraction" => Ok(values.pair_orientation_fr_fraction),
        _ => bail!("insert-size metric is outside the cohort measurement contract: {metric}"),
    }
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
