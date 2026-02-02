use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use anyhow::Result;
use bijux_core::{
    CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1, ToolId,
};
use bijux_engine::api::execute_plan;
use bijux_env_runtime::api::RunnerKind;
use bijux_stages_fastq::fastq::{
    correct, filter, merge, qc_post, screen, stats_neutral, trim, umi, validate_pre,
};
use tempfile::TempDir;

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn write_fake_docker(dir: &Path) -> Result<PathBuf> {
    let bin_dir = dir.join("bin");
    fs::create_dir_all(&bin_dir)?;
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
    fs::write(&docker_path, script)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&docker_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&docker_path, perms)?;
    }
    Ok(bin_dir)
}

fn temp_inputs() -> Result<(TempDir, PathBuf, PathBuf)> {
    let dir = TempDir::new()?;
    let r1 = dir.path().join("input_r1.fastq");
    let r2 = dir.path().join("input_r2.fastq");
    fs::write(&r1, "@r1\nACGT\n+\n!!!!\n")?;
    fs::write(&r2, "@r2\nTGCA\n+\n!!!!\n")?;
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
        fs::create_dir_all(parent)?;
    }
    if path.extension().is_none() {
        fs::create_dir_all(path)?;
        return Ok(());
    }
    fs::write(path, "")?;
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

fn build_plan(
    stage_id: &str,
    r1: &Path,
    r2: &Path,
    out_dir: &Path,
    image: &ContainerImageRefV1,
) -> Result<(bijux_core::StagePlanV1, Vec<PathBuf>)> {
    let plan = match stage_id {
        "fastq.trim" => trim::plan(&dummy_tool("fastp", image), r1, out_dir, None, None, None)?,
        "fastq.filter" => {
            let options = bijux_stages_fastq::fastq::filter::FilterPlanOptions::default();
            filter::plan_filter(&dummy_tool("fastp", image), r1, out_dir, &options)?
        }
        "fastq.detect_adapters" => bijux_stages_fastq::fastq::detect_adapters::plan(
            &dummy_tool("fastqc", image),
            r1,
            out_dir,
        ),
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
    Ok((plan, outputs))
}

#[test]
#[allow(clippy::too_many_lines)]
fn metrics_completeness_contract() -> Result<()> {
    let _guard = ENV_LOCK
        .lock()
        .map_err(|_| anyhow::anyhow!("env lock poisoned"))?;
    let (dir, r1, r2) = temp_inputs()?;
    let bin_dir = write_fake_docker(dir.path())?;
    let original_path = std::env::var("PATH").unwrap_or_default();
    std::env::set_var("PATH", format!("{}:{}", bin_dir.display(), original_path));

    let image = test_image();
    for stage in bijux_stages_fastq::fastq::registry() {
        if stage.id == "fastq.preprocess" {
            continue;
        }
        let out_dir = dir.path().join(stage.id.replace('.', "_"));
        fs::create_dir_all(&out_dir)?;
        let (plan, outputs) = build_plan(stage.id, &r1, &r2, &out_dir, &image)?;
        for output in &outputs {
            touch(output)?;
        }
        if stage.id == "fastq.qc_post" {
            touch(&out_dir.join("fastqc_trimmed").join("fastqc_data.txt"))?;
            touch(&out_dir.join("multiqc_report.html"))?;
            fs::create_dir_all(out_dir.join("multiqc_data"))?;
        }
        if stage.id == "fastq.merge" {
            for output in merge_outputs_for(&plan.tool_id.0, &out_dir) {
                touch(&output)?;
            }
        }

        let _result = execute_plan(&plan, RunnerKind::Docker, None)?;
        let stage_metrics_path = out_dir.join("run_artifacts").join("stage_metrics.json");
        let stage_metrics_raw = fs::read_to_string(&stage_metrics_path)?;
        let stage_metrics: serde_json::Value = serde_json::from_str(&stage_metrics_raw)?;
        let execution = stage_metrics.get("execution").and_then(|v| v.as_object());
        assert!(execution.is_some());
        let metrics = stage_metrics["metrics"]
            .as_object()
            .ok_or_else(|| anyhow::anyhow!("metrics missing"))?;
        assert!(metrics.contains_key("reads_in"));
        assert!(metrics.contains_key("reads_out"));
        assert!(metrics.contains_key("bases_in"));
        assert!(metrics.contains_key("bases_out"));
        assert!(metrics.contains_key("pairs_in"));
        assert!(metrics.contains_key("pairs_out"));
        if stage.affects_read_counts && matches!(stage.id, "fastq.trim" | "fastq.filter") {
            let retention = metrics
                .get("retention")
                .and_then(|value| value.as_object())
                .ok_or_else(|| anyhow::anyhow!("retention missing"))?;
            assert!(retention.contains_key("value"));
            assert!(retention.contains_key("numerator_reads"));
            assert!(retention.contains_key("denominator_reads"));
            assert!(retention.contains_key("definition"));
            assert!(retention.contains_key("stage_boundary"));
            assert!(retention.contains_key("conditions"));
        }
    }

    std::env::set_var("PATH", original_path);
    Ok(())
}
