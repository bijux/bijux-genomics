use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use crate::commands::benchmark::local_stage_commands::render_local_stage_commands;
use crate::commands::benchmark::local_stage_fake_runs::{
    path_relative_to_repo, stage_fake_run_manifest_path, DEFAULT_LOCAL_STAGE_FAKE_RUN_ROOT,
};
use crate::commands::benchmark::local_stage_inventory::LocalStageReadinessKind;
use crate::commands::benchmark::local_stage_result_manifest::{
    load_validated_stage_result_manifest_path, BenchStageResultStatus,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

const DEFAULT_RENDERED_STAGE_COMMANDS_PATH: &str = "target/local-ready/rendered-stage-commands.sh";
pub(crate) const DEFAULT_RUNTIME_METRICS_REPORT_PATH: &str =
    "target/local-ready/runtime-metrics.json";
const LOCAL_STAGE_RUNTIME_METRICS_REPORT_SCHEMA_VERSION: &str =
    "bijux.bench.local_runtime_metrics.v1";

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BenchLocalStageRuntimeMetricEntry {
    pub(crate) stage_id: String,
    pub(crate) readiness_kind: LocalStageReadinessKind,
    pub(crate) tool_id: String,
    pub(crate) manifest_path: String,
    pub(crate) runtime_mode: String,
    pub(crate) started_at: String,
    pub(crate) finished_at: String,
    pub(crate) elapsed_seconds: f64,
    pub(crate) exit_code: i32,
    pub(crate) status: BenchStageResultStatus,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BenchLocalStageRuntimeMetricsReport {
    pub(crate) schema_version: &'static str,
    pub(crate) fake_run_root: String,
    pub(crate) report_output_path: String,
    pub(crate) source_stage_command_manifest_path: String,
    pub(crate) stage_count: usize,
    pub(crate) stages: Vec<BenchLocalStageRuntimeMetricEntry>,
}

pub(crate) fn run_collect_runtime_metrics(
    args: &parse::BenchLocalCollectRuntimeMetricsArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = collect_local_stage_runtime_metrics(
        &repo_root,
        args.fake_run_root
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_STAGE_FAKE_RUN_ROOT)),
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_RUNTIME_METRICS_REPORT_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.report_output_path);
    }
    Ok(())
}

