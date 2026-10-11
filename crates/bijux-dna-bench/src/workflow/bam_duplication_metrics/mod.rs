//! Owner: bijux-dna-bench
//! Executable material for the governed BAM duplication-metrics campaign.

mod commands;
mod report;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    validate_attempt_identity, AdmittedBamCorpus, BenchmarkAcceptanceExpectedValue,
    BenchmarkAcceptanceMetric, BenchmarkAcceptanceOperator, BenchmarkAttemptIdentity,
    BenchmarkEffectiveParameterValue as EffectiveValue, BenchmarkExecutionAcceptance,
    BenchmarkExecutionDeterminism, BenchmarkLifecycleStatus,
};

use self::commands::{measured_command, negative_command, required_files, warmup_command};
use super::bam_observation::{ensure_parameter_evidence, execution_inputs, BamParameterPolicy};
use super::{load_attempt_lifecycle, BenchmarkExecutionMaterial};

pub(crate) use report::publish_bam_duplication_metrics_stage_report_with_presentation_authority;
pub use report::{
    publish_bam_duplication_metrics_stage_report, render_bam_duplication_metrics_markdown,
    BamDuplicationMetricsStageReportPublication,
};

const RUN_ID: &str = "run-2011";
const STAGE_ID: &str = "bam.duplication_metrics";
const CORPUS_ID: &str = "ena-prjeb22390-err2112767-bam";
const SAMPLE_ID: &str = "ERR2112767";

#[derive(Debug, Clone, PartialEq)]
pub struct BamDuplicationMetricsMaterial {
    pub execution: BenchmarkExecutionMaterial,
    pub admitted_corpus: AdmittedBamCorpus,
}

