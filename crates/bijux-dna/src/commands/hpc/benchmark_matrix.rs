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
    pub domains: Vec<String>,
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

#[derive(Debug, Clone, Copy)]
struct CrossBridge {
    id: &'static str,
    from_stage: &'static str,
    to_stage: &'static str,
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

fn cross_bridges() -> &'static [CrossBridge] {
    &[
        CrossBridge {
            id: "fastq_to_bam",
            from_stage: "fastq.trim_reads",
            to_stage: "bam.align",
        },
        CrossBridge {
            id: "bam_to_vcf",
            from_stage: "bam.genotyping",
            to_stage: "vcf.call",
        },
        CrossBridge {
            id: "fastq_to_vcf",
            from_stage: "fastq.trim_reads",
            to_stage: "vcf.call_gl",
        },
    ]
}

fn resolve_matrix_domains(value: &str) -> Result<Vec<String>> {
    match value {
        "all" => Ok(vec![
            "fastq".to_string(),
            "bam".to_string(),
            "vcf".to_string(),
            "cross".to_string(),
        ]),
        "fastq" | "bam" | "vcf" => Ok(vec![value.to_string()]),
        "cross" => Ok(vec!["cross".to_string()]),
        other => Err(anyhow!(
            "benchmark-matrix supports --domain fastq|bam|vcf|cross|all; got `{other}`"
        )),
    }
}

pub fn benchmark_matrix(args: &BenchmarkMatrixArgs) -> Result<BenchmarkMatrixReport> {
    let domains = resolve_matrix_domains(&args.domain)?;
    let dry_run =
        campaign_dry_run(&args.config, args.env_file.as_deref(), args.user_overrides.as_deref())?;
    let root = workspace_root()?;
    let registry_path = registry_path_from_root(&root);
    let mut rows = Vec::new();
    for domain in &domains {
        if domain == "cross" {
            for bridge in cross_bridges() {
                let left_tools = registry_tools_for_stage(&registry_path, bridge.from_stage, None, "all")
                    .unwrap_or_default();
                let right_tools = registry_tools_for_stage(&registry_path, bridge.to_stage, None, "all")
                    .unwrap_or_default();
                for left in &left_tools {
                    for right in &right_tools {
                        let stage_binding = format!("{}=>{}", bridge.from_stage, bridge.to_stage);
                        let tool_binding = format!("{left}=>{right}");
                        rows.push(BenchmarkMatrixRow {
                            row_id: format!("cross.{}::{}::{}", bridge.id, stage_binding, tool_binding),
                            matrix_domain: "cross".to_string(),
                            stage_id: stage_binding,
                            tool_id: tool_binding,
                        });
                    }
                }
            }
            continue;
        }
        let stages = domain_stage_ids(&root, domain)?;
        for stage_id in stages {
            for tool_id in registry_tools_for_stage(&registry_path, &stage_id, None, "all")? {
                rows.push(BenchmarkMatrixRow {
                    row_id: format!("{stage_id}::{tool_id}"),
                    matrix_domain: domain.clone(),
                    stage_id: stage_id.clone(),
                    tool_id,
                });
            }
        }
    }
    Ok(BenchmarkMatrixReport {
        schema_version: BENCHMARK_MATRIX_SCHEMA_VERSION,
        campaign_id: dry_run.campaign_id,
        domain: args.domain.clone(),
        domains,
        generated_at: now_timestamp_compact(),
        rows,
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use super::{cross_bridges, domain_stage_ids, resolve_matrix_domains};

    #[test]
    fn stage_catalog_lists_non_schema_fastq_entries() {
        let root = super::workspace_root().expect("workspace root");
        let stages = domain_stage_ids(&root, "fastq").expect("stages");
        assert!(stages.iter().all(|stage| stage.starts_with("fastq.")));
        assert!(stages.iter().any(|stage| stage == "fastq.validate_reads"));
        assert!(!stages.iter().any(|stage| stage.ends_with("._schema")));
    }

    #[test]
    fn stage_catalog_lists_non_schema_bam_entries() {
        let root = super::workspace_root().expect("workspace root");
        let stages = domain_stage_ids(&root, "bam").expect("stages");
        assert!(stages.iter().all(|stage| stage.starts_with("bam.")));
        assert!(stages.iter().any(|stage| stage == "bam.align"));
    }

    #[test]
    fn stage_catalog_lists_non_schema_vcf_entries() {
        let root = super::workspace_root().expect("workspace root");
        let stages = domain_stage_ids(&root, "vcf").expect("stages");
        assert!(stages.iter().all(|stage| stage.starts_with("vcf.")));
        assert!(stages.iter().any(|stage| stage == "vcf.call"));
    }

    #[test]
    fn matrix_domain_selector_supports_all_and_single_domains() {
        assert_eq!(
            resolve_matrix_domains("all").expect("all"),
            vec![
                "fastq".to_string(),
                "bam".to_string(),
                "vcf".to_string(),
                "cross".to_string()
            ]
        );
        assert_eq!(
            resolve_matrix_domains("bam").expect("bam"),
            vec!["bam".to_string()]
        );
        assert_eq!(
            resolve_matrix_domains("cross").expect("cross"),
            vec!["cross".to_string()]
        );
        assert!(resolve_matrix_domains("unknown").is_err());
    }

    #[test]
    fn cross_bridge_catalog_is_populated() {
        let bridges = cross_bridges();
        assert!(bridges.len() >= 3);
        assert!(bridges.iter().any(|bridge| bridge.id == "fastq_to_bam"));
    }
}