pub(crate) fn collect_local_stage_runtime_metrics(
    repo_root: &Path,
    fake_run_root: PathBuf,
    report_output_path: PathBuf,
) -> Result<BenchLocalStageRuntimeMetricsReport> {
    let source_manifest = render_local_stage_commands(
        repo_root,
        PathBuf::from(DEFAULT_RENDERED_STAGE_COMMANDS_PATH),
    )?;
    let absolute_fake_run_root =
        if fake_run_root.is_absolute() { fake_run_root } else { repo_root.join(&fake_run_root) };
    let absolute_report_output_path = if report_output_path.is_absolute() {
        report_output_path
    } else {
        repo_root.join(&report_output_path)
    };
    if let Some(parent) = absolute_report_output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let mut stages = Vec::with_capacity(source_manifest.commands.len());
    for command in &source_manifest.commands {
        let stage_root = absolute_fake_run_root.join(&command.stage_id);
        let stage_manifest_path = stage_fake_run_manifest_path(&stage_root);
        let manifest =
            load_validated_stage_result_manifest_path(&stage_manifest_path).map_err(|err| {
                anyhow!("collect runtime metrics from {}: {err:#}", stage_manifest_path.display())
            })?;
        stages.push(BenchLocalStageRuntimeMetricEntry {
            stage_id: command.stage_id.clone(),
            readiness_kind: command.readiness_kind,
            tool_id: command.tool_id.clone(),
            manifest_path: path_relative_to_repo(repo_root, &stage_manifest_path),
            runtime_mode: manifest.runtime.mode,
            started_at: manifest.runtime.started_at,
            finished_at: manifest.runtime.finished_at,
            elapsed_seconds: manifest.runtime.elapsed_seconds,
            exit_code: manifest.runtime.exit_code,
            status: manifest.runtime.status,
        });
    }

    let report = BenchLocalStageRuntimeMetricsReport {
        schema_version: LOCAL_STAGE_RUNTIME_METRICS_REPORT_SCHEMA_VERSION,
        fake_run_root: path_relative_to_repo(repo_root, &absolute_fake_run_root),
        report_output_path: path_relative_to_repo(repo_root, &absolute_report_output_path),
        source_stage_command_manifest_path: source_manifest.manifest_output_path,
        stage_count: stages.len(),
        stages,
    };
    bijux_dna_infra::atomic_write_json(&absolute_report_output_path, &report)?;
    Ok(report)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    #[cfg(feature = "bam_downstream")]
    use super::{
        collect_local_stage_runtime_metrics, DEFAULT_RUNTIME_METRICS_REPORT_PATH,
        LOCAL_STAGE_RUNTIME_METRICS_REPORT_SCHEMA_VERSION,
    };
    #[cfg(feature = "bam_downstream")]
    use crate::commands::benchmark::local_stage_fake_runs::fake_run_local_stage_commands;
    #[cfg(feature = "bam_downstream")]
    use std::fs;

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[cfg(feature = "bam_downstream")]
    #[test]
    fn runtime_metrics_report_governed_51_stage_slice_from_fake_run_manifests() {
        let root = repo_root();
        let fake_run_root = PathBuf::from("target/local-fake-runs/stages-runtime-metrics");
        fake_run_local_stage_commands(&root, fake_run_root.clone())
            .expect("fake-run local stage commands");
        let report = collect_local_stage_runtime_metrics(
            &root,
            fake_run_root,
            PathBuf::from(DEFAULT_RUNTIME_METRICS_REPORT_PATH),
        )
        .expect("collect local stage runtime metrics");

        assert_eq!(report.schema_version, LOCAL_STAGE_RUNTIME_METRICS_REPORT_SCHEMA_VERSION);
        assert_eq!(report.stage_count, 51);
        assert_eq!(report.stages.len(), 51);
        assert!(report.stages.iter().all(|stage| {
            stage.runtime_mode == "fake_run"
                && !stage.started_at.is_empty()
                && !stage.finished_at.is_empty()
                && stage.elapsed_seconds >= 0.0
                && stage.exit_code == 0
        }));
    }

    #[cfg(feature = "bam_downstream")]
    #[test]
    fn runtime_metrics_report_refuses_stage_manifest_missing_runtime_fields() {
        let root = repo_root();
        let fake_run_root =
            PathBuf::from("target/local-fake-runs/stages-runtime-metrics-missing-runtime");
        let report_output_path = PathBuf::from("target/local-ready/runtime-metrics.missing.json");
        let fake_runs = fake_run_local_stage_commands(&root, fake_run_root.clone())
            .expect("fake-run local stage commands");
        let stage_manifest_path = root.join(
            fake_runs
                .stages
                .iter()
                .find(|stage| stage.stage_id == "fastq.report_qc")
                .expect("fastq.report_qc stage")
                .stage_manifest_path
                .clone(),
        );
        let mut payload: serde_json::Value =
            serde_json::from_slice(&fs::read(&stage_manifest_path).expect("read stage manifest"))
                .expect("parse stage manifest");
        payload.as_object_mut().expect("stage manifest object").remove("runtime");
        fs::write(
            &stage_manifest_path,
            serde_json::to_vec_pretty(&payload).expect("serialize broken stage manifest"),
        )
        .expect("write broken stage manifest");

        let error = collect_local_stage_runtime_metrics(&root, fake_run_root, report_output_path)
            .expect_err("runtime metrics should reject missing runtime fields");
        assert!(
            error.to_string().contains("missing field `runtime`"),
            "runtime-metrics failure should identify missing runtime field: {error:#}"
        );
    }
}
