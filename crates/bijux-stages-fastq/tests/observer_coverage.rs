use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use anyhow::Result;
use bijux_core::{
    CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1, ToolId,
};
use bijux_environment::api::RunnerKind;
use bijux_exec::primitives::execute_stage_plan;
use bijux_stages_fastq::fastq::trim;

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

#[test]
fn observer_populates_key_metrics_fields() -> Result<()> {
    let _guard = ENV_LOCK
        .lock()
        .map_err(|_| anyhow::anyhow!("env lock poisoned"))?;
    let dir = bijux_infra::temp_dir("bijux")?;
    let bin_dir = write_fake_docker(dir.path())?;
    let original_path = std::env::var("PATH").unwrap_or_default();
    std::env::set_var("PATH", format!("{}:{}", bin_dir.display(), original_path));

    let r1 = dir.path().join("input.fastq");
    bijux_infra::write_bytes(&r1, "@r1\nACGT\n+\n!!!!\n")?;
    let out_dir = dir.path().join("out");
    bijux_infra::ensure_dir(&out_dir)?;
    let image = ContainerImageRefV1 {
        image: "bijux/test:latest".to_string(),
        digest: None,
    };
    let plan = trim::plan(
        &dummy_tool("fastp", &image),
        &r1,
        &out_dir,
        None,
        None,
        None,
    )?;
    for output in &plan.io.outputs {
        bijux_infra::write_bytes(&output.path, "")?;
    }
    let _ = execute_stage_plan(&plan, RunnerKind::Docker, None)?;

    let metrics_raw = fs::read_to_string(out_dir.join("run_artifacts").join("stage_metrics.json"))?;
    let metrics: serde_json::Value = serde_json::from_str(&metrics_raw)?;
    let metrics = metrics["metrics"]
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("metrics missing"))?;
    for key in [
        "reads_in",
        "reads_out",
        "bases_in",
        "bases_out",
        "mean_q_before",
        "mean_q_after",
    ] {
        assert!(
            metrics
                .get(key)
                .and_then(serde_json::Value::as_f64)
                .is_some()
                || metrics
                    .get(key)
                    .and_then(serde_json::Value::as_u64)
                    .is_some(),
            "missing numeric metric {key}"
        );
    }

    std::env::set_var("PATH", original_path);
    Ok(())
}
