use std::collections::HashMap;
use std::time::Duration;

use anyhow::{anyhow, Result};
use uuid::Uuid;

use bijux_dna_core::contract::ToolRegistry;
use bijux_dna_domain_fastq::{
    STAGE_CORRECT_ERRORS, STAGE_FILTER_READS, STAGE_MERGE_PAIRS, STAGE_TRIM_READS,
    STAGE_VALIDATE_READS,
};

use crate::api::{PlatformSpec, ToolImageSpec};
use crate::image_qa::fs::temp_out_dir;
use crate::image_qa::support::{
    adapter_hit_reads, docker_rm, ensure_gzip_integrity, output_fastq_stats, resolve_image_for_run,
    run_merge_container_with_timeout, run_tool_container_with_timeout,
    run_trim_container_with_timeout, run_validate_container_with_timeout,
    validate_execution_outputs, ResolvedImage, TrimExecutionOutput,
};
use crate::image_qa::QaDataset;

use super::postprocess::{find_fastq_in_dir, tool_contract};

const QA_TIMEOUT_SECS: u64 = 300;
const QA_MERGE_TIMEOUT_SECS: u64 = 300;

pub(crate) fn qa_trim_tool(
    tool: &str,
    platform: &PlatformSpec,
    catalog: &HashMap<String, ToolImageSpec>,
    registry: &ToolRegistry,
    dataset: &QaDataset,
    seqkit_image: &ResolvedImage,
) -> Result<()> {
    let contract = tool_contract(registry, STAGE_TRIM_READS.as_str(), tool)?;
    let spec = catalog.get(tool).ok_or_else(|| anyhow!("tool {tool} missing from images.toml"))?;
    let image = resolve_image_for_run(spec, platform)?;
    let out_dir = temp_out_dir("trim", tool)?;
    let container_name = format!("bijux-dna-qa-trim-{}-{}", tool, Uuid::new_v4());
    let timeout = Duration::from_secs(QA_TIMEOUT_SECS);
    let execution = match run_trim_container_with_timeout(
        tool,
        &image,
        &dataset.r1_dir,
        &dataset.r1,
        dataset.r2.as_deref(),
        &out_dir,
        &container_name,
        timeout,
    ) {
        Ok(execution) => execution,
        Err(err) => {
            let _ = docker_rm(&container_name);
            return Err(err);
        }
    };
    docker_rm(&container_name)?;
    if execution.exit_code != 0 {
        return Err(anyhow!("exit code {}", execution.exit_code));
    }
    validate_execution_outputs(contract, &out_dir)?;
    let TrimExecutionOutput { output_r1, output_r2, .. } = execution;
    let out_fastq = resolve_output_fastq(output_r1, "output FASTQ")?;
    ensure_gzip_integrity(&out_fastq)?;
    let output_stats = output_fastq_stats(seqkit_image, &out_dir, &out_fastq)?;
    if output_stats.reads == 0 {
        return Err(anyhow!("reads_out is zero"));
    }
    if output_stats.reads > dataset.input_stats_r1.reads {
        return Err(anyhow!(
            "reads_out {} exceeds reads_in {}",
            output_stats.reads,
            dataset.input_stats_r1.reads
        ));
    }
    let adapter = "AGATCGGAAGAGC";
    let input_hits = adapter_hit_reads(seqkit_image, &dataset.r1_dir, &dataset.r1, adapter)?;
    let output_hits = adapter_hit_reads(seqkit_image, &out_dir, &out_fastq, adapter)?;
    if input_hits >= 10 && output_hits >= input_hits {
        return Err(anyhow!("adapter hits not reduced: in={input_hits} out={output_hits}"));
    }

    if let Some(out_r2) = output_r2 {
        let out_r2 = resolve_output_fastq(out_r2, "output R2")?;
        ensure_gzip_integrity(&out_r2)?;
        let stats_r2 = output_fastq_stats(seqkit_image, &out_dir, &out_r2)?;
        if stats_r2.reads == 0 {
            return Err(anyhow!("reads_out_r2 is zero"));
        }
        if let Some(input_stats_r2) = dataset.input_stats_r2 {
            if stats_r2.reads > input_stats_r2.reads {
                return Err(anyhow!(
                    "reads_out_r2 {} exceeds reads_in_r2 {}",
                    stats_r2.reads,
                    input_stats_r2.reads
                ));
            }
        }
        let input_r2 = dataset.r2.as_deref().ok_or_else(|| anyhow!("r2 missing for paired QA"))?;
        let input_hits_r2 = adapter_hit_reads(seqkit_image, &dataset.r1_dir, input_r2, adapter)?;
        let output_hits_r2 = adapter_hit_reads(seqkit_image, &out_dir, &out_r2, adapter)?;
        if input_hits_r2 >= 10 && output_hits_r2 >= input_hits_r2 {
            return Err(anyhow!(
                "adapter hits not reduced for R2: in={input_hits_r2} out={output_hits_r2}"
            ));
        }
    }
    Ok(())
}

