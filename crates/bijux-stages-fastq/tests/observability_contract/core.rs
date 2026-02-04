use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use anyhow::{Context, Result};
use bijux_exec::primitives::execute_stage_plan as execute_plan;
use bijux_core::{
    parameters_json_canonicalization, params_hash, CommandSpecV1, ContainerImageRefV1,
    ToolConstraints, ToolExecutionSpecV1, ToolId,
};
use bijux_core::StagePlanV1;
use bijux_environment::api::RunnerKind;
use bijux_stages_fastq::fastq::{
    correct, filter, merge, qc_post, screen, stats_neutral, trim, umi, validate_pre,
};

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn write_fake_docker(dir: &Path) -> Result<PathBuf> {
    let bin_dir = dir.join("bin");
    bijux_infra::ensure_dir(&bin_dir)?;
    let docker_path = bin_dir.join("docker");
    let script = r#"#!/bin/sh
set -e
cmd="$1"
shift || true
case "$cmd" in
  run)
    echo "fake-container-id"
    exit 0
    ;;
  wait)
    echo "0"
    exit 0
    ;;
  logs)
    echo "fake logs"
    exit 0
    ;;
  stats)
    echo "10MiB / 100MiB"
    exit 0
    ;;
  rm)
    exit 0
    ;;
  inspect)
    echo "exited"
    exit 0
    ;;
  *)
    exit 0
    ;;
 esac
"#;
    bijux_infra::write_bytes(&docker_path, script)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&docker_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&docker_path, perms)?;
    }
    Ok(bin_dir)
}

fn temp_inputs() -> Result<(tempfile::TempDir, PathBuf, PathBuf)> {
    let dir = bijux_infra::temp_dir("bijux")?;
    let r1 = dir.path().join("input_r1.fastq");
    let r2 = dir.path().join("input_r2.fastq");
    bijux_infra::write_bytes(&r1, "@r1\nACGT\n+\n!!!!\n")?;
    bijux_infra::write_bytes(&r2, "@r2\nTGCA\n+\n!!!!\n")?;
    Ok((dir, r1, r2))
}

fn test_image() -> ContainerImageRefV1 {
    ContainerImageRefV1 {
        image: "bijux/test:latest".to_string(),
        digest: None,
    }
}

fn dummy_tool(tool: &str, image: &ContainerImageRefV1) -> ToolExecutionSpecV1 {
    ToolExecutionSpecV1 {
        tool_id: ToolId(tool.to_string()),
        tool_version: "1.0.0".to_string(),
        image: image.clone(),
        command: CommandSpecV1 {
            template: Vec::new(),
        },
        resources: ToolConstraints {
            runtime: "docker".to_string(),
            mem_gb: 1,
            tmp_gb: 1,
            threads: 1,
        },
    }
}

fn touch(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        bijux_infra::ensure_dir(parent)?;
    }
    if path.extension().is_none() {
        bijux_infra::ensure_dir(path)?;
        return Ok(());
    }
    bijux_infra::write_bytes(path, "")?;
    Ok(())
}

fn merge_outputs_for(tool: &str, out_dir: &Path) -> Vec<PathBuf> {
    match tool {
        "pear" => vec![
            out_dir.join("pear.assembled.fastq"),
            out_dir.join("pear.unassembled.forward.fastq"),
            out_dir.join("pear.unassembled.reverse.fastq"),
        ],
        "vsearch" => vec![
            out_dir.join("vsearch.merged.fastq"),
            out_dir.join("vsearch.unmerged_r1.fastq"),
            out_dir.join("vsearch.unmerged_r2.fastq"),
        ],
        "bbmerge" => vec![
            out_dir.join("bbmerge.merged.fastq"),
            out_dir.join("bbmerge.unmerged_r1.fastq"),
            out_dir.join("bbmerge.unmerged_r2.fastq"),
        ],
        _ => Vec::new(),
    }
}

fn stage_version_i32(version: bijux_core::StageVersion) -> i32 {
    i32::try_from(version.0).unwrap_or(i32::MAX)
}

#[allow(clippy::too_many_lines)]
fn build_plan(
    stage_id: &str,
    r1: &Path,
    r2: &Path,
    out_dir: &Path,
    image: &ContainerImageRefV1,
) -> Result<(StagePlanV1, Vec<PathBuf>, bool)> {
    let plan = match stage_id {
        "fastq.trim" => {
            let adapter_bank = serde_json::json!({
                "bank_id": "bank",
                "bank_version": "1",
                "bank_hash": "hash",
                "presets_hash": "preset_hashes",
                "preset": "ancientdna-illumina",
                "preset_hash": "preset_hash",
                "enabled_categories": ["truseq"],
                "disabled_categories": ["custom"],
                "enable_adapters": [],
                "disable_adapters": [],
            });
            trim::plan(
                &dummy_tool("fastp", image),
                r1,
                out_dir,
                Some(&adapter_bank),
                None,
                None,
            )?
        }
        "fastq.detect_adapters" => {
            bijux_stages_fastq::fastq::detect_adapters::plan(&dummy_tool("fastqc", image), r1, out_dir)
        }
        "fastq.filter" => {
            let options = bijux_stages_fastq::fastq::filter::FilterPlanOptions::default();
            filter::plan_filter(&dummy_tool("fastp", image), r1, out_dir, &options)?
        }
        "fastq.validate_pre" => {
            validate_pre::plan(&dummy_tool("fastqvalidator_official", image), r1, out_dir)
        }
        "fastq.merge" => merge::plan_merge(&dummy_tool("pear", image), r1, r2, out_dir)?,
        "fastq.correct" => {
            correct::plan_correct(&dummy_tool("rcorrector", image), r1, r2, out_dir)?
        }
        "fastq.umi" => umi::plan_umi(&dummy_tool("umi_tools", image), r1, r2, out_dir)?,
        "fastq.screen" => screen::plan_screen(&dummy_tool("kraken2", image), r1, out_dir)?,
        "fastq.qc_post" => {
            let mut aux_images = std::collections::BTreeMap::new();
            aux_images.insert("fastqc".to_string(), image.clone());
            qc_post::plan_qc_post(&dummy_tool("multiqc", image), r1, out_dir, aux_images, None)?
        }
        "fastq.stats_neutral" => {
            stats_neutral::plan_stats_neutral(&dummy_tool("seqkit", image), r1, out_dir)?
        }
        _ => return Err(anyhow::anyhow!("unsupported stage {stage_id}")),
    };

    let outputs: Vec<PathBuf> = plan.io.outputs.iter().map(|o| o.path.clone()).collect();
    let is_retention = matches!(
        stage_id,
        "fastq.trim" | "fastq.filter" | "fastq.merge" | "fastq.correct" | "fastq.umi"
    );

    Ok((plan, outputs, is_retention))
}
