//! Benchmarking helpers for v1.
//!
//! Stability: v1 (stable).

use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

pub use bijux_dna_analyze::compare::compare_runs_with_baseline;
pub use bijux_dna_analyze::{build_rankings, compare_runs, print_bench_schema, RankInput};
pub use bijux_dna_bench::{
    acquire_fastq_corpus, admit_checked_in_benchmark_campaign, ensure_stage_campaign_attempts,
    load_checked_in_benchmark_campaign_plan, materialize_fastq_corpus,
    plan_checked_in_benchmark_campaign, prepare_checked_in_benchmark_campaign,
    publish_benchmark_data_index, publish_checked_in_benchmark_evidence_book,
    publish_governed_data_catalog_registry, replay_checked_in_benchmark_publication,
    status_checked_in_benchmark_campaign,
};
use bijux_dna_bench::{
    admitted_benchmark_stage_report_schema, benchmark_attempt_archive_schema,
    benchmark_attempt_schema, benchmark_cycle_status_schema,
    benchmark_pipeline_parameter_config_schema, benchmark_stage_interface_catalog_schema,
    benchmark_stage_report_schema,
};
pub use bijux_dna_bench::{
    AdmittedBenchmarkStageReport, BenchmarkAttemptArchive, BenchmarkAttemptRecord,
    BenchmarkCycleAdmission, BenchmarkCycleAdmissionRequest, BenchmarkCycleAdmissionState,
    BenchmarkCycleAdmissionUnit, BenchmarkCyclePlanRequest, BenchmarkCyclePreparation,
    BenchmarkCyclePreparationRequest, BenchmarkCyclePreparationState,
    BenchmarkCyclePreparationUnit, BenchmarkCycleStatus, BenchmarkCycleStatusRequest,
    BenchmarkDataIndexPublicationRequest, BenchmarkDataIndexRequest, BenchmarkEvidenceBookRequest,
    BenchmarkJobReference, BenchmarkLifecycleStatus, BenchmarkPipelineParameterConfig,
    BenchmarkReplayRequest, BenchmarkStageInterfaceCatalog, BenchmarkStageReport,
    CorpusAcquisitionRequest, CorpusMaterializationPublication, CorpusMaterializationRequest,
    GovernedDataCatalogRegistryPublication, GovernedDataCatalogRegistryPublicationRequest,
    BENCHMARK_ATTEMPT_ARCHIVE_SCHEMA_V1, BENCHMARK_ATTEMPT_SCHEMA_V1,
    BENCHMARK_CYCLE_STATUS_SCHEMA_V1, BENCHMARK_PIPELINE_PARAMETERS_SCHEMA_V1,
    BENCHMARK_REPORT_ADMISSION_SCHEMA_V1, BENCHMARK_STAGE_INTERFACES_SCHEMA_V1,
    BENCHMARK_STAGE_REPORT_SCHEMA_V1,
};
pub use bijux_dna_runner::api::{
    collect_benchmark_campaign_cycle, collect_corpus_materialization_slurm_job,
    load_corpus_materialization_slurm_job_handle, load_corpus_materialization_slurm_spec,
    load_prepared_corpus_materialization_slurm_job, observe_corpus_materialization_slurm_job,
    prepare_corpus_materialization_slurm_job, publish_corpus_materialization_evidence_correction,
    submit_benchmark_campaign_frontier, submit_prepared_corpus_materialization_job,
    BenchmarkCampaignSubmission, BenchmarkCampaignSubmissionState, BenchmarkCampaignSubmissionUnit,
    CorpusMaterializationCollection, CorpusMaterializationEvidence,
    CorpusMaterializationEvidenceCorrection, CorpusMaterializationEvidenceCorrectionRequest,
    CorpusMaterializationEvidenceStatus, CorpusMaterializationSlurmResources,
    CorpusMaterializationSlurmSpec, PreparedCorpusMaterializationSlurmJob, SlurmClientPaths,
    SlurmJobHandle, SlurmJobObservation, CORPUS_MATERIALIZATION_EVIDENCE_CORRECTION_SCHEMA_V1,
    CORPUS_MATERIALIZATION_EVIDENCE_SCHEMA_V1, CORPUS_MATERIALIZATION_PREPARED_JOB_SCHEMA_V1,
    CORPUS_MATERIALIZATION_SLURM_SPEC_SCHEMA_V1,
};

