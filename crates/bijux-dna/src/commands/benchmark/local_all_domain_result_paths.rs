use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};

pub(crate) const LOCAL_ALL_DOMAIN_SLURM_RUN_ID: &str = "all-domain-benchmark-dry-run";

pub(crate) fn benchmark_sample_scope(domain: &str, result_id: &str) -> Result<Option<String>> {
    let parsed =
        crate::commands::benchmark::benchmark_result_ids::parse_benchmark_result_id(result_id)?;
    if parsed.domain != domain {
        return Err(anyhow!(
            "all-domain result paths expected domain `{domain}` but result id `{result_id}` belongs to `{}`",
            parsed.domain
        ));
    }
    match parsed.scope_kind {
        crate::commands::benchmark::benchmark_result_ids::BenchmarkResultScopeKind::SampleScope => {
            Ok(Some(parsed.scope_id))
        }
        crate::commands::benchmark::benchmark_result_ids::BenchmarkResultScopeKind::AssetProfile => {
            Ok(None)
        }
    }
}

pub(crate) fn benchmark_result_scope_id(result_id: &str) -> Result<String> {
    Ok(crate::commands::benchmark::benchmark_result_ids::parse_benchmark_result_id(result_id)?
        .scope_id)
}

pub(crate) fn benchmark_result_scope_kind(
    result_id: &str,
) -> Result<crate::commands::benchmark::benchmark_result_ids::BenchmarkResultScopeKind> {
    Ok(crate::commands::benchmark::benchmark_result_ids::parse_benchmark_result_id(result_id)?
        .scope_kind)
}

pub(crate) fn benchmark_result_matches_identity(
    result_id: &str,
    domain: &str,
    corpus_id: &str,
    stage_id: &str,
    tool_id: &str,
) -> Result<bool> {
    let parsed =
        crate::commands::benchmark::benchmark_result_ids::parse_benchmark_result_id(result_id)?;
    Ok(parsed.domain == domain
        && parsed.corpus_id == corpus_id
        && parsed.stage_id == stage_id
        && parsed.tool_id == tool_id)
}

pub(crate) fn benchmark_result_asset_profile_id(
    domain: &str,
    asset_profile_id: &str,
    result_id: &str,
) -> Result<String> {
    if benchmark_sample_scope(domain, result_id)?.is_some() {
        Ok(asset_profile_id.to_string())
    } else {
        benchmark_result_scope_id(result_id)
    }
}

pub(crate) fn benchmark_result_sample_scope(
    domain: &str,
    result_id: &str,
) -> Result<Option<String>> {
    match domain {
        "fastq" | "bam" | "vcf" => benchmark_sample_scope(domain, result_id),
        other => Err(anyhow!("all-domain result paths do not support legacy domain `{other}`")),
    }
}

pub(crate) fn benchmark_result_root(
    root_path: &Path,
    domain: &str,
    stage_id: &str,
    tool_id: &str,
    corpus_id: &str,
    asset_profile_id: &str,
    result_id: &str,
) -> Result<PathBuf> {
    let mut result_root = root_path
        .join("runs")
        .join(LOCAL_ALL_DOMAIN_SLURM_RUN_ID)
        .join(domain)
        .join(stage_id)
        .join(tool_id)
        .join(corpus_id);
    if let Some(sample_scope) = benchmark_sample_scope(domain, result_id)? {
        result_root = result_root.join(sample_scope);
    } else {
        result_root = result_root.join(asset_profile_id);
    }
    Ok(result_root)
}

pub(crate) fn essential_pipeline_result_root(
    root_path: &Path,
    domain: &str,
    pipeline_id: &str,
    node_id: &str,
    tool_id: &str,
    corpus_id: &str,
    sample_scope: &str,
) -> PathBuf {
    root_path
        .join("runs")
        .join(LOCAL_ALL_DOMAIN_SLURM_RUN_ID)
        .join(domain)
        .join(pipeline_id)
        .join(node_id)
        .join(tool_id)
        .join(corpus_id)
        .join(sample_scope)
}
