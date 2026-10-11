//! Owner: checked stage-tool resource projection and domain defaults for HPC jobs.
use crate::{
    validate_stage_tool_resources_config, BenchmarkLocalHpcScratchJobResources,
    StageToolResourcesConfig,
};
use anyhow::{Context, Result};
use std::collections::BTreeMap;
/// Resource declarations indexed by domain, stage and tool identity.
pub type HpcJobResourceHints =
    BTreeMap<(String, String, String), BenchmarkLocalHpcScratchJobResources>;
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

/// Admit source declarations before projecting scheduler resource units.
/// # Errors
/// Rejects invalid source contracts and memory values that cannot be represented in MiB.
pub fn project_hpc_job_resources(config: &StageToolResourcesConfig) -> Result<HpcJobResourceHints> {
    validate_stage_tool_resources_config(config)?;
    config
        .rows
        .iter()
        .map(|row| {
            Ok((
                (row.domain.clone(), row.stage_id.clone(), row.tool_id.clone()),
                BenchmarkLocalHpcScratchJobResources {
                    cpus_per_task: row.threads,
                    memory_mb: row.memory_gb.checked_mul(1024).with_context(|| {
                        format!(
                            "HPC resource memory for {} / {} / {} exceeds the MiB representation",
                            row.domain, row.stage_id, row.tool_id
                        )
                    })?,
                    time_limit: minutes_to_time_limit(row.walltime_minutes),
                    scratch_gb: row.scratch_gb,
                },
            ))
        })
        .collect()
}
/// Resolve the exact tool declaration or the domain default.
#[must_use]
pub fn resolve_hpc_job_resources(
    resource_hints: &BTreeMap<(String, String, String), BenchmarkLocalHpcScratchJobResources>,
    domain: &str,
    stage_id: &str,
    tool_id: &str,
) -> BenchmarkLocalHpcScratchJobResources {
    resource_hints
        .get(&(domain.to_string(), stage_id.to_string(), tool_id.to_string()))
        .cloned()
        .unwrap_or_else(|| default_resource_hint(domain))
}

fn default_resource_hint(domain: &str) -> BenchmarkLocalHpcScratchJobResources {
    match domain {
        "fastq" => BenchmarkLocalHpcScratchJobResources {
            cpus_per_task: DEFAULT_FASTQ_CPUS,
            memory_mb: DEFAULT_FASTQ_MEMORY_MB,
            time_limit: DEFAULT_TIME_LIMIT.to_string(),
            scratch_gb: DEFAULT_FASTQ_SCRATCH_GB,
        },
        "bam" => BenchmarkLocalHpcScratchJobResources {
            cpus_per_task: DEFAULT_BAM_CPUS,
            memory_mb: DEFAULT_BAM_MEMORY_MB,
            time_limit: DEFAULT_TIME_LIMIT.to_string(),
            scratch_gb: DEFAULT_BAM_SCRATCH_GB,
        },
        "vcf" => BenchmarkLocalHpcScratchJobResources {
            cpus_per_task: DEFAULT_VCF_CPUS,
            memory_mb: DEFAULT_VCF_MEMORY_MB,
            time_limit: DEFAULT_TIME_LIMIT.to_string(),
            scratch_gb: DEFAULT_VCF_SCRATCH_GB,
        },
        _ => BenchmarkLocalHpcScratchJobResources {
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