pub(crate) fn qa_validate_tool(
    tool: &str,
    platform: &PlatformSpec,
    catalog: &HashMap<String, ToolImageSpec>,
    registry: &ToolRegistry,
    dataset: &QaDataset,
) -> Result<()> {
    let contract = tool_contract(registry, STAGE_VALIDATE_READS.as_str(), tool)?;
    let spec = catalog.get(tool).ok_or_else(|| anyhow!("tool {tool} missing from images.toml"))?;
    let image = resolve_image_for_run(spec, platform)?;
    let out_dir = temp_out_dir("validate", tool)?;
    let container_name = format!("bijux-dna-qa-validate-{}-{}", tool, Uuid::new_v4());
    let timeout = Duration::from_secs(QA_TIMEOUT_SECS);
    let execution = match run_validate_container_with_timeout(
        tool,
        &image,
        &dataset.r1_dir,
        &dataset.r1,
        &out_dir,
        &container_name,
        timeout,
    ) {
        Ok(execution) => execution,
        Err(err) => {
            let _ = docker_rm(&container_name);
            return Err(err);
        }
    };
    docker_rm(&container_name)?;
    if execution.exit_code != 0 {
        return Err(anyhow!("exit code {}", execution.exit_code));
    }
    validate_execution_outputs(contract, &out_dir)?;
    Ok(())
}

pub(crate) fn qa_filter_tool(
    tool: &str,
    platform: &PlatformSpec,
    catalog: &HashMap<String, ToolImageSpec>,
    registry: &ToolRegistry,
    dataset: &QaDataset,
    seqkit_image: &ResolvedImage,
) -> Result<()> {
    let contract = tool_contract(registry, STAGE_FILTER_READS.as_str(), tool)?;
    let spec = catalog.get(tool).ok_or_else(|| anyhow!("tool {tool} missing from images.toml"))?;
    let image = resolve_image_for_run(spec, platform)?;
    let out_dir = temp_out_dir("filter", tool)?;
    let container_name = format!("bijux-dna-qa-filter-{}-{}", tool, Uuid::new_v4());
    let timeout = Duration::from_secs(QA_TIMEOUT_SECS);
    let execution = match run_tool_container_with_timeout(
        tool,
        &image,
        &dataset.r1_dir,
        &dataset.r1,
        &out_dir,
        &container_name,
        timeout,
    ) {
        Ok(execution) => execution,
        Err(err) => {
            let _ = docker_rm(&container_name);
            return Err(err);
        }
    };
    docker_rm(&container_name)?;
    if execution.exit_code != 0 {
        return Err(anyhow!("exit code {}", execution.exit_code));
    }
    validate_execution_outputs(contract, &out_dir)?;
    let out_fastq = execution.output_fastq.ok_or_else(|| anyhow!("output FASTQ missing"))?;
    let out_fastq = if out_fastq.exists() {
        out_fastq
    } else {
        let alt = out_fastq.with_extension("");
        if alt.exists() {
            alt
        } else {
            return Err(anyhow!("output FASTQ not found: {}", out_fastq.display()));
        }
    };
    let output_stats = output_fastq_stats(seqkit_image, &out_dir, &out_fastq)?;
    if output_stats.reads > dataset.input_stats_r1.reads {
        return Err(anyhow!(
            "reads_out {} exceeds reads_in {}",
            output_stats.reads,
            dataset.input_stats_r1.reads
        ));
    }
    Ok(())
}

