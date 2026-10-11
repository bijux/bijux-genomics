//! Semantic admission for local pipeline input declarations.

use std::collections::BTreeSet;
use std::path::Path;

use crate::model::{
    BenchmarkLocalPipelineAncientDnaPolicy as LocalPipelineAncientDnaPolicy,
    BenchmarkLocalPipelineDagConfig as LocalPipelineDagConfig,
    BenchmarkLocalPipelineProjectSourceConfig as LocalPipelineProjectSourceConfig,
    BenchmarkLocalPipelineReferenceAssets as LocalPipelineReferenceAssets,
    BenchmarkLocalPipelineVariantAssets as LocalPipelineVariantAssets,
};
use anyhow::{anyhow, Result};

/// Existing TOML and YAML local benchmark pipeline configuration protocol.
pub const BENCHMARK_LOCAL_PIPELINE_DAG_CONFIG_SCHEMA_V1: &str = "bijux.bench.local_pipeline_dag.v1";

/// Admit the existing configuration identity and required graph envelope.
///
/// Node topology, stage readiness and repository-backed scientific profile checks
/// belong to their respective services and must follow this shape admission.
///
/// # Errors
/// Rejects unknown schema versions, blank identity fields and an empty node list.
pub fn validate_benchmark_local_pipeline_dag_config(
    config: &crate::model::BenchmarkLocalPipelineDagConfig,
) -> anyhow::Result<()> {
    if config.schema_version != BENCHMARK_LOCAL_PIPELINE_DAG_CONFIG_SCHEMA_V1 {
        anyhow::bail!("unsupported local pipeline DAG schema `{}`", config.schema_version);
    }
    for (field, value) in [
        ("pipeline_id", &config.pipeline_id),
        ("summary", &config.summary),
        ("default_corpus_id", &config.default_corpus_id),
    ] {
        if value.trim().is_empty() {
            anyhow::bail!("local pipeline DAG must declare a non-empty `{field}`");
        }
    }
    if config.nodes.is_empty() {
        anyhow::bail!("local pipeline DAG must declare at least one `nodes` entry");
    }
    Ok(())
}

/// Admit declared sources, tool policy and required operational assets without repository IO.
///
/// Run after repository-backed reference and execution-context admission when composing
/// the CLI workflow, preserving its diagnostic precedence.
///
/// # Errors
/// Rejects invalid project sources, ancient DNA policy and reference or variant assets.
pub fn validate_benchmark_local_pipeline_dag_declarations(
    config_path: &Path,
    config: &LocalPipelineDagConfig,
) -> Result<()> {
    validate_project_sources(&config.project_sources)?;
    if let Some(policy) = &config.ancient_dna_policy {
        validate_ancient_dna_policy(config_path, config, policy)?;
    }
    if let Some(reference_assets) = &config.reference_assets {
        validate_reference_assets(reference_assets)?;
    }
    if let Some(variant_assets) = &config.variant_assets {
        validate_variant_assets(variant_assets)?;
    }
    validate_operational_asset_requirements(config)?;
    Ok(())
}

fn validate_reference_assets(reference_assets: &LocalPipelineReferenceAssets) -> Result<()> {
    require_non_empty(
        &reference_assets.directory_name,
        "local pipeline reference_assets must declare a non-empty `directory_name`",
    )?;
    require_http_url(&reference_assets.genome_url, "local pipeline reference_assets genome_url")?;
    require_non_empty(
        &reference_assets.genome_filename,
        "local pipeline reference_assets must declare a non-empty `genome_filename`",
    )?;
    require_http_url(
        &reference_assets.assembly_report_url,
        "local pipeline reference_assets assembly_report_url",
    )?;
    require_non_empty(
        &reference_assets.assembly_report_filename,
        "local pipeline reference_assets must declare a non-empty `assembly_report_filename`",
    )?;
    Ok(())
}

fn validate_variant_assets(variant_assets: &LocalPipelineVariantAssets) -> Result<()> {
    require_non_empty(
        &variant_assets.directory_name,
        "local pipeline variant_assets must declare a non-empty `directory_name`",
    )?;
    require_http_url(
        &variant_assets.source_vcf_url,
        "local pipeline variant_assets source_vcf_url",
    )?;
    require_non_empty(
        &variant_assets.source_vcf_filename,
        "local pipeline variant_assets must declare a non-empty `source_vcf_filename`",
    )?;
    require_non_empty(
        &variant_assets.refseq_vcf_filename,
        "local pipeline variant_assets must declare a non-empty `refseq_vcf_filename`",
    )?;
    require_non_empty(
        &variant_assets.annotated_vcf_filename,
        "local pipeline variant_assets must declare a non-empty `annotated_vcf_filename`",
    )?;
    require_non_empty(
        &variant_assets.snpeff.genome_id,
        "local pipeline variant_assets snpeff.genome_id must declare a non-empty value",
    )?;
    require_http_url(
        &variant_assets.snpeff.database_url,
        "local pipeline variant_assets snpeff.database_url",
    )?;
    require_non_empty(
        &variant_assets.snpeff.database_filename,
        "local pipeline variant_assets snpeff.database_filename must declare a non-empty value",
    )?;
    require_http_url(
        &variant_assets.snpeff.core_url,
        "local pipeline variant_assets snpeff.core_url",
    )?;
    require_non_empty(
        &variant_assets.snpeff.core_filename,
        "local pipeline variant_assets snpeff.core_filename must declare a non-empty value",
    )?;
    Ok(())
}

