use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Serialize;

use crate::commands::benchmark::local_stage_commands::render_local_stage_commands;
use crate::commands::benchmark::local_stage_fake_runs::{
    path_relative_to_repo, stage_fake_run_manifest_path, stage_fake_run_output_path,
    DEFAULT_LOCAL_STAGE_FAKE_RUN_ROOT,
};
use crate::commands::benchmark::local_stage_inventory::LocalStageReadinessKind;
use crate::commands::cli::parse;
use crate::commands::cli::render;

const DEFAULT_RENDERED_STAGE_COMMANDS_PATH: &str = "target/local-ready/rendered-stage-commands.sh";
const DEFAULT_MANIFEST_COMPLETION_REPORT_PATH: &str =
    "target/local-ready/manifest-completion-report.json";
const LOCAL_STAGE_MANIFEST_COMPLETION_REPORT_SCHEMA_VERSION: &str =
    "bijux.bench.local_stage_manifest_completion.v1";

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BenchLocalStageManifestCompletionEntry {
    pub(crate) stage_id: String,
    pub(crate) readiness_kind: LocalStageReadinessKind,
    pub(crate) tool_id: String,
    pub(crate) stage_manifest_path: String,
    pub(crate) manifest_exists: bool,
    pub(crate) declared_output_count: usize,
    pub(crate) present_output_count: usize,
    pub(crate) complete: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BenchLocalStageManifestCompletionReport {
    pub(crate) schema_version: &'static str,
    pub(crate) fake_run_root: String,
    pub(crate) report_output_path: String,
    pub(crate) source_stage_command_manifest_path: String,
    pub(crate) stage_count: usize,
    pub(crate) complete_stage_count: usize,
    pub(crate) incomplete_stage_count: usize,
    pub(crate) complete: bool,
    pub(crate) stages: Vec<BenchLocalStageManifestCompletionEntry>,
}

pub(crate) fn run_check_manifest_completion(
    args: &parse::BenchLocalCheckManifestCompletionArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = check_local_stage_manifest_completion(
        &repo_root,
        args.fake_run_root
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_STAGE_FAKE_RUN_ROOT)),
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_MANIFEST_COMPLETION_REPORT_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.report_output_path);
    }
    Ok(())
}

pub(crate) fn check_local_stage_manifest_completion(
    repo_root: &Path,
    fake_run_root: PathBuf,
    report_output_path: PathBuf,
) -> Result<BenchLocalStageManifestCompletionReport> {
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
        let declared_output_count = command.outputs.len();
        let present_output_count = command
            .outputs
            .iter()
            .filter(|artifact| stage_fake_run_output_path(&stage_root, &artifact.path).exists())
            .count();
        let manifest_exists = stage_manifest_path.is_file();
        stages.push(BenchLocalStageManifestCompletionEntry {
            stage_id: command.stage_id.clone(),
            readiness_kind: command.readiness_kind,
            tool_id: command.tool_id.clone(),
            stage_manifest_path: path_relative_to_repo(repo_root, &stage_manifest_path),
            manifest_exists,
            declared_output_count,
            present_output_count,
            complete: manifest_exists,
        });
    }

    let complete_stage_count = stages.iter().filter(|stage| stage.complete).count();
    let incomplete_stage_count = stages.len().saturating_sub(complete_stage_count);
    let report = BenchLocalStageManifestCompletionReport {
        schema_version: LOCAL_STAGE_MANIFEST_COMPLETION_REPORT_SCHEMA_VERSION,
        fake_run_root: path_relative_to_repo(repo_root, &absolute_fake_run_root),
        report_output_path: path_relative_to_repo(repo_root, &absolute_report_output_path),
        source_stage_command_manifest_path: source_manifest.manifest_output_path,
        stage_count: stages.len(),
        complete_stage_count,
        incomplete_stage_count,
        complete: incomplete_stage_count == 0,
        stages,
    };
    bijux_dna_infra::atomic_write_json(&absolute_report_output_path, &report)?;
    Ok(report)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use super::{
        check_local_stage_manifest_completion, DEFAULT_MANIFEST_COMPLETION_REPORT_PATH,
        LOCAL_STAGE_MANIFEST_COMPLETION_REPORT_SCHEMA_VERSION,
    };
    use crate::commands::benchmark::local_stage_fake_runs::{
        fake_run_local_stage_commands, DEFAULT_LOCAL_STAGE_FAKE_RUN_ROOT,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[cfg(feature = "bam_downstream")]
    #[test]
    fn manifest_completion_reports_governed_51_stage_slice_complete_when_manifests_exist() {
        let root = repo_root();
        fake_run_local_stage_commands(&root, PathBuf::from(DEFAULT_LOCAL_STAGE_FAKE_RUN_ROOT))
            .expect("fake-run local stage commands");
        let report = check_local_stage_manifest_completion(
            &root,
            PathBuf::from(DEFAULT_LOCAL_STAGE_FAKE_RUN_ROOT),
            PathBuf::from(DEFAULT_MANIFEST_COMPLETION_REPORT_PATH),
        )
        .expect("check local stage manifest completion");

        assert_eq!(report.schema_version, LOCAL_STAGE_MANIFEST_COMPLETION_REPORT_SCHEMA_VERSION);
        assert_eq!(report.stage_count, 51);
        assert_eq!(report.complete_stage_count, 51);
        assert_eq!(report.incomplete_stage_count, 0);
        assert!(report.complete);
        assert!(report.stages.iter().all(|stage| {
            stage.complete
                && stage.manifest_exists
                && stage.present_output_count == stage.declared_output_count
        }));
    }

    #[cfg(feature = "bam_downstream")]
    #[test]
    fn manifest_completion_reports_stage_incomplete_when_stage_manifest_is_missing() {
        let root = repo_root();
        let fake_run_root =
            PathBuf::from("target/local-fake-runs/stages-manifest-completion-missing");
        let report_output_path =
            PathBuf::from("target/local-ready/manifest-completion-report.missing.json");
        let fake_runs = fake_run_local_stage_commands(&root, fake_run_root.clone())
            .expect("fake-run local stage commands");
        let missing_manifest_path = root.join(
            fake_runs
                .stages
                .iter()
                .find(|stage| stage.stage_id == "fastq.report_qc")
                .expect("fastq.report_qc stage")
                .stage_manifest_path
                .clone(),
        );
        fs::remove_file(&missing_manifest_path).expect("remove fake stage manifest");

        let report =
            check_local_stage_manifest_completion(&root, fake_run_root, report_output_path)
                .expect("check local stage manifest completion");

        assert!(!report.complete);
        assert!(report.incomplete_stage_count >= 1);
        let stage = report
            .stages
            .iter()
            .find(|stage| stage.stage_id == "fastq.report_qc")
            .expect("fastq.report_qc manifest completion stage");
        assert!(!stage.complete);
        assert!(!stage.manifest_exists);
        assert_eq!(stage.present_output_count, stage.declared_output_count);
        assert!(stage.stage_manifest_path.ends_with("fastq.report_qc/stage-result.json"));
    }
}
