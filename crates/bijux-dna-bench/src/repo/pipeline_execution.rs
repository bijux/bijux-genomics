//! Owner: bijux-dna-bench
//! Explicit repository lookup for pipeline site-profile admission.

use anyhow::{anyhow, Result};
use bijux_dna_bench_model::{
    validate_benchmark_local_pipeline_execution_context, BenchmarkLocalPipelineExecutionContext,
};
use std::path::Path;

/// Admit execution declarations before resolving their selected site profile.
///
/// # Errors
/// Rejects invalid execution declarations and missing custom site profiles.
pub fn validate_benchmark_pipeline_execution_context(
    repo_root: &Path,
    execution_context: &BenchmarkLocalPipelineExecutionContext,
) -> Result<()> {
    validate_benchmark_local_pipeline_execution_context(execution_context)?;
    let site_profile_id = execution_context.site_profile_id.as_str();
    let built_in = matches!(site_profile_id, "lunarc" | "generic");
    let profile_path = repo_root
        .join("benchmarks/configs/hpc/campaign/site-profiles")
        .join(format!("{site_profile_id}.toml"));
    if !built_in && !profile_path.is_file() {
        return Err(anyhow!(
            "local pipeline execution_context site_profile_id `{site_profile_id}` is not a built-in profile and {} is missing",
            profile_path.display()
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn execution_site_lookup_uses_only_the_selected_repository() -> Result<()> {
        let first = tempfile::tempdir()?;
        let second = tempfile::tempdir()?;
        let mut context = BenchmarkLocalPipelineExecutionContext {
            site_profile_id: "generic".into(),
            default_campaign_config: None,
            frontend_surfaces: Vec::new(),
            compute_surfaces: Vec::new(),
        };
        validate_benchmark_pipeline_execution_context(first.path(), &context)?;
        context.site_profile_id = "lunarc".into();
        validate_benchmark_pipeline_execution_context(first.path(), &context)?;
        context.site_profile_id = "custom-site".into();
        assert!(validate_benchmark_pipeline_execution_context(first.path(), &context).is_err());
        let profile =
            first.path().join("benchmarks/configs/hpc/campaign/site-profiles/custom-site.toml");
        std::fs::create_dir_all(profile.parent().ok_or_else(|| anyhow!("profile parent"))?)?;
        std::fs::write(&profile, "# Site availability declaration\n")?;
        validate_benchmark_pipeline_execution_context(first.path(), &context)?;
        assert!(validate_benchmark_pipeline_execution_context(second.path(), &context).is_err());
        Ok(())
    }

    #[test]
    fn invalid_execution_declaration_precedes_missing_site_diagnostics() -> Result<()> {
        let root = tempfile::tempdir()?;
        let context = BenchmarkLocalPipelineExecutionContext {
            site_profile_id: "missing-site".into(),
            default_campaign_config: None,
            frontend_surfaces: vec!["bam-qc".into()],
            compute_surfaces: Vec::new(),
        };
        let result = validate_benchmark_pipeline_execution_context(root.path(), &context);
        let Err(error) = result else {
            return Err(anyhow!("invalid frontend surface was admitted"));
        };
        assert!(error.to_string().contains("unsupported surface `bam-qc`"));
        Ok(())
    }
}