pub use crate::internal::public_surface::handlers::bam::BamBenchOutcome;
pub use crate::internal::public_surface::handlers::bam::{bench_bam_pipeline, bench_bam_stage};
pub use crate::surface::request_contracts::{BamRunArgs, BenchBamPipelineArgs, BenchBamStageArgs};
pub use bijux_dna_core::contract::objective_spec;
pub use bijux_dna_core::contract::{Objective, ObjectiveSpec, ObjectiveWeights};
pub use bijux_dna_planner_bam::stage_api::{bam_stage_completeness, BamStage};
pub use bijux_dna_planner_fastq::stage_api as fastq_banks;
pub use bijux_dna_planner_fastq::stage_api::args as fastq_args;
pub use bijux_dna_planner_fastq::stage_api::banks as fastq_bank_ops;
pub use bijux_dna_planner_fastq::stage_api::*;

/// Stability: v1 (stable).
/// Alias for `BenchOutcome<M: StageMetricSchema>` from the fastq handlers.
pub type BenchOutcome<M> = crate::internal::public_surface::handlers::fastq::BenchOutcome<M>;
pub use crate::internal::public_surface::handlers::fastq::*;

fn render_owned_schema(schema: schemars::Schema) -> serde_json::Value {
    match serde_json::to_value(schema) {
        Ok(value) => value,
        Err(error) => panic!("serialize JSON schema: {error}"),
    }
}

/// Render the governed benchmark attempt schema.
#[must_use]
pub fn render_benchmark_attempt_schema() -> serde_json::Value {
    render_owned_schema(benchmark_attempt_schema())
}

/// Write the governed benchmark attempt schema to disk.
///
/// # Errors
/// Returns an error if the schema cannot be written to `path`.
pub fn write_benchmark_attempt_schema(path: &Path) -> Result<()> {
    let schema = render_benchmark_attempt_schema();
    bijux_dna_infra::atomic_write_json(path, &schema)
        .with_context(|| format!("write {}", path.display()))
}

/// Render the governed benchmark attempt-archive schema.
#[must_use]
pub fn render_benchmark_attempt_archive_schema() -> serde_json::Value {
    render_owned_schema(benchmark_attempt_archive_schema())
}

/// Write the governed benchmark attempt-archive schema to disk.
///
/// # Errors
/// Returns an error if the schema cannot be written to `path`.
pub fn write_benchmark_attempt_archive_schema(path: &Path) -> Result<()> {
    let schema = render_benchmark_attempt_archive_schema();
    bijux_dna_infra::atomic_write_json(path, &schema)
        .with_context(|| format!("write {}", path.display()))
}

/// Render the governed benchmark cycle status schema.
#[must_use]
pub fn render_benchmark_cycle_status_schema() -> serde_json::Value {
    render_owned_schema(benchmark_cycle_status_schema())
}

/// Write the governed benchmark cycle status schema to disk.
///
/// # Errors
/// Returns an error if the schema cannot be written to `path`.
pub fn write_benchmark_cycle_status_schema(path: &Path) -> Result<()> {
    let schema = render_benchmark_cycle_status_schema();
    bijux_dna_infra::atomic_write_json(path, &schema)
        .with_context(|| format!("write {}", path.display()))
}

/// Render the governed benchmark stage-interface catalog schema.
#[must_use]
pub fn render_benchmark_stage_interfaces_schema() -> serde_json::Value {
    render_owned_schema(benchmark_stage_interface_catalog_schema())
}

