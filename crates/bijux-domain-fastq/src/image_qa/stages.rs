use std::collections::HashMap;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use bijux_bench::ImageQaOutcome;
use bijux_core::ToolRegistry;
use bijux_environment::api::{PlatformSpec, ToolImageSpec};
use uuid::Uuid;

use bijux_engine::api::output_fastq_stats;
use bijux_engine::api::validate_execution_outputs;
use bijux_engine::api::{
    docker_rm, resolve_image_for_run, run_merge_container_with_timeout,
    run_multiqc_container_with_timeout, run_tool_container_with_timeout,
    run_validate_container_with_timeout,
};

use super::helpers::temp_out_dir;
use super::{QaDataset, QaStage};

const QA_TIMEOUT_SECS: u64 = 300;
const QA_MERGE_TIMEOUT_SECS: u64 = 300;

pub(crate) fn run_stage_qa(
    stage: QaStage,
    tool: &str,
    platform: &PlatformSpec,
    catalog: &HashMap<String, ToolImageSpec>,
    registry: &ToolRegistry,
    dataset: &QaDataset,
    seqkit_image: &bijux_engine::api::ResolvedImage,
) -> ImageQaOutcome {
    let outcome = match stage {
        QaStage::Trim => qa_trim_tool(tool, platform, catalog, registry, dataset, seqkit_image),
        QaStage::Validate => qa_validate_tool(tool, platform, catalog, registry, dataset),
        QaStage::Filter => qa_filter_tool(tool, platform, catalog, registry, dataset, seqkit_image),
        QaStage::Merge => qa_merge_tool(tool, platform, catalog, registry, dataset, seqkit_image),
        QaStage::Correct => {
            qa_correct_tool(tool, platform, catalog, registry, dataset, seqkit_image)
        }
        QaStage::Qc2 => qa_qc2_tool(tool, platform, catalog, registry, dataset),
        QaStage::Umi => qa_umi_tool(tool, platform, catalog, registry, dataset, seqkit_image),
        QaStage::Stats => qa_stats_tool(tool, platform, catalog, registry, dataset),
        QaStage::Screen => qa_screen_tool(tool, platform, catalog, registry, dataset),
    };
    match outcome {
        Ok(()) => ImageQaOutcome::Pass,
        Err(err) => ImageQaOutcome::Fail(err.to_string()),
    }
}

