use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use anyhow::{Context, Result};
use bijux_core::canonicalize_json_value;
use bijux_engine::api::{execute_plan, params_hash, StagePlan};
use bijux_environment::api::{ResolvedImage, RunnerKind};
use bijux_stages_fastq::fastq::{
    correct, filter, merge, qc_post, screen, stats_neutral, trim, umi, validate_pre,
};
use bijux_stages_fastq::StagePlanJson;
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

fn test_image() -> ResolvedImage {
    ResolvedImage {
        full_name: "bijux/test:latest".to_string(),
        arch: "amd64".to_string(),
        runner: RunnerKind::Docker,
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

fn stage_version_i32(version: bijux_core::StageVersion) -> i32 {
    i32::try_from(version.0).unwrap_or(i32::MAX)
}

#[allow(clippy::too_many_lines)]
fn build_plan(
    stage_id: &str,
    r1: &Path,
    r2: &Path,
    out_dir: &Path,
    image: &ResolvedImage,
) -> Result<(StagePlan, Vec<PathBuf>, bool)> {
    let mut aux_images = HashMap::new();
    let (tool, stage_version, inputs, outputs, params) = match stage_id {
        "fastq.trim" => {
            let plan = trim::plan("fastp", r1, out_dir)?;
            let plan_json = StagePlanJson::from_plan(&plan);
            (
                "fastp".to_string(),
                stage_version_i32(trim::STAGE_VERSION),
                vec![r1.to_path_buf()],
                vec![plan.output.clone()],
                plan_json.parameters,
            )
        }
        "fastq.filter" => {
            let plan = filter::plan_filter("fastp", r1, out_dir)?;
            let plan_json = StagePlanJson::from_plan(&plan);
            (
                "fastp".to_string(),
                stage_version_i32(filter::STAGE_VERSION),
                vec![r1.to_path_buf()],
                vec![plan.output.clone()],
                plan_json.parameters,
            )
        }
        "fastq.validate_pre" => {
            let plan = validate_pre::plan("fastqvalidator_official", r1, out_dir);
            let plan_json = StagePlanJson::from_plan(&plan);
            (
                "fastqvalidator_official".to_string(),
                stage_version_i32(validate_pre::STAGE_VERSION),
                vec![r1.to_path_buf()],
                Vec::new(),
                plan_json.parameters,
            )
        }
        "fastq.merge" => {
            let plan = merge::plan_merge("pear", r1, r2, out_dir)?;
            let plan_json = StagePlanJson::from_plan(&plan);
            (
                "pear".to_string(),
                stage_version_i32(merge::STAGE_VERSION),
                vec![r1.to_path_buf(), r2.to_path_buf()],
                merge_outputs_for("pear", out_dir),
                plan_json.parameters,
            )
        }
        "fastq.correct" => {
            let plan = correct::plan_correct("rcorrector", r1, r2, out_dir)?;
            let plan_json = StagePlanJson::from_plan(&plan);
            (
                "rcorrector".to_string(),
                stage_version_i32(correct::STAGE_VERSION),
                vec![r1.to_path_buf(), r2.to_path_buf()],
                vec![plan.output_r1.clone(), plan.output_r2.clone()],
                plan_json.parameters,
            )
        }
        "fastq.umi" => {
            let plan = umi::plan_umi("umi_tools", r1, r2, out_dir)?;
            let plan_json = StagePlanJson::from_plan(&plan);
            (
                "umi_tools".to_string(),
                stage_version_i32(umi::STAGE_VERSION),
                vec![r1.to_path_buf(), r2.to_path_buf()],
                vec![plan.output_r1.clone(), plan.output_r2.clone()],
                plan_json.parameters,
            )
        }
        "fastq.screen" => {
            let plan = screen::plan_screen("kraken2", r1, out_dir)?;
            let plan_json = StagePlanJson::from_plan(&plan);
            (
                "kraken2".to_string(),
                stage_version_i32(screen::STAGE_VERSION),
                vec![r1.to_path_buf()],
                vec![plan.report.clone()],
                plan_json.parameters,
            )
        }
        "fastq.qc_post" => {
            let plan = qc_post::plan_qc_post("multiqc", r1, out_dir)?;
            let plan_json = StagePlanJson::from_plan(&plan);
            aux_images.insert("fastqc".to_string(), image.clone());
            (
                "multiqc".to_string(),
                stage_version_i32(qc_post::STAGE_VERSION),
                vec![r1.to_path_buf()],
                Vec::new(),
                plan_json.parameters,
            )
        }
        "fastq.stats_neutral" => {
            let plan = stats_neutral::plan_stats_neutral("seqkit", r1, out_dir)?;
            let plan_json = StagePlanJson::from_plan(&plan);
            (
                "seqkit".to_string(),
                stage_version_i32(stats_neutral::STAGE_VERSION),
                vec![r1.to_path_buf()],
                Vec::new(),
                plan_json.parameters,
            )
        }
        _ => return Err(anyhow::anyhow!("unsupported stage {stage_id}")),
    };

    let exec_plan = StagePlan {
        stage_id: stage_id.to_string(),
        stage_version,
        tool: tool.clone(),
        tool_version: "1.0.0".to_string(),
        image: image.clone(),
        runner: RunnerKind::Docker,
        inputs,
        out_dir: out_dir.to_path_buf(),
        outputs: outputs.clone(),
        params,
        aux_images,
    };

    let is_retention = matches!(
        stage_id,
        "fastq.trim" | "fastq.filter" | "fastq.merge" | "fastq.correct" | "fastq.umi"
    );

    Ok((exec_plan, outputs, is_retention))
}

#[test]
fn fastq_stages_emit_observability_contracts() -> Result<()> {
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
        fs::create_dir_all(&out_dir).context("create out dir")?;
        let (exec_plan, outputs, is_retention) = build_plan(stage.id, &r1, &r2, &out_dir, &image)?;
        for output in &outputs {
            touch(output).context("touch output")?;
        }
        if stage.id == "fastq.merge" {
            for output in merge_outputs_for(&exec_plan.tool, &out_dir) {
                touch(&output).context("touch merge output")?;
            }
        }

        let _result = execute_plan(&exec_plan)?;
        let run_artifacts = out_dir.join("run_artifacts");
        assert!(run_artifacts.join("plan.json").exists());
        assert!(run_artifacts.join("effective_config.json").exists());
        assert!(run_artifacts.join("metrics_envelope.json").exists());
        assert!(run_artifacts.join("stage_report.json").exists());
        let stage_config = run_artifacts
            .join("config")
            .join(format!("{}.effective.json", stage.id));
        assert!(stage_config.exists());
        if stage.affects_read_counts || is_retention {
            assert!(run_artifacts.join("retention_report.json").exists());
        }
        let manifest_path = run_artifacts.join("observability_manifest.json");
        assert!(manifest_path.exists());
        let manifest_raw = fs::read_to_string(&manifest_path)?;
        let manifest: serde_json::Value = serde_json::from_str(&manifest_raw)?;
        let artifacts = manifest["artifacts"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("manifest artifacts missing"))?;
        let mut names = std::collections::BTreeSet::new();
        for artifact in artifacts {
            if let Some(name) = artifact["name"].as_str() {
                names.insert(name.to_string());
            }
        }
        assert!(names.contains("plan"));
        assert!(names.contains("effective_config"));
        assert!(names.contains("stage_config"));
        assert!(names.contains("metrics_envelope"));
        assert!(names.contains("stage_report"));
        if stage.affects_read_counts {
            assert!(names.contains("retention_report"));
        }

        let envelope_path = run_artifacts.join("metrics_envelope.json");
        let envelope_raw = fs::read_to_string(&envelope_path).context("read metrics envelope")?;
        let envelope: serde_json::Value = serde_json::from_str(&envelope_raw)?;
        assert_eq!(envelope["stage_id"], stage.id);
        assert_eq!(
            envelope["stage_version"],
            serde_json::Value::from(stage_version_i32(stage.version))
        );
        assert_eq!(envelope["tool_id"], exec_plan.tool);
        assert_eq!(envelope["tool_version"], exec_plan.tool_version);
        assert!(
            envelope["input_hash"].as_str().is_some(),
            "input_hash missing"
        );
        let canonical_params = canonicalize_json_value(&exec_plan.params);
        let expected_hash = params_hash(&canonical_params)?;
        assert_eq!(envelope["params_hash"], expected_hash);
        assert_eq!(envelope["parameters_json"], canonical_params);

        let telemetry_path = run_artifacts.join("telemetry").join("events.jsonl");
        assert!(telemetry_path.exists());
        let telemetry_raw = fs::read_to_string(&telemetry_path)?;
        let mut has_stage_start = false;
        let mut has_stage_end = false;
        for line in telemetry_raw.lines() {
            if line.trim().is_empty() {
                continue;
            }
            let event: bijux_core::TelemetryEventV1 = serde_json::from_str(line)?;
            if event.event_name == "stage_start" {
                has_stage_start = true;
            }
            if event.event_name == "stage_end" {
                has_stage_end = true;
            }
        }
        assert!(has_stage_start, "missing stage_start telemetry event");
        assert!(has_stage_end, "missing stage_end telemetry event");
    }

    std::env::set_var("PATH", original_path);
    Ok(())
}
