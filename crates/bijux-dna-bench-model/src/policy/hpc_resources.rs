//! Owner: benchmark stage walltime profiles and checked resource hints for HPC jobs.
use crate::{
    validate_stage_tool_resources_config, BenchmarkLocalHpcScratchJobResources,
    StageToolResourcesConfig,
};
use anyhow::{Context, Result};
use bijux_dna_core::id_catalog::{
    FASTQ_CLUSTER_OTUS, FASTQ_CORRECT, FASTQ_DEPLETE_HOST, FASTQ_DEPLETE_REFERENCE_CONTAMINANTS,
    FASTQ_DEPLETE_RRNA, FASTQ_INDEX_REFERENCE, FASTQ_INFER_ASVS, FASTQ_PROFILE_READ_LENGTHS,
    FASTQ_REMOVE_CHIMERAS, FASTQ_SCREEN, FASTQ_STATS_NEUTRAL, FASTQ_VALIDATE_READS,
};
use bijux_dna_domain_vcf::VcfDomainStage;
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

const FASTQ_MINUTES_QC: u32 = 10;
const FASTQ_MINUTES_TRANSFORM: u32 = 15;
const FASTQ_MINUTES_REFERENCE: u32 = 20;
const FASTQ_MINUTES_HEAVY_ANALYSIS: u32 = 30;
const FASTQ_MINUTES_DADA2: u32 = 45;
const VCF_MINUTES_STANDARD: u32 = 20;
const VCF_MINUTES_PREPARE_PANEL: u32 = 30;
const VCF_MINUTES_PANEL_WORKFLOW: u32 = 60;

/// Return the governed benchmark walltime hint for a FASTQ stage/tool pair.
///
/// DADA2 ASV inference takes its specific budget before the stage profiles; unprofiled
/// pairs retain the transformation fallback. This is a declaration hint, not a runtime estimate.
#[must_use]
pub fn fastq_benchmark_walltime_minutes(stage_id: &str, tool_id: &str) -> u32 {
    match (stage_id, tool_id) {
        (FASTQ_INFER_ASVS, "dada2") => FASTQ_MINUTES_DADA2,
        (FASTQ_CORRECT | FASTQ_CLUSTER_OTUS | FASTQ_REMOVE_CHIMERAS, _) => {
            FASTQ_MINUTES_HEAVY_ANALYSIS
        }
        (
            FASTQ_DEPLETE_HOST
            | FASTQ_DEPLETE_REFERENCE_CONTAMINANTS
            | FASTQ_DEPLETE_RRNA
            | FASTQ_INDEX_REFERENCE
            | FASTQ_SCREEN,
            _,
        ) => FASTQ_MINUTES_REFERENCE,
        (FASTQ_PROFILE_READ_LENGTHS | FASTQ_STATS_NEUTRAL | FASTQ_VALIDATE_READS, _) => {
            FASTQ_MINUTES_QC
        }
        _ => FASTQ_MINUTES_TRANSFORM,
    }
}

/// Return the governed benchmark walltime hint for a typed VCF stage.
///
/// Panel workflows, panel preparation and other stages retain their distinct budgets.
/// This is a declaration hint, not a runtime estimate.
#[must_use]
pub fn vcf_benchmark_walltime_minutes(stage: VcfDomainStage) -> u32 {
    match stage {
        VcfDomainStage::Phasing | VcfDomainStage::Impute | VcfDomainStage::ImputationMetrics => {
            VCF_MINUTES_PANEL_WORKFLOW
        }
        VcfDomainStage::PrepareReferencePanel => VCF_MINUTES_PREPARE_PANEL,
        _ => VCF_MINUTES_STANDARD,
    }
}

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
