use anyhow::{bail, Result};
use bijux_dna_bench_artifacts::schemas::paths::{
    DEFAULT_VCF_NORMALIZED_METRICS_SCHEMA_PATH, DEFAULT_VCF_NORMALIZED_METRICS_STAGE_DIR,
};
use bijux_dna_bench_artifacts::schemas::validation::{
    validate_vcf_schemas, DEFAULT_VCF_SCHEMA_VALIDATION_REPORT_PATH,
};
use bijux_dna_bench_artifacts::schemas::vcf_report::render_vcf_normalized_metrics_schema;
use std::path::Path;
pub(super) fn normalized_schemas(repo_root: &Path) -> Result<String> {
    let schema_report = render_vcf_normalized_metrics_schema(
        repo_root,
        Path::new(DEFAULT_VCF_NORMALIZED_METRICS_SCHEMA_PATH),
        Path::new(DEFAULT_VCF_NORMALIZED_METRICS_STAGE_DIR),
    )?;
    if schema_report.stage_count != 20 || schema_report.extension_count != 20 {
        bail!(
            "VCF normalized metrics schema report drifted: stage_count={}, extension_count={}",
            schema_report.stage_count,
            schema_report.extension_count
        );
    }

    let validation_report = validate_vcf_schemas(
        repo_root,
        Path::new(DEFAULT_VCF_SCHEMA_VALIDATION_REPORT_PATH),
        Path::new(DEFAULT_VCF_NORMALIZED_METRICS_SCHEMA_PATH),
        Path::new(DEFAULT_VCF_NORMALIZED_METRICS_STAGE_DIR),
    )?;
    if !validation_report.passes_gate
        || !validation_report.shared_schema_matches
        || validation_report.stage_count != 20
        || validation_report.required_stage_count != 20
        || validation_report.exact_stage_schema_file_count != 20
    {
        bail!("VCF schema validation drifted from the governed pass state");
    }
    Ok("validated the governed shared and stage-specific VCF normalized metrics schemas"
        .to_string())
}
