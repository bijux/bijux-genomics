use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{anyhow, Context, Result};

use super::readiness::stage_tool_resources::{
    StageToolResourcesConfig, DEFAULT_STAGE_TOOL_RESOURCES_PATH,
    LOCAL_STAGE_TOOL_RESOURCES_SCHEMA_VERSION,
};

const DEFAULT_TIME_LIMIT: &str = "00:20:00";
const DEFAULT_FASTQ_CPUS: u32 = 4;
const DEFAULT_FASTQ_MEMORY_MB: u32 = 2048;
const DEFAULT_FASTQ_SCRATCH_GB: u32 = 2;
const DEFAULT_BAM_CPUS: u32 = 3;
const DEFAULT_BAM_MEMORY_MB: u32 = 2048;
const DEFAULT_BAM_SCRATCH_GB: u32 = 2;
const DEFAULT_VCF_CPUS: u32 = 1;
const DEFAULT_VCF_MEMORY_MB: u32 = 2048;
const DEFAULT_VCF_SCRATCH_GB: u32 = 2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LocalHpcJobResourceHint {
    pub(crate) cpus_per_task: u32,
    pub(crate) memory_mb: u32,
    pub(crate) time_limit: String,
    pub(crate) scratch_gb: u32,
}

pub(crate) fn load_local_hpc_job_resource_hints(
    repo_root: &Path,
) -> Result<BTreeMap<(String, String, String), LocalHpcJobResourceHint>> {
    let path = repo_root.join(DEFAULT_STAGE_TOOL_RESOURCES_PATH);
    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config = toml::from_str::<StageToolResourcesConfig>(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != LOCAL_STAGE_TOOL_RESOURCES_SCHEMA_VERSION {
        return Err(anyhow!(
            "unexpected stage-tool resources schema `{}` in {}",
            config.schema_version,
            path.display()
        ));
    }
    Ok(config
        .rows
        .into_iter()
        .map(|row| {
            (
                (row.domain, row.stage_id, row.tool_id),
                LocalHpcJobResourceHint {
                    cpus_per_task: row.threads.max(1),
                    memory_mb: row.memory_gb.max(1) * 1024,
                    time_limit: minutes_to_time_limit(row.walltime_minutes.max(1)),
                    scratch_gb: row.scratch_gb.max(1),
                },
            )
        })
        .collect())
}

pub(crate) fn resolve_local_hpc_job_resource_hint(
    resource_hints: &BTreeMap<(String, String, String), LocalHpcJobResourceHint>,
    domain: &str,
    stage_id: &str,
    tool_id: &str,
) -> LocalHpcJobResourceHint {
    resource_hints
        .get(&(domain.to_string(), stage_id.to_string(), tool_id.to_string()))
        .cloned()
        .unwrap_or_else(|| default_resource_hint(domain))
}

fn default_resource_hint(domain: &str) -> LocalHpcJobResourceHint {
    match domain {
        "fastq" => LocalHpcJobResourceHint {
            cpus_per_task: DEFAULT_FASTQ_CPUS,
            memory_mb: DEFAULT_FASTQ_MEMORY_MB,
            time_limit: DEFAULT_TIME_LIMIT.to_string(),
            scratch_gb: DEFAULT_FASTQ_SCRATCH_GB,
        },
        "bam" => LocalHpcJobResourceHint {
            cpus_per_task: DEFAULT_BAM_CPUS,
            memory_mb: DEFAULT_BAM_MEMORY_MB,
            time_limit: DEFAULT_TIME_LIMIT.to_string(),
            scratch_gb: DEFAULT_BAM_SCRATCH_GB,
        },
        "vcf" => LocalHpcJobResourceHint {
            cpus_per_task: DEFAULT_VCF_CPUS,
            memory_mb: DEFAULT_VCF_MEMORY_MB,
            time_limit: DEFAULT_TIME_LIMIT.to_string(),
            scratch_gb: DEFAULT_VCF_SCRATCH_GB,
        },
        _ => LocalHpcJobResourceHint {
            cpus_per_task: 1,
            memory_mb: 1024,
            time_limit: DEFAULT_TIME_LIMIT.to_string(),
            scratch_gb: 1,
        },
    }
}

fn minutes_to_time_limit(minutes: u32) -> String {
    let hours = minutes / 60;
    let remainder_minutes = minutes % 60;
    format!("{hours:02}:{remainder_minutes:02}:00")
}