pub(crate) fn qa_merge_tool(
    tool: &str,
    platform: &PlatformSpec,
    catalog: &HashMap<String, ToolImageSpec>,
    registry: &ToolRegistry,
    dataset: &QaDataset,
    seqkit_image: &ResolvedImage,
) -> Result<()> {
    let contract = tool_contract(registry, STAGE_MERGE_PAIRS.as_str(), tool)?;
    let spec = catalog.get(tool).ok_or_else(|| anyhow!("tool {tool} missing from images.toml"))?;
    let image = resolve_image_for_run(spec, platform)?;
    let r2 = dataset.r2.as_ref().ok_or_else(|| anyhow!("missing paired input"))?;
    let input_stats_r2 =
        dataset.input_stats_r2.as_ref().ok_or_else(|| anyhow!("missing paired stats"))?;
    let out_dir = temp_out_dir("merge", tool)?;
    let container_name = format!("bijux-dna-qa-merge-{}-{}", tool, Uuid::new_v4());
    let timeout = Duration::from_secs(QA_MERGE_TIMEOUT_SECS);
    let execution = match run_merge_container_with_timeout(
        tool,
        &image,
        &dataset.r1_dir,
        &dataset.r1,
        r2,
        &out_dir,
        &container_name,
        timeout,
    ) {
        Ok(execution) => execution,
        Err(err) => {
            let _ = docker_rm(&container_name);
            return Err(err);
        }
    };
    docker_rm(&container_name)?;
    if execution.exit_code != 0 {
        return Err(anyhow!("exit code {}", execution.exit_code));
    }
    validate_execution_outputs(contract, &out_dir)?;

    let merged_exists = execution.merged_fastq.exists();
    let unmerged_exists = execution.unmerged_r1.exists() && execution.unmerged_r2.exists();
    if !merged_exists && !unmerged_exists {
        return Err(anyhow!("merge outputs missing"));
    }

    let merged_reads = if merged_exists {
        let stats = output_fastq_stats(seqkit_image, &out_dir, &execution.merged_fastq)?;
        stats.reads
    } else {
        0
    };
    let min_reads = dataset.input_stats_r1.reads.min(input_stats_r2.reads);
    if merged_reads > min_reads {
        return Err(anyhow!(
            "merged_reads {merged_reads} exceeds min(reads_r1, reads_r2) {min_reads}"
        ));
    }
    Ok(())
}

pub(crate) fn qa_correct_tool(
    tool: &str,
    platform: &PlatformSpec,
    catalog: &HashMap<String, ToolImageSpec>,
    registry: &ToolRegistry,
    dataset: &QaDataset,
    seqkit_image: &ResolvedImage,
) -> Result<()> {
    let contract = tool_contract(registry, STAGE_CORRECT_ERRORS.as_str(), tool)?;
    let spec = catalog.get(tool).ok_or_else(|| anyhow!("tool {tool} missing from images.toml"))?;
    let image = resolve_image_for_run(spec, platform)?;
    let out_dir = temp_out_dir("correct", tool)?;
    let container_name = format!("bijux-dna-qa-correct-{}-{}", tool, Uuid::new_v4());
    let timeout = Duration::from_secs(QA_TIMEOUT_SECS);
    let execution = match run_tool_container_with_timeout(
        tool,
        &image,
        &dataset.r1_dir,
        &dataset.r1,
        &out_dir,
        &container_name,
        timeout,
    ) {
        Ok(execution) => execution,
        Err(err) => {
            let _ = docker_rm(&container_name);
            return Err(err);
        }
    };
    docker_rm(&container_name)?;
    if execution.exit_code != 0 {
        return Err(anyhow!("exit code {}", execution.exit_code));
    }
    validate_execution_outputs(contract, &out_dir)?;
    let out_fastq =
        if let Some(path) = execution.output_fastq { path } else { find_fastq_in_dir(&out_dir)? };
    let output_stats = output_fastq_stats(seqkit_image, &out_dir, &out_fastq)?;
    if output_stats.reads != dataset.input_stats_r1.reads {
        return Err(anyhow!(
            "reads_out {} must equal reads_in {}",
            output_stats.reads,
            dataset.input_stats_r1.reads
        ));
    }
    Ok(())
}

fn resolve_output_fastq(path: std::path::PathBuf, label: &str) -> Result<std::path::PathBuf> {
    if path.exists() {
        return Ok(path);
    }

    let alternate = path.with_extension("");
    if alternate.exists() {
        return Ok(alternate);
    }

    Err(anyhow!("{label} not found: {}", path.display()))
}
