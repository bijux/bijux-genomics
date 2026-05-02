use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use crate::commands::cli::BenchmarkMatrixArgs;
use crate::commands::cli::env::registry_tools_for_stage;
use crate::commands::hpc::campaign_dry_run;

const BENCHMARK_MATRIX_SCHEMA_VERSION: &str = "bijux.hpc.benchmark_matrix.v1";

#[derive(Debug, Clone, Serialize)]
pub struct BenchmarkMatrixReport {
    pub schema_version: &'static str,
    pub campaign_id: String,
    pub domain: String,
    pub generated_at: String,
    pub rows: Vec<BenchmarkMatrixRow>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BenchmarkMatrixRow {
    pub row_id: String,
    pub matrix_domain: String,
    pub stage_id: String,
    pub tool_id: String,
}

fn now_timestamp_compact() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |delta| delta.as_secs());
    secs.to_string()
}

fn workspace_root() -> Result<PathBuf> {
    let mut cursor = std::env::current_dir().context("resolve current directory")?;
    loop {
        let domain_dir = cursor.join("domain");
        let registry = cursor.join("configs").join("ci").join("registry").join("tool_registry.toml");
        if domain_dir.is_dir() && registry.is_file() {
            return Ok(cursor);
        }
        let Some(parent) = cursor.parent() else {
            break;
        };
        cursor = parent.to_path_buf();
    }
    Err(anyhow!(
        "unable to locate workspace root containing domain/ and configs/ci/registry/tool_registry.toml"
    ))
}

fn domain_stage_ids(root: &Path, domain: &str) -> Result<Vec<String>> {
    let stages_dir = root.join("domain").join(domain).join("stages");
    if !stages_dir.is_dir() {
        return Err(anyhow!("stage catalog not found: {}", stages_dir.display()));
    }
    let mut stages = Vec::new();
    for entry in std::fs::read_dir(&stages_dir)
        .with_context(|| format!("read {}", stages_dir.display()))?
    {
        let path = entry?.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("yaml") {
            continue;
        }
        let Some(name) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        if name.starts_with('_') {
            continue;
        }
        stages.push(format!("{domain}.{name}"));
    }
    stages.sort();
    stages.dedup();
    Ok(stages)
}

fn registry_path_from_root(root: &Path) -> PathBuf {
    bijux_dna_infra::configs_file(root, "ci/registry/tool_registry.toml")
}

pub fn benchmark_matrix(args: &BenchmarkMatrixArgs) -> Result<BenchmarkMatrixReport> {
    if args.domain != "fastq" {
        return Err(anyhow!(
            "benchmark-matrix currently supports --domain fastq for this iteration"
        ));
    }
    let dry_run =
        campaign_dry_run(&args.config, args.env_file.as_deref(), args.user_overrides.as_deref())?;
    let root = workspace_root()?;
    let registry_path = registry_path_from_root(&root);
    let stages = domain_stage_ids(&root, &args.domain)?;
    let mut rows = Vec::new();
    for stage_id in stages {
        for tool_id in registry_tools_for_stage(&registry_path, &stage_id, None, "all")? {
            rows.push(BenchmarkMatrixRow {
                row_id: format!("{stage_id}::{tool_id}"),
                matrix_domain: args.domain.clone(),
                stage_id: stage_id.clone(),
                tool_id,
            });
        }
    }
    Ok(BenchmarkMatrixReport {
        schema_version: BENCHMARK_MATRIX_SCHEMA_VERSION,
        campaign_id: dry_run.campaign_id,
        domain: args.domain.clone(),
        generated_at: now_timestamp_compact(),
        rows,
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use super::domain_stage_ids;

    #[test]
    fn stage_catalog_lists_non_schema_fastq_entries() {
        let root = super::workspace_root().expect("workspace root");
        let stages = domain_stage_ids(&root, "fastq").expect("stages");
        assert!(stages.iter().all(|stage| stage.starts_with("fastq.")));
        assert!(stages.iter().any(|stage| stage == "fastq.validate_reads"));
        assert!(!stages.iter().any(|stage| stage.ends_with("._schema")));
    }
}