fn qa_trim_tool(
    tool: &str,
    platform: &PlatformSpec,
    catalog: &HashMap<String, ToolImageSpec>,
    registry: &ToolRegistry,
    dataset: &QaDataset,
    seqkit_image: &bijux_engine::api::ResolvedImage,
) -> Result<()> {
    let contract = tool_contract(registry, "fastq.trim", tool)?;
    let spec = catalog
        .get(tool)
        .ok_or_else(|| anyhow!("tool {tool} missing from images.yaml"))?;
    let image = resolve_image_for_run(spec, platform)?;
    let out_dir = temp_out_dir("trim", tool)?;
    let container_name = format!("bijux-qa-trim-{}-{}", tool, Uuid::new_v4());
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
    let out_fastq = execution
        .output_fastq
        .ok_or_else(|| anyhow!("output FASTQ missing"))?;
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

fn qa_validate_tool(
    tool: &str,
    platform: &PlatformSpec,
    catalog: &HashMap<String, ToolImageSpec>,
    registry: &ToolRegistry,
    dataset: &QaDataset,
) -> Result<()> {
    let contract = tool_contract(registry, "fastq.validate", tool)?;
    let spec = catalog
        .get(tool)
        .ok_or_else(|| anyhow!("tool {tool} missing from images.yaml"))?;
    let image = resolve_image_for_run(spec, platform)?;
    let out_dir = temp_out_dir("validate", tool)?;
    let container_name = format!("bijux-qa-validate-{}-{}", tool, Uuid::new_v4());
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

fn qa_filter_tool(
    tool: &str,
    platform: &PlatformSpec,
    catalog: &HashMap<String, ToolImageSpec>,
    registry: &ToolRegistry,
    dataset: &QaDataset,
    seqkit_image: &bijux_engine::api::ResolvedImage,
) -> Result<()> {
    let contract = tool_contract(registry, "fastq.filter", tool)?;
    let spec = catalog
        .get(tool)
        .ok_or_else(|| anyhow!("tool {tool} missing from images.yaml"))?;
    let image = resolve_image_for_run(spec, platform)?;
    let out_dir = temp_out_dir("filter", tool)?;
    let container_name = format!("bijux-qa-filter-{}-{}", tool, Uuid::new_v4());
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
    let out_fastq = execution
        .output_fastq
        .ok_or_else(|| anyhow!("output FASTQ missing"))?;
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

fn qa_merge_tool(
    tool: &str,
    platform: &PlatformSpec,
    catalog: &HashMap<String, ToolImageSpec>,
    registry: &ToolRegistry,
    dataset: &QaDataset,
    seqkit_image: &bijux_engine::api::ResolvedImage,
) -> Result<()> {
    let contract = tool_contract(registry, "fastq.merge", tool)?;
    let spec = catalog
        .get(tool)
        .ok_or_else(|| anyhow!("tool {tool} missing from images.yaml"))?;
    let image = resolve_image_for_run(spec, platform)?;
    let r2 = dataset
        .r2
        .as_ref()
        .ok_or_else(|| anyhow!("missing paired input"))?;
    let input_stats_r2 = dataset
        .input_stats_r2
        .as_ref()
        .ok_or_else(|| anyhow!("missing paired stats"))?;
    let out_dir = temp_out_dir("merge", tool)?;
    let container_name = format!("bijux-qa-merge-{}-{}", tool, Uuid::new_v4());
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

fn qa_correct_tool(
    tool: &str,
    platform: &PlatformSpec,
    catalog: &HashMap<String, ToolImageSpec>,
    registry: &ToolRegistry,
    dataset: &QaDataset,
    seqkit_image: &bijux_engine::api::ResolvedImage,
) -> Result<()> {
    let contract = tool_contract(registry, "fastq.correct", tool)?;
    let spec = catalog
        .get(tool)
        .ok_or_else(|| anyhow!("tool {tool} missing from images.yaml"))?;
    let image = resolve_image_for_run(spec, platform)?;
    let out_dir = temp_out_dir("correct", tool)?;
    let container_name = format!("bijux-qa-correct-{}-{}", tool, Uuid::new_v4());
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
    let out_fastq = if let Some(path) = execution.output_fastq {
        path
    } else {
        find_fastq_in_dir(&out_dir)?
    };
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

fn qa_qc2_tool(
    tool: &str,
    platform: &PlatformSpec,
    catalog: &HashMap<String, ToolImageSpec>,
    registry: &ToolRegistry,
    dataset: &QaDataset,
) -> Result<()> {
    let contract = tool_contract(registry, "fastq.qc2", tool)?;
    let spec = catalog
        .get(tool)
        .ok_or_else(|| anyhow!("tool {tool} missing from images.yaml"))?;
    let image = resolve_image_for_run(spec, platform)?;
    let out_dir = temp_out_dir("qc2", tool)?;
    let container_name = format!("bijux-qa-qc2-{}-{}", tool, Uuid::new_v4());
    let timeout = Duration::from_secs(QA_TIMEOUT_SECS);
    if tool == "multiqc" {
        let fastqc_spec = catalog
            .get("fastqc")
            .ok_or_else(|| anyhow!("fastqc missing from images.yaml"))?;
        let fastqc_image = resolve_image_for_run(fastqc_spec, platform)?;
        let fastqc_dir = out_dir.join("fastqc");
        std::fs::create_dir_all(&fastqc_dir).context("create fastqc output dir")?;
        let fastqc_container = format!("bijux-qa-qc2-fastqc-{}", Uuid::new_v4());
        let fastqc_exec = match run_validate_container_with_timeout(
            "fastqc",
            &fastqc_image,
            &dataset.r1_dir,
            &dataset.r1,
            &fastqc_dir,
            &fastqc_container,
            timeout,
        ) {
            Ok(execution) => execution,
            Err(err) => {
                let _ = docker_rm(&fastqc_container);
                return Err(err);
            }
        };
        docker_rm(&fastqc_container)?;
        if fastqc_exec.exit_code != 0 {
            return Err(anyhow!("fastqc exit code {}", fastqc_exec.exit_code));
        }
        let execution = match run_multiqc_container_with_timeout(
            &image,
            &fastqc_dir,
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
        if !dir_has_files(&out_dir) {
            return Err(anyhow!("qc2 output missing"));
        }
        return Ok(());
    }

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
    if !dir_has_files(&out_dir) {
        return Err(anyhow!("qc2 output missing"));
    }
    Ok(())
}

fn qa_umi_tool(
    tool: &str,
    platform: &PlatformSpec,
    catalog: &HashMap<String, ToolImageSpec>,
    registry: &ToolRegistry,
    dataset: &QaDataset,
    seqkit_image: &bijux_engine::api::ResolvedImage,
) -> Result<()> {
    let contract = tool_contract(registry, "fastq.umi", tool)?;
    let spec = catalog
        .get(tool)
        .ok_or_else(|| anyhow!("tool {tool} missing from images.yaml"))?;
    let image = resolve_image_for_run(spec, platform)?;
    let out_dir = temp_out_dir("umi", tool)?;
    let container_name = format!("bijux-qa-umi-{}-{}", tool, Uuid::new_v4());
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
    let out_fastq = execution
        .output_fastq
        .ok_or_else(|| anyhow!("output FASTQ missing"))?;
    if !out_fastq.exists() {
        return Err(anyhow!("output FASTQ not found: {}", out_fastq.display()));
    }
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

fn qa_stats_tool(
    tool: &str,
    platform: &PlatformSpec,
    catalog: &HashMap<String, ToolImageSpec>,
    registry: &ToolRegistry,
    dataset: &QaDataset,
) -> Result<()> {
    let contract = tool_contract(registry, "fastq.stats", tool)?;
    let spec = catalog
        .get(tool)
        .ok_or_else(|| anyhow!("tool {tool} missing from images.yaml"))?;
    let image = resolve_image_for_run(spec, platform)?;
    let out_dir = temp_out_dir("stats", tool)?;
    let container_name = format!("bijux-qa-stats-{}-{}", tool, Uuid::new_v4());
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

fn qa_screen_tool(
    _tool: &str,
    _platform: &PlatformSpec,
    _catalog: &HashMap<String, ToolImageSpec>,
    _registry: &ToolRegistry,
    _dataset: &QaDataset,
) -> Result<()> {
    Err(anyhow!(
        "screen QA requires BIJUX_SCREEN_DB and is not enabled"
    ))
}

fn tool_contract<'a>(
    registry: &'a ToolRegistry,
    stage_id: &str,
    tool_id: &str,
) -> Result<&'a bijux_core::ExecutionContract> {
    let tool = registry
        .tool_by_id(stage_id, tool_id)
        .ok_or_else(|| anyhow!("tool {tool_id} missing from manifests"))?;
    Ok(&tool.execution_contract)
}

fn dir_has_files(path: &std::path::Path) -> bool {
    match std::fs::read_dir(path) {
        Ok(mut iter) => iter.next().is_some(),
        Err(_) => false,
    }
}

fn find_fastq_in_dir(dir: &std::path::Path) -> Result<std::path::PathBuf> {
    let entries = std::fs::read_dir(dir).context("read output dir")?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if is_fastq_path(&path) {
            return Ok(path);
        }
    }
    Err(anyhow!("output FASTQ not found in {}", dir.display()))
}

fn is_fastq_path(path: &std::path::Path) -> bool {
    let ext = path.extension().and_then(|ext| ext.to_str());
    if let Some(ext) = ext {
        if ext.eq_ignore_ascii_case("fastq") || ext.eq_ignore_ascii_case("fq") {
            return true;
        }
        if ext.eq_ignore_ascii_case("gz") {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                let nested = std::path::Path::new(stem);
                if let Some(stem_ext) = nested.extension().and_then(|s| s.to_str()) {
                    return stem_ext.eq_ignore_ascii_case("fastq")
                        || stem_ext.eq_ignore_ascii_case("fq");
                }
            }
        }
    }
    false
}
