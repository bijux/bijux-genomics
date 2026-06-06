//! Benchmarking helpers for v1.
//!
//! Stability: v1 (stable).

use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

pub use bijux_dna_analyze::compare::compare_runs_with_baseline;
pub use bijux_dna_analyze::{build_rankings, compare_runs, print_bench_schema, RankInput};

pub use crate::internal::public_bridge::handlers::bam::BamBenchOutcome;
pub use crate::internal::public_bridge::handlers::bam::{bench_bam_pipeline, bench_bam_stage};
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
pub type BenchOutcome<M> = crate::internal::public_bridge::handlers::fastq::BenchOutcome<M>;
pub use crate::internal::public_bridge::handlers::fastq::*;

/// Render the governed FASTQ normalized metrics schema.
#[must_use]
pub fn render_fastq_normalized_metrics_schema() -> serde_json::Value {
    crate::internal::fastq::stages::preprocess::render_governed_fastq_normalized_metrics_schema()
}

/// Write the governed FASTQ normalized metrics schema to disk.
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
pub fn write_bam_normalized_metrics_schema(path: &Path) -> Result<()> {
    let schema = render_bam_normalized_metrics_schema();
    bijux_dna_infra::atomic_write_json(path, &schema)
        .with_context(|| format!("write {}", path.display()))
}

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
pub fn render_vcf_normalized_metrics_schema() -> Result<serde_json::Value> {
    crate::internal::vcf::normalized_metrics_contract::render_vcf_normalized_metrics_schema()
}

/// Render the governed stage-specific VCF normalized metrics schema.
pub fn render_vcf_normalized_metrics_stage_schema(stage_id: &str) -> Result<serde_json::Value> {
    crate::internal::vcf::normalized_metrics_contract::render_vcf_normalized_metrics_stage_schema(
        stage_id,
    )
}

/// Validate a normalized VCF metrics payload against the governed stage schema contract.
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
pub fn write_vcf_normalized_metrics_schema(path: &Path) -> Result<()> {
    let schema = render_vcf_normalized_metrics_schema()?;
    bijux_dna_infra::atomic_write_json(path, &schema)
        .with_context(|| format!("write {}", path.display()))
}

/// Write the governed stage-specific VCF normalized metrics schema set to disk.
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
