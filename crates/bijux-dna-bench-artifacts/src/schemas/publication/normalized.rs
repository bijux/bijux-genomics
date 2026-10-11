use anyhow::{Context, Result};
use bijux_dna_bench_model::normalized_result::vcf_envelope::VcfNormalizedMetricsStageSchemaDescriptor;
use std::path::Path;
/// Render the governed FASTQ normalized metrics schema.
#[must_use]
pub fn render_fastq_normalized_metrics_schema() -> serde_json::Value {
    bijux_dna_bench_model::normalized_result::fastq_envelope::render_fastq_normalized_metrics_schema(
    )
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
    bijux_dna_bench_model::normalized_result::bam_envelope::render_bam_normalized_metrics_schema()
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

/// Render the governed VCF normalized metrics schema.
///
/// # Errors
/// Returns an error if the governed schema cannot be rendered.
pub fn render_vcf_normalized_metrics_schema() -> Result<serde_json::Value> {
    bijux_dna_bench_model::normalized_result::vcf_envelope::render_vcf_normalized_metrics_schema()
}

/// Render the governed stage-specific VCF normalized metrics schema.
///
/// # Errors
/// Returns an error if the stage-specific schema cannot be rendered.
pub fn render_vcf_normalized_metrics_stage_schema(stage_id: &str) -> Result<serde_json::Value> {
    bijux_dna_bench_model::normalized_result::vcf_envelope::render_vcf_normalized_metrics_stage_schema(stage_id)
}

/// Validate a normalized VCF metrics payload against the governed stage schema contract.
///
/// # Errors
/// Returns an error if the payload does not satisfy the governed schema contract.
pub fn validate_vcf_normalized_metrics(
    metrics: &serde_json::Value,
) -> Result<VcfNormalizedMetricsStageSchemaDescriptor> {
    bijux_dna_bench_model::normalized_result::vcf_envelope::validate_vcf_normalized_metrics(metrics)
}

/// Describe the governed stage-specific VCF normalized metrics schema set.
///
/// # Errors
/// Returns an error if the governed stage schema descriptors cannot be enumerated.
pub fn vcf_normalized_metrics_stage_schema_descriptors(
) -> Result<Vec<VcfNormalizedMetricsStageSchemaDescriptor>> {
    bijux_dna_bench_model::normalized_result::vcf_envelope::vcf_normalized_metrics_stage_descriptors(
    )
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
    bijux_dna_infra::ensure_dir(stage_dir)
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