/// Write the governed benchmark stage-interface catalog schema to disk.
///
/// # Errors
/// Returns an error if the schema cannot be written to `path`.
pub fn write_benchmark_stage_interfaces_schema(path: &Path) -> Result<()> {
    let schema = render_benchmark_stage_interfaces_schema();
    bijux_dna_infra::atomic_write_json(path, &schema)
        .with_context(|| format!("write {}", path.display()))
}

/// Render the governed benchmark pipeline-parameter schema.
#[must_use]
pub fn render_benchmark_pipeline_parameters_schema() -> serde_json::Value {
    render_owned_schema(benchmark_pipeline_parameter_config_schema())
}

/// Write the governed benchmark pipeline-parameter schema to disk.
///
/// # Errors
/// Returns an error if the schema cannot be written to `path`.
pub fn write_benchmark_pipeline_parameters_schema(path: &Path) -> Result<()> {
    let schema = render_benchmark_pipeline_parameters_schema();
    bijux_dna_infra::atomic_write_json(path, &schema)
        .with_context(|| format!("write {}", path.display()))
}

/// Render the governed benchmark stage-report schema.
#[must_use]
pub fn render_benchmark_stage_report_schema() -> serde_json::Value {
    render_owned_schema(benchmark_stage_report_schema())
}

/// Write the governed benchmark stage-report schema to disk.
///
/// # Errors
/// Returns an error if the schema cannot be written to `path`.
pub fn write_benchmark_stage_report_schema(path: &Path) -> Result<()> {
    let schema = render_benchmark_stage_report_schema();
    bijux_dna_infra::atomic_write_json(path, &schema)
        .with_context(|| format!("write {}", path.display()))
}

/// Render the governed benchmark report-admission schema.
#[must_use]
pub fn render_benchmark_report_admission_schema() -> serde_json::Value {
    render_owned_schema(admitted_benchmark_stage_report_schema())
}

/// Write the governed benchmark report-admission schema to disk.
///
/// # Errors
/// Returns an error if the schema cannot be written to `path`.
pub fn write_benchmark_report_admission_schema(path: &Path) -> Result<()> {
    let schema = render_benchmark_report_admission_schema();
    bijux_dna_infra::atomic_write_json(path, &schema)
        .with_context(|| format!("write {}", path.display()))
}

/// Render the governed FASTQ normalized metrics schema.
#[must_use]
pub fn render_fastq_normalized_metrics_schema() -> serde_json::Value {
    crate::internal::fastq::stages::preprocess::render_governed_fastq_normalized_metrics_schema()
}

/// Write the governed FASTQ normalized metrics schema to disk.
///
/// # Errors
/// Returns an error if the schema cannot be written to `path`.
pub fn write_fastq_normalized_metrics_schema(path: &Path) -> Result<()> {
    let schema = render_fastq_normalized_metrics_schema();
    bijux_dna_infra::atomic_write_json(path, &schema)
        .with_context(|| format!("write {}", path.display()))
}

/// Render the governed BAM normalized metrics schema.
#[must_use]
pub fn render_bam_normalized_metrics_schema() -> serde_json::Value {
    crate::internal::handlers::cross::render_governed_bam_normalized_metrics_schema()
}

/// Write the governed BAM normalized metrics schema to disk.
///
/// # Errors
/// Returns an error if the schema cannot be written to `path`.
pub fn write_bam_normalized_metrics_schema(path: &Path) -> Result<()> {
    let schema = render_bam_normalized_metrics_schema();
    bijux_dna_infra::atomic_write_json(path, &schema)
        .with_context(|| format!("write {}", path.display()))
}

/// Stability: v1 (stable).
/// Descriptor for one governed stage-specific VCF normalized metrics schema.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VcfNormalizedMetricsStageSchemaDescriptor {
    pub stage_id: String,
    pub schema_version: String,
    pub schema_id: String,
    pub file_name: String,
    pub extension_id: String,
    pub required_key_count: usize,
}

