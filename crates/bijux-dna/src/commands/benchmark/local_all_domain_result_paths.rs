use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};

pub(crate) const LOCAL_ALL_DOMAIN_SLURM_RUN_ID: &str = "all-domain-benchmark-dry-run";

pub(crate) fn benchmark_sample_scope(domain: &str, result_id: &str) -> Result<Option<String>> {
    let segments = result_id.split(':').collect::<Vec<_>>();
    if segments.len() != 5 {
        return Err(anyhow!(
            "all-domain result paths require five-part result ids, found `{result_id}`"
        ));
    }
    match domain {
        "fastq" | "bam" => Ok(Some(segments[3].to_string())),
        "vcf" => Ok(None),
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
