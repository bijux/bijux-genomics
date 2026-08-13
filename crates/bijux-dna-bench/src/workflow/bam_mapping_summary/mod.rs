//! Owner: bijux-dna-bench
//! Executable material for the governed BAM mapping-summary campaign.

mod commands;
mod inputs;
mod parameters;
mod report;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    validate_attempt_identity, AdmittedBamCorpus, BenchmarkAcceptanceExpectedValue,
    BenchmarkAcceptanceMetric, BenchmarkAcceptanceOperator, BenchmarkAttemptIdentity,
    BenchmarkExecutionAcceptance, BenchmarkExecutionDeterminism, BenchmarkLifecycleStatus,
};

use self::commands::{measured_command, negative_command, required_files, warmup_command};
use self::inputs::execution_inputs;
use self::parameters::ensure_parameter_evidence;
use super::{load_attempt_lifecycle, BenchmarkExecutionMaterial};

pub(crate) use report::publish_bam_mapping_summary_stage_report_with_presentation_authority;
pub use report::{
    publish_bam_mapping_summary_stage_report, render_bam_mapping_summary_markdown,
    BamMappingSummaryStageReportPublication,
};

const RUN_ID: &str = "run-2004";
const STAGE_ID: &str = "bam.mapping_summary";
const CORPUS_ID: &str = "ena-prjeb22390-err2112767-bam";
const SAMPLE_ID: &str = "ERR2112767";

#[derive(Debug, Clone, PartialEq)]
pub struct BamMappingSummaryMaterial {
    pub execution: BenchmarkExecutionMaterial,
    pub admitted_corpus: AdmittedBamCorpus,
}

#[allow(clippy::too_many_arguments)]
/// Prepare complete executable material for one planned BAM mapping-summary attempt.
///
/// # Errors
/// Returns an error when attempt state, source locks, corpus authorities, parameters, commands, or
/// acceptance closure differs from the governed campaign.
pub fn prepare_bam_mapping_summary_material(
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
) -> Result<BamMappingSummaryMaterial> {
    let corpus = crate::admit_bam_benchmark_corpus(study_root, data_index_path, CORPUS_ID)?;
    prepare_bam_mapping_summary_material_from_corpus(
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
/// Prepare one attempt from a campaign-wide BAM corpus admitted by the same invocation.
///
/// # Errors
/// Returns an error when corpus or attempt authorities disagree with run-2004.
pub fn prepare_bam_mapping_summary_material_from_corpus(
    repository_root: &Path,
    benchmark_root: &Path,
    identity: &BenchmarkAttemptIdentity,
    source_contract_sha256: &BTreeMap<String, String>,
    operator: &str,
    prepared_at_utc: &str,
    account: &str,
    partition: &str,
    corpus: &AdmittedBamCorpus,
) -> Result<BamMappingSummaryMaterial> {
    validate_attempt_identity(identity).context("validate BAM mapping-summary attempt identity")?;
    if identity.run.run_id != RUN_ID
        || identity.run.domain != "bam"
        || identity.run.stage_id != STAGE_ID
        || !matches!(identity.tool_id.as_str(), "picard" | "samtools")
    {
        bail!("attempt identity is outside the governed run-2004 closure");
    }
    let lifecycle = load_attempt_lifecycle(benchmark_root, identity)?;
    if lifecycle.current_status() != Some(BenchmarkLifecycleStatus::Planned) {
        bail!("BAM mapping-summary execution material requires a wholly planned attempt");
    }
    let expected_input_bytes = corpus
        .bam
        .size_bytes
        .checked_add(corpus.bam_index.size_bytes)
        .context("admitted BAM corpus size overflow")?;
    if corpus.corpus_id != CORPUS_ID
        || corpus.sample_id != SAMPLE_ID
        || corpus.bam.size_bytes == 0
        || corpus.bam_index.size_bytes == 0
        || corpus.input_bytes != expected_input_bytes
    {
        bail!("admitted BAM corpus differs from the run-2004 closure");
    }
    let attempt_directory = crate::benchmark_attempt_directory(benchmark_root, identity)?;
    let parameters =
        ensure_parameter_evidence(repository_root, &attempt_directory, &corpus.sample_id)?;
    let measured_repetitions = 3;
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
        required_files: required_files(&identity.tool_id, measured_repetitions),
        account: account.to_string(),
        partition: partition.to_string(),
        memory_mb: 8192,
        walltime: "00:30:00".to_string(),
        scratch_mb: 4096,
        random_seed: None,
        determinism: BenchmarkExecutionDeterminism::Deterministic,
        acceptance: acceptance(),
        stage_plan_units: None,
    };
    Ok(BamMappingSummaryMaterial { execution, admitted_corpus: corpus.clone() })
}

fn acceptance() -> BenchmarkExecutionAcceptance {
    BenchmarkExecutionAcceptance {
        version_regex: String::new(),
        negative_exit_nonzero: true,
        metrics: [
            ("primary_reads", "reads"),
            ("mapped_primary_reads", "reads"),
            ("mapped_primary_fraction", "fraction"),
            ("paired_primary_reads", "reads"),
        ]
        .into_iter()
        .map(|(id, unit)| BenchmarkAcceptanceMetric {
            id: id.to_string(),
            path: "repetitions/repetition-1.metrics.json".to_string(),
            json_pointer: format!("/{id}"),
            operator: BenchmarkAcceptanceOperator::GreaterThan,
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

    use super::prepare_bam_mapping_summary_material_from_corpus;

    #[test]
    fn material_closes_both_run_2004_tools() -> anyhow::Result<()> {
        let repository =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize()?;
        let temporary = bijux_dna_infra::temp_dir("bam-mapping-summary-material")?;
        let benchmark_root = temporary.path().join("benchmarks");
        let (registry, _, _) = crate::load_checked_in_benchmark_planning_contracts()?;
        let plan = crate::plan_stage_campaign_attempts(
            &registry,
            &"a".repeat(40),
            &"b".repeat(64),
            "run-2004",
            "horse-mapping-summary",
            "attempt-001",
            "2026-08-05T00:00:00Z",
        )?;
        crate::ensure_stage_campaign_attempts(&benchmark_root, &plan)?;
        let corpus = admitted_corpus(temporary.path());
        let source = BTreeMap::from([
            ("campaign_registry".to_string(), "b".repeat(64)),
            ("container_catalog".to_string(), "c".repeat(64)),
            ("image_inventory".to_string(), "d".repeat(64)),
            ("stage_interface_registry".to_string(), "e".repeat(64)),
        ]);
        for attempt in &plan.cycle.attempts {
            let material = prepare_bam_mapping_summary_material_from_corpus(
                &repository,
                &benchmark_root,
                &attempt.identity,
                &source,
                "operator",
                "2026-08-05T00:01:00Z",
                "project",
                "compute",
                &corpus,
            )?;
            assert!(material.execution.measured_command[2].contains("bam_mapping_summary.awk"));
            assert_eq!(
                material.execution.parameters.overrides_path,
                "config-pipeline-parameters.yml"
            );
            assert_eq!(material.execution.parameters.effective_path, "effective-parameters.json");
            assert_eq!(material.execution.required_files.len(), 18);
            assert_eq!(material.execution.acceptance.metrics.len(), 4);
        }
        Ok(())
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