#[allow(clippy::too_many_arguments)]
pub fn prepare_bam_duplication_metrics_material(
    repository_root: &Path,
    study_root: &Path,
    data_index_path: &Path,
    benchmark_root: &Path,
    identity: &BenchmarkAttemptIdentity,
    source_contract_sha256: &BTreeMap<String, String>,
    operator: &str,
    prepared_at_utc: &str,
    account: &str,
    partition: &str,
) -> Result<BamDuplicationMetricsMaterial> {
    let corpus = crate::admit_bam_benchmark_corpus(study_root, data_index_path, CORPUS_ID)?;
    prepare_bam_duplication_metrics_material_from_corpus(
        repository_root,
        benchmark_root,
        identity,
        source_contract_sha256,
        operator,
        prepared_at_utc,
        account,
        partition,
        &corpus,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn prepare_bam_duplication_metrics_material_from_corpus(
    repository_root: &Path,
    benchmark_root: &Path,
    identity: &BenchmarkAttemptIdentity,
    source_contract_sha256: &BTreeMap<String, String>,
    operator: &str,
    prepared_at_utc: &str,
    account: &str,
    partition: &str,
    corpus: &AdmittedBamCorpus,
) -> Result<BamDuplicationMetricsMaterial> {
    validate_attempt_identity(identity).context("validate BAM duplication-metrics attempt")?;
    if identity.run.run_id != RUN_ID
        || identity.run.domain != "bam"
        || identity.run.stage_id != STAGE_ID
        || !matches!(identity.tool_id.as_str(), "picard" | "samtools")
    {
        bail!("attempt identity is outside the governed run-2011 closure");
    }
    if load_attempt_lifecycle(benchmark_root, identity)?.current_status()
        != Some(BenchmarkLifecycleStatus::Planned)
    {
        bail!("BAM duplication-metrics material requires a wholly planned attempt");
    }
    let expected = corpus
        .bam
        .size_bytes
        .checked_add(corpus.bam_index.size_bytes)
        .context("BAM corpus size overflow")?;
    if corpus.corpus_id != CORPUS_ID
        || corpus.sample_id != SAMPLE_ID
        || corpus.input_bytes != expected
        || corpus.bam.size_bytes == 0
        || corpus.bam_index.size_bytes == 0
    {
        bail!("admitted BAM corpus differs from the run-2011 closure");
    }
    let attempt_directory = crate::benchmark_attempt_directory(benchmark_root, identity)?;
    let parameters = ensure_parameter_evidence(
        repository_root,
        &attempt_directory,
        &corpus.sample_id,
        BamParameterPolicy {
            stage_id: STAGE_ID,
            schema_version: "bijux.bam.params.duplication_metrics.v1",
            defaults_provider:
                "bijux_dna_domain_bam::stage_specs::stage_spec(BamStage::DuplicationMetrics)",
        },
        BTreeMap::from([
            (
                "schema_version".to_string(),
                EffectiveValue::Text("bijux.bam.params.duplication_metrics.v1".to_string()),
            ),
            ("optical_duplicates".to_string(), EffectiveValue::Text("mark_only".to_string())),
            ("umi_policy".to_string(), EffectiveValue::Text("ignore".to_string())),
            ("duplicate_action".to_string(), EffectiveValue::Text("mark".to_string())),
        ]),
    )?;
    let execution = BenchmarkExecutionMaterial {
        operator: operator.to_string(),
        prepared_at_utc: prepared_at_utc.to_string(),
        repository_path: PathBuf::from(repository_root),
        benchmark_root: PathBuf::from(benchmark_root),
        source_contract_sha256: source_contract_sha256.clone(),
        inputs: execution_inputs(corpus),
        warmup_command: warmup_command(&identity.tool_id),
        measured_command: measured_command(
            &identity.tool_id,
            &corpus.sample_id,
            corpus.bam.size_bytes,
            corpus.bam_index.size_bytes,
            corpus.input_bytes,
        ),
        negative_command: negative_command(&identity.tool_id),
        parameters,
        required_files: required_files(&identity.tool_id, 3),
        account: account.to_string(),
        partition: partition.to_string(),
        memory_mb: 8192,
        walltime: "01:00:00".to_string(),
        scratch_mb: 16384,
        random_seed: None,
        determinism: BenchmarkExecutionDeterminism::Deterministic,
        acceptance: acceptance(),
    };
    Ok(BamDuplicationMetricsMaterial { execution, admitted_corpus: corpus.clone() })
}

fn acceptance() -> BenchmarkExecutionAcceptance {
    BenchmarkExecutionAcceptance {
        version_regex: String::new(),
        negative_exit_nonzero: true,
        metrics: [
            ("examined_reads", BenchmarkAcceptanceOperator::GreaterThan, "reads"),
            ("duplicate_reads", BenchmarkAcceptanceOperator::GreaterOrEqual, "reads"),
            ("duplication_fraction", BenchmarkAcceptanceOperator::GreaterOrEqual, "fraction"),
        ]
        .into_iter()
        .map(|(id, operator, unit)| BenchmarkAcceptanceMetric {
            id: id.to_string(),
            path: "repetitions/repetition-1.metrics.json".to_string(),
            json_pointer: format!("/{id}"),
            operator,
            expected: BenchmarkAcceptanceExpectedValue::Integer(0),
            unit: unit.to_string(),
        })
        .collect(),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use bijux_dna_bench_model::{AdmittedBamCorpus, AdmittedBamCorpusFile};

    use super::prepare_bam_duplication_metrics_material_from_corpus;

    #[test]
    fn material_closes_both_run_2011_tools() -> anyhow::Result<()> {
        let repository =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize()?;
        let temporary = bijux_dna_infra::temp_dir("bam-duplication-material")?;
        let benchmark_root = temporary.path().join("benchmarks");
        let (registry, _, _) = crate::load_checked_in_benchmark_planning_contracts()?;
        let plan = crate::plan_stage_campaign_attempts(
            &registry,
            &"a".repeat(40),
            &"b".repeat(64),
            "run-2011",
            "horse-duplication",
            "attempt-001",
            "2026-08-05T00:00:00Z",
        )?;
        crate::ensure_stage_campaign_attempts(&benchmark_root, &plan)?;
        let corpus = admitted_corpus(temporary.path());
        for attempt in &plan.cycle.attempts {
            let material = prepare_bam_duplication_metrics_material_from_corpus(
                &repository,
                &benchmark_root,
                &attempt.identity,
                &source_contracts(),
                "operator",
                "2026-08-05T00:01:00Z",
                "project",
                "compute",
                &corpus,
            )?;
            assert!(material.execution.measured_command[2].contains("duplication_metrics.awk"));
            assert_eq!(material.execution.required_files.len(), 18);
            assert_eq!(material.execution.acceptance.metrics.len(), 3);
        }
        Ok(())
    }

    fn source_contracts() -> BTreeMap<String, String> {
        BTreeMap::from([
            ("campaign_registry".to_string(), "b".repeat(64)),
            ("container_catalog".to_string(), "c".repeat(64)),
            ("image_inventory".to_string(), "d".repeat(64)),
            ("stage_interface_registry".to_string(), "e".repeat(64)),
        ])
    }

    fn admitted_corpus(root: &std::path::Path) -> AdmittedBamCorpus {
        let file = |name: &str, size_bytes| AdmittedBamCorpusFile {
            path: root.join(name).display().to_string(),
            sha256: "f".repeat(64),
            size_bytes,
        };
        AdmittedBamCorpus {
            corpus_id: "ena-prjeb22390-err2112767-bam".to_string(),
            sample_id: "ERR2112767".to_string(),
            data_index_path: root.join("data-index.json").display().to_string(),
            data_index_sha256: "a".repeat(64),
            catalog_path: root.join("catalog.json").display().to_string(),
            catalog_sha256: "b".repeat(64),
            bam: file("ERR2112767.sorted.bam", 23),
            bam_index: file("ERR2112767.sorted.bam.bai", 3),
            input_bytes: 26,
        }
    }
}
