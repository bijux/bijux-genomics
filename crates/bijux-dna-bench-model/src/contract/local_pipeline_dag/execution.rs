//! Pure semantic admission for declared pipeline execution surfaces.

use super::config::require_non_empty;
use crate::model::BenchmarkLocalPipelineExecutionContext;
use anyhow::{anyhow, Result};
use std::collections::BTreeSet;

/// Validate declared execution surfaces independently of repository or site availability.
///
/// # Errors
/// Rejects blank identities, duplicate or unsupported surfaces and blank campaign paths.
pub fn validate_benchmark_local_pipeline_execution_context(
    execution_context: &BenchmarkLocalPipelineExecutionContext,
) -> Result<()> {
    require_non_empty(
        &execution_context.site_profile_id,
        "local pipeline execution_context must declare a non-empty `site_profile_id`",
    )?;
    validate_symbol_list(
        &execution_context.frontend_surfaces,
        "local pipeline execution_context.frontend_surfaces",
    )?;
    validate_symbol_list(
        &execution_context.compute_surfaces,
        "local pipeline execution_context.compute_surfaces",
    )?;
    if let Some(default_campaign_config) = &execution_context.default_campaign_config {
        require_non_empty(
            default_campaign_config,
            "local pipeline execution_context.default_campaign_config must not be empty when present",
        )?;
    }

    let allowed_frontend =
        BTreeSet::from(["downloads".to_string(), "ref-prep".to_string(), "snp-prep".to_string()]);
    for surface in &execution_context.frontend_surfaces {
        if !allowed_frontend.contains(surface) {
            return Err(anyhow!(
                "local pipeline execution_context.frontend_surfaces contains unsupported surface `{surface}`"
            ));
        }
    }

    let allowed_compute = BTreeSet::from([
        "fastq-qc".to_string(),
        "fastq-to-bam".to_string(),
        "bam-qc".to_string(),
        "bam-to-vcf".to_string(),
    ]);
    for surface in &execution_context.compute_surfaces {
        if !allowed_compute.contains(surface) {
            return Err(anyhow!(
                "local pipeline execution_context.compute_surfaces contains unsupported surface `{surface}`"
            ));
        }
    }

    Ok(())
}

fn validate_symbol_list(values: &[String], label: &str) -> Result<()> {
    let mut seen = BTreeSet::new();
    for value in values {
        if value.trim().is_empty() {
            return Err(anyhow!("{label} contains an empty symbol"));
        }
        if !seen.insert(value.clone()) {
            return Err(anyhow!("{label} repeats symbol `{value}`"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context() -> BenchmarkLocalPipelineExecutionContext {
        BenchmarkLocalPipelineExecutionContext {
            site_profile_id: "generic".into(),
            default_campaign_config: Some("benchmarks/configs/hpc/campaign.toml".into()),
            frontend_surfaces: vec!["downloads".into(), "ref-prep".into(), "snp-prep".into()],
            compute_surfaces: vec![
                "fastq-qc".into(),
                "fastq-to-bam".into(),
                "bam-qc".into(),
                "bam-to-vcf".into(),
            ],
        }
    }

    #[test]
    fn execution_contract_accepts_supported_surfaces_without_repository_access() -> Result<()> {
        let mut declaration = context();
        validate_benchmark_local_pipeline_execution_context(&declaration)?;
        declaration.site_profile_id = "custom-site".into();
        declaration.frontend_surfaces.clear();
        declaration.compute_surfaces.clear();
        declaration.default_campaign_config = None;
        validate_benchmark_local_pipeline_execution_context(&declaration)
    }

    #[test]
    fn execution_contract_rejects_wrong_surface_domains_duplicates_and_blank_symbols() {
        for symbols in [vec!["bam-qc"], vec!["downloads", "downloads"], vec![" "], vec!["unknown"]]
        {
            let mut declaration = context();
            declaration.frontend_surfaces = symbols.into_iter().map(str::to_owned).collect();
            assert!(validate_benchmark_local_pipeline_execution_context(&declaration).is_err());
        }
        for symbols in [vec!["ref-prep"], vec!["bam-qc", "bam-qc"], vec![""], vec!["unknown"]] {
            let mut declaration = context();
            declaration.compute_surfaces = symbols.into_iter().map(str::to_owned).collect();
            assert!(validate_benchmark_local_pipeline_execution_context(&declaration).is_err());
        }
    }

    #[test]
    fn execution_contract_rejects_blank_site_and_present_but_blank_campaign() {
        let mut declaration = context();
        declaration.site_profile_id = " ".into();
        assert!(validate_benchmark_local_pipeline_execution_context(&declaration).is_err());
        declaration = context();
        declaration.default_campaign_config = Some("\t".into());
        assert!(validate_benchmark_local_pipeline_execution_context(&declaration).is_err());
    }
}