/// Render the governed VCF normalized metrics schema.
///
/// # Errors
/// Returns an error if the governed schema cannot be rendered.
pub fn render_vcf_normalized_metrics_schema() -> Result<serde_json::Value> {
    crate::internal::vcf::normalized_metrics_contract::render_vcf_normalized_metrics_schema()
}

/// Render the governed stage-specific VCF normalized metrics schema.
///
/// # Errors
/// Returns an error if the stage-specific schema cannot be rendered.
pub fn render_vcf_normalized_metrics_stage_schema(stage_id: &str) -> Result<serde_json::Value> {
    crate::internal::vcf::normalized_metrics_contract::render_vcf_normalized_metrics_stage_schema(
        stage_id,
    )
}

/// Validate a normalized VCF metrics payload against the governed stage schema contract.
///
/// # Errors
/// Returns an error if the payload does not satisfy the governed schema contract.
pub fn validate_vcf_normalized_metrics(
    metrics: &serde_json::Value,
) -> Result<VcfNormalizedMetricsStageSchemaDescriptor> {
    crate::internal::vcf::normalized_metrics_contract::validate_vcf_normalized_metrics(metrics).map(
        |descriptor| VcfNormalizedMetricsStageSchemaDescriptor {
            stage_id: descriptor.stage_id,
            schema_version: descriptor.schema_version,
            schema_id: descriptor.schema_id,
            file_name: descriptor.file_name,
            extension_id: descriptor.extension_id,
            required_key_count: descriptor.required_key_count,
        },
    )
}

/// Describe the governed stage-specific VCF normalized metrics schema set.
///
/// # Errors
/// Returns an error if the governed stage schema descriptors cannot be enumerated.
pub fn vcf_normalized_metrics_stage_schema_descriptors(
) -> Result<Vec<VcfNormalizedMetricsStageSchemaDescriptor>> {
    crate::internal::vcf::normalized_metrics_contract::vcf_normalized_metrics_stage_descriptors()
        .map(|descriptors| {
            descriptors
                .into_iter()
                .map(|descriptor| VcfNormalizedMetricsStageSchemaDescriptor {
                    stage_id: descriptor.stage_id,
                    schema_version: descriptor.schema_version,
                    schema_id: descriptor.schema_id,
                    file_name: descriptor.file_name,
                    extension_id: descriptor.extension_id,
                    required_key_count: descriptor.required_key_count,
                })
                .collect()
        })
}

/// Write the governed VCF normalized metrics schema to disk.
///
/// # Errors
/// Returns an error if the schema cannot be written to `path`.
pub fn write_vcf_normalized_metrics_schema(path: &Path) -> Result<()> {
    let schema = render_vcf_normalized_metrics_schema()?;
    bijux_dna_infra::atomic_write_json(path, &schema)
        .with_context(|| format!("write {}", path.display()))
}

/// Write the governed stage-specific VCF normalized metrics schema set to disk.
///
/// # Errors
/// Returns an error if the directory cannot be prepared or any schema cannot be written.
pub fn write_vcf_normalized_metrics_stage_schemas(
    stage_dir: &Path,
) -> Result<Vec<VcfNormalizedMetricsStageSchemaDescriptor>> {
    std::fs::create_dir_all(stage_dir)
        .with_context(|| format!("create {}", stage_dir.display()))?;
    let descriptors = vcf_normalized_metrics_stage_schema_descriptors()?;
    for descriptor in &descriptors {
        let schema = render_vcf_normalized_metrics_stage_schema(&descriptor.stage_id)?;
        let path = stage_dir.join(&descriptor.file_name);
        bijux_dna_infra::atomic_write_json(&path, &schema)
            .with_context(|| format!("write {}", path.display()))?;
    }
    Ok(descriptors)
}