fn validate_operational_asset_requirements(config: &LocalPipelineDagConfig) -> Result<()> {
    let Some(execution_context) = &config.execution_context else {
        return Ok(());
    };
    if execution_context.frontend_surfaces.iter().any(|surface| surface == "ref-prep")
        && config.reference_assets.is_none()
    {
        return Err(anyhow!(
            "local pipeline execution_context.frontend_surfaces includes `ref-prep` but reference_assets is missing"
        ));
    }
    if execution_context.frontend_surfaces.iter().any(|surface| surface == "snp-prep")
        && config.variant_assets.is_none()
    {
        return Err(anyhow!(
            "local pipeline execution_context.frontend_surfaces includes `snp-prep` but variant_assets is missing"
        ));
    }
    Ok(())
}

fn validate_project_sources(project_sources: &[LocalPipelineProjectSourceConfig]) -> Result<()> {
    let mut seen_project_ids = BTreeSet::new();
    for source in project_sources {
        require_non_empty(
            &source.project_id,
            "local pipeline project_sources entries must declare a non-empty `project_id`",
        )?;
        require_non_empty(
            &source.metadata_url,
            "local pipeline project_sources entries must declare a non-empty `metadata_url`",
        )?;
        require_http_url(&source.metadata_url, "local pipeline project_sources metadata_url")?;
        if source.retries == 0 {
            return Err(anyhow!(
                "local pipeline project_sources entry `{}` must declare retries >= 1",
                source.project_id
            ));
        }
        if !seen_project_ids.insert(source.project_id.clone()) {
            return Err(anyhow!(
                "local pipeline project_sources repeats project_id `{}`",
                source.project_id
            ));
        }
    }
    Ok(())
}

fn require_http_url(value: &str, label: &str) -> Result<()> {
    if value.starts_with("http://") || value.starts_with("https://") {
        Ok(())
    } else {
        Err(anyhow!("{label} `{value}` must start with http:// or https://"))
    }
}

fn validate_ancient_dna_policy(
    config_path: &Path,
    config: &LocalPipelineDagConfig,
    policy: &LocalPipelineAncientDnaPolicy,
) -> Result<()> {
    require_non_empty(
        &policy.aligner_mode,
        "local pipeline ancient_dna_policy must declare a non-empty `aligner_mode`",
    )?;
    if !matches!(policy.aligner_mode.as_str(), "auto" | "bwa_aln" | "bwa_mem") {
        return Err(anyhow!(
            "local pipeline ancient_dna_policy aligner_mode `{}` is unsupported",
            policy.aligner_mode
        ));
    }
    if policy.auto_bwa_aln_p75_threshold_bp == 0 {
        return Err(anyhow!(
            "local pipeline ancient_dna_policy auto_bwa_aln_p75_threshold_bp must be greater than zero"
        ));
    }
    require_non_empty(
        &policy.adapter_cleanup_tool,
        "local pipeline ancient_dna_policy must declare a non-empty `adapter_cleanup_tool`",
    )?;
    require_non_empty(
        &policy.duplicate_stage_id,
        "local pipeline ancient_dna_policy must declare a non-empty `duplicate_stage_id`",
    )?;
    require_non_empty(
        &policy.duplicate_tool,
        "local pipeline ancient_dna_policy must declare a non-empty `duplicate_tool`",
    )?;
    require_non_empty(
        &policy.caller_tool,
        "local pipeline ancient_dna_policy must declare a non-empty `caller_tool`",
    )?;
    if !config.nodes.iter().any(|node| {
        node.node_id == policy.duplicate_stage_id || node.stage_id == policy.duplicate_stage_id
    }) {
        return Err(anyhow!(
            "{} declares ancient_dna_policy duplicate_stage_id `{}` but that stage is not present in the pipeline DAG",
            config_path.display(),
            policy.duplicate_stage_id
        ));
    }
    Ok(())
}

pub(super) fn require_non_empty(value: &str, message: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(anyhow!(message.to_string()));
    }
    Ok(())
}
