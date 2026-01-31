use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use anyhow::Result;
use bijux_core::{
    CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1, ToolId,
};
use bijux_engine::api::execute_plan;
use bijux_env_runtime::api::RunnerKind;
use bijux_stages_fastq::fastq::{filter, merge, trim, validate_pre};
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
    echo "${BIJUX_TEST_DOCKER_EXIT_CODE:-0}"
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

fn shape_value(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Null => serde_json::Value::String("null".to_string()),
        serde_json::Value::Bool(_) => serde_json::Value::String("bool".to_string()),
        serde_json::Value::Number(_) => serde_json::Value::String("number".to_string()),
        serde_json::Value::String(_) => serde_json::Value::String("string".to_string()),
        serde_json::Value::Array(items) => {
            if let Some(first) = items.first() {
                serde_json::Value::Array(vec![shape_value(first)])
            } else {
                serde_json::Value::Array(Vec::new())
            }
        }
        serde_json::Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            let mut ordered = serde_json::Map::new();
            for key in keys {
                ordered.insert(key.clone(), shape_value(&map[key]));
            }
            serde_json::Value::Object(ordered)
        }
    }
}

fn assert_metrics_shape(stage: &str, metrics: &serde_json::Value) -> Result<()> {
    let shape = shape_value(metrics);
    let rendered = serde_json::to_string_pretty(&shape)?;
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let snapshot_path = Path::new(manifest_dir)
        .join("tests")
        .join("snapshots")
        .join(format!("{stage}_metrics_shape.json"));
    let snapshot = fs::read_to_string(&snapshot_path)?;
    assert_eq!(rendered.trim(), snapshot.trim());
    Ok(())
}

fn assert_report_shape(stage: &str, report: &serde_json::Value) -> Result<()> {
    let shape = shape_value(report);
    let rendered = serde_json::to_string_pretty(&shape)?;
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let snapshot_path = Path::new(manifest_dir)
        .join("tests")
        .join("snapshots")
        .join(format!("{stage}_stage_report_shape.json"));
    let snapshot = fs::read_to_string(&snapshot_path)?;
    assert_eq!(rendered.trim(), snapshot.trim());
    Ok(())
}

#[test]
fn metrics_shape_snapshots() -> Result<()> {
    let _guard = ENV_LOCK
        .lock()
        .map_err(|_| anyhow::anyhow!("env lock poisoned"))?;
    let (dir, r1, r2) = temp_inputs()?;
    let bin_dir = write_fake_docker(dir.path())?;
    let original_path = std::env::var("PATH").unwrap_or_default();
    std::env::set_var("PATH", format!("{}:{}", bin_dir.display(), original_path));

    let image = test_image();

    let trim_out = dir.path().join("trim");
    fs::create_dir_all(&trim_out)?;
    let trim_plan = trim::plan(
        &dummy_tool("fastp", &image),
        &r1,
        &trim_out,
        None,
        None,
        None,
    )?;
    for output in &trim_plan.io.outputs {
        touch(&output.path)?;
    }
    let _ = execute_plan(&trim_plan, RunnerKind::Docker, None)?;
    let trim_metrics_raw =
        fs::read_to_string(trim_out.join("run_artifacts").join("stage_metrics.json"))?;
    let trim_metrics: serde_json::Value = serde_json::from_str(&trim_metrics_raw)?;
    assert_metrics_shape("fastq_trim", &trim_metrics["metrics"])?;
    let trim_report_raw =
        fs::read_to_string(trim_out.join("run_artifacts").join("stage_report.json"))?;
    let trim_report: serde_json::Value = serde_json::from_str(&trim_report_raw)?;
    assert_report_shape("fastq_trim", &trim_report)?;

    let filter_out = dir.path().join("filter");
    fs::create_dir_all(&filter_out)?;
    let filter_options = bijux_stages_fastq::fastq::filter::FilterPlanOptions::default();
    let filter_plan = filter::plan_filter(
        &dummy_tool("fastp", &image),
        &r1,
        &filter_out,
        &filter_options,
    )?;
    for output in &filter_plan.io.outputs {
        touch(&output.path)?;
    }
    let _ = execute_plan(&filter_plan, RunnerKind::Docker, None)?;
    let filter_metrics_raw =
        fs::read_to_string(filter_out.join("run_artifacts").join("stage_metrics.json"))?;
    let filter_metrics: serde_json::Value = serde_json::from_str(&filter_metrics_raw)?;
    assert_metrics_shape("fastq_filter", &filter_metrics["metrics"])?;
    let filter_report_raw =
        fs::read_to_string(filter_out.join("run_artifacts").join("stage_report.json"))?;
    let filter_report: serde_json::Value = serde_json::from_str(&filter_report_raw)?;
    assert_report_shape("fastq_filter", &filter_report)?;

    let merge_out = dir.path().join("merge");
    fs::create_dir_all(&merge_out)?;
    let merge_plan = merge::plan_merge(&dummy_tool("pear", &image), &r1, &r2, &merge_out)?;
    for output in &merge_plan.io.outputs {
        touch(&output.path)?;
    }
    for output in merge_outputs_for(&merge_plan.tool_id.0, &merge_out) {
        touch(&output)?;
    }
    let _ = execute_plan(&merge_plan, RunnerKind::Docker, None)?;
    let merge_metrics_raw =
        fs::read_to_string(merge_out.join("run_artifacts").join("stage_metrics.json"))?;
    let merge_metrics: serde_json::Value = serde_json::from_str(&merge_metrics_raw)?;
    assert_metrics_shape("fastq_merge", &merge_metrics["metrics"])?;
    let merge_report_raw =
        fs::read_to_string(merge_out.join("run_artifacts").join("stage_report.json"))?;
    let merge_report: serde_json::Value = serde_json::from_str(&merge_report_raw)?;
    assert_report_shape("fastq_merge", &merge_report)?;

    let validate_out = dir.path().join("validate");
    fs::create_dir_all(&validate_out)?;
    let validate_plan = validate_pre::plan(
        &dummy_tool("fastqvalidator_official", &image),
        &r1,
        &validate_out,
    );
    for output in &validate_plan.io.outputs {
        touch(&output.path)?;
    }
    let _ = execute_plan(&validate_plan, RunnerKind::Docker, None)?;
    let validate_metrics_raw = fs::read_to_string(
        validate_out
            .join("run_artifacts")
            .join("stage_metrics.json"),
    )?;
    let validate_metrics: serde_json::Value = serde_json::from_str(&validate_metrics_raw)?;
    assert_metrics_shape("fastq_validate_pre", &validate_metrics["metrics"])?;
    let validate_report_raw =
        fs::read_to_string(validate_out.join("run_artifacts").join("stage_report.json"))?;
    let validate_report: serde_json::Value = serde_json::from_str(&validate_report_raw)?;
    assert_report_shape("fastq_validate_pre", &validate_report)?;

    std::env::set_var("PATH", original_path);
    Ok(())
}
