use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use crate::commands::benchmark::local_stage_commands::{
    render_local_stage_commands, BenchLocalStageArtifactEntry, BenchLocalStageCommandEntry,
};
use crate::commands::benchmark::local_stage_inventory::LocalStageReadinessKind;
use crate::commands::cli::parse;
use crate::commands::cli::render;

const LOCAL_STAGE_FAKE_RUN_MANIFEST_SCHEMA_VERSION: &str = "bijux.bench.local_stage_fake_runs.v1";
const LOCAL_STAGE_FAKE_RUN_RESULT_SCHEMA_VERSION: &str =
    "bijux.bench.local_stage_fake_run_result.v1";
const LOCAL_STAGE_FAKE_FAILURE_MANIFEST_SCHEMA_VERSION: &str =
    "bijux.bench.local_stage_fake_failures.v1";
const LOCAL_STAGE_FAKE_FAILURE_RECORD_SCHEMA_VERSION: &str =
    "bijux.bench.local_stage_fake_failure_record.v1";
const DEFAULT_LOCAL_STAGE_FAKE_RUN_ROOT: &str = "target/local-fake-runs/stages";
const DEFAULT_LOCAL_STAGE_FAKE_FAILURE_ROOT: &str = "target/local-fake-runs/failures";
const DEFAULT_RENDERED_STAGE_COMMANDS_PATH: &str = "target/local-ready/rendered-stage-commands.sh";

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BenchLocalStageFakeRunOutputEntry {
    pub(crate) artifact_id: String,
    pub(crate) declared_path: String,
    pub(crate) fake_run_path: String,
    pub(crate) role: String,
    pub(crate) optional: bool,
    pub(crate) exists: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BenchLocalStageFakeRunResult {
    pub(crate) schema_version: &'static str,
    pub(crate) stage_id: String,
    pub(crate) readiness_kind: LocalStageReadinessKind,
    pub(crate) tool_id: String,
    pub(crate) command: String,
    pub(crate) stage_manifest_path: String,
    pub(crate) declared_output_count: usize,
    pub(crate) created_output_count: usize,
    pub(crate) outputs: Vec<BenchLocalStageFakeRunOutputEntry>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BenchLocalStageFakeRunManifest {
    pub(crate) schema_version: &'static str,
    pub(crate) fake_run_root: String,
    pub(crate) source_stage_command_manifest_path: String,
    pub(crate) stage_count: usize,
    pub(crate) stages: Vec<BenchLocalStageFakeRunResult>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BenchLocalStageFakeFailureOutputEntry {
    pub(crate) artifact_id: String,
    pub(crate) declared_path: String,
    pub(crate) expected_fake_run_path: String,
    pub(crate) role: String,
    pub(crate) optional: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BenchLocalStageFakeFailureRecord {
    pub(crate) schema_version: &'static str,
    pub(crate) stage_id: String,
    pub(crate) readiness_kind: LocalStageReadinessKind,
    pub(crate) tool_id: String,
    pub(crate) command: String,
    pub(crate) exit_code: i32,
    pub(crate) stderr_path: String,
    pub(crate) failure_record_path: String,
    pub(crate) failed_output_count: usize,
    pub(crate) failed_outputs: Vec<BenchLocalStageFakeFailureOutputEntry>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BenchLocalStageFakeFailureManifest {
    pub(crate) schema_version: &'static str,
    pub(crate) failure_root: String,
    pub(crate) source_stage_command_manifest_path: String,
    pub(crate) stage_count: usize,
    pub(crate) failures: Vec<BenchLocalStageFakeFailureRecord>,
}

pub(crate) fn run_fake_run_stages(args: &parse::BenchLocalFakeRunStagesArgs) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let manifest = fake_run_local_stage_commands(
        &repo_root,
        args.output_root
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_STAGE_FAKE_RUN_ROOT)),
    )?;
    if args.json {
        render::json::print_pretty(&manifest)?;
    } else {
        println!("{}", manifest.fake_run_root);
    }
    Ok(())
}

pub(crate) fn run_fake_run_failures(args: &parse::BenchLocalFakeRunFailuresArgs) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let manifest = fake_run_local_stage_failures(
        &repo_root,
        args.output_root
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_STAGE_FAKE_FAILURE_ROOT)),
        &args.stage_ids,
        args.exit_code,
    )?;
    if args.json {
        render::json::print_pretty(&manifest)?;
    } else {
        println!("{}", manifest.failure_root);
    }
    Ok(())
}

pub(crate) fn fake_run_local_stage_commands(
    repo_root: &Path,
    output_root: PathBuf,
) -> Result<BenchLocalStageFakeRunManifest> {
    let source_manifest = render_local_stage_commands(
        repo_root,
        PathBuf::from(DEFAULT_RENDERED_STAGE_COMMANDS_PATH),
    )?;
    let absolute_output_root =
        if output_root.is_absolute() { output_root } else { repo_root.join(&output_root) };
    fs::create_dir_all(&absolute_output_root)
        .with_context(|| format!("create {}", absolute_output_root.display()))?;

    let mut stages = Vec::with_capacity(source_manifest.commands.len());
    for command in &source_manifest.commands {
        stages.push(fake_run_stage_command(repo_root, &absolute_output_root, command)?);
    }

    let manifest_path = absolute_output_root.join("manifest.json");
    let manifest = BenchLocalStageFakeRunManifest {
        schema_version: LOCAL_STAGE_FAKE_RUN_MANIFEST_SCHEMA_VERSION,
        fake_run_root: path_relative_to_repo(repo_root, &absolute_output_root),
        source_stage_command_manifest_path: source_manifest.manifest_output_path,
        stage_count: stages.len(),
        stages,
    };
    bijux_dna_infra::atomic_write_json(&manifest_path, &manifest)?;
    Ok(manifest)
}

pub(crate) fn fake_run_local_stage_failures(
    repo_root: &Path,
    output_root: PathBuf,
    stage_ids: &[String],
    exit_code: i32,
) -> Result<BenchLocalStageFakeFailureManifest> {
    let source_manifest = render_local_stage_commands(
        repo_root,
        PathBuf::from(DEFAULT_RENDERED_STAGE_COMMANDS_PATH),
    )?;
    let commands = select_stage_commands(&source_manifest.commands, stage_ids)?;
    let absolute_output_root =
        if output_root.is_absolute() { output_root } else { repo_root.join(&output_root) };
    fs::create_dir_all(&absolute_output_root)
        .with_context(|| format!("create {}", absolute_output_root.display()))?;

    let mut failures = Vec::with_capacity(commands.len());
    for command in commands {
        failures.push(fake_run_stage_failure(
            repo_root,
            &absolute_output_root,
            command,
            exit_code,
        )?);
    }

    let manifest_path = absolute_output_root.join("manifest.json");
    let manifest = BenchLocalStageFakeFailureManifest {
        schema_version: LOCAL_STAGE_FAKE_FAILURE_MANIFEST_SCHEMA_VERSION,
        failure_root: path_relative_to_repo(repo_root, &absolute_output_root),
        source_stage_command_manifest_path: source_manifest.manifest_output_path,
        stage_count: failures.len(),
        failures,
    };
    bijux_dna_infra::atomic_write_json(&manifest_path, &manifest)?;
    Ok(manifest)
}

fn fake_run_stage_command(
    repo_root: &Path,
    fake_run_root: &Path,
    command: &BenchLocalStageCommandEntry,
) -> Result<BenchLocalStageFakeRunResult> {
    let stage_root = fake_run_root.join(&command.stage_id);
    fs::create_dir_all(&stage_root).with_context(|| format!("create {}", stage_root.display()))?;

    let outputs = command
        .outputs
        .iter()
        .map(|artifact| fake_run_output_entry(repo_root, &stage_root, &command.stage_id, artifact))
        .collect::<Result<Vec<_>>>()?;
    let created_output_count = outputs.iter().filter(|artifact| artifact.exists).count();

    let stage_manifest_path = stage_root.join("stage-result.json");
    let result = BenchLocalStageFakeRunResult {
        schema_version: LOCAL_STAGE_FAKE_RUN_RESULT_SCHEMA_VERSION,
        stage_id: command.stage_id.clone(),
        readiness_kind: command.readiness_kind,
        tool_id: command.tool_id.clone(),
        command: command.command.clone(),
        stage_manifest_path: path_relative_to_repo(repo_root, &stage_manifest_path),
        declared_output_count: outputs.len(),
        created_output_count,
        outputs,
    };
    bijux_dna_infra::atomic_write_json(&stage_manifest_path, &result)?;
    Ok(result)
}

fn fake_run_stage_failure(
    repo_root: &Path,
    failure_root: &Path,
    command: &BenchLocalStageCommandEntry,
    exit_code: i32,
) -> Result<BenchLocalStageFakeFailureRecord> {
    let stage_root = failure_root.join(&command.stage_id);
    fs::create_dir_all(&stage_root).with_context(|| format!("create {}", stage_root.display()))?;

    let failed_outputs = command
        .outputs
        .iter()
        .map(|artifact| BenchLocalStageFakeFailureOutputEntry {
            artifact_id: artifact.artifact_id.clone(),
            declared_path: artifact.path.clone(),
            expected_fake_run_path: path_relative_to_repo(
                repo_root,
                &stage_root.join("declared-outputs").join(&artifact.path),
            ),
            role: artifact.role.clone(),
            optional: artifact.optional,
        })
        .collect::<Vec<_>>();

    let stderr_path = stage_root.join("stderr.txt");
    let failure_record_path = stage_root.join("failure.json");
    fs::write(
        &stderr_path,
        format!(
            "fake local benchmark failure\nstage_id={}\ntool_id={}\nexit_code={exit_code}\ncommand={}\n",
            command.stage_id, command.tool_id, command.command
        ),
    )
    .with_context(|| format!("write {}", stderr_path.display()))?;

    let record = BenchLocalStageFakeFailureRecord {
        schema_version: LOCAL_STAGE_FAKE_FAILURE_RECORD_SCHEMA_VERSION,
        stage_id: command.stage_id.clone(),
        readiness_kind: command.readiness_kind,
        tool_id: command.tool_id.clone(),
        command: command.command.clone(),
        exit_code,
        stderr_path: path_relative_to_repo(repo_root, &stderr_path),
        failure_record_path: path_relative_to_repo(repo_root, &failure_record_path),
        failed_output_count: failed_outputs.len(),
        failed_outputs,
    };
    bijux_dna_infra::atomic_write_json(&failure_record_path, &record)?;
    Ok(record)
}

fn fake_run_output_entry(
    repo_root: &Path,
    stage_root: &Path,
    stage_id: &str,
    artifact: &BenchLocalStageArtifactEntry,
) -> Result<BenchLocalStageFakeRunOutputEntry> {
    let fake_run_path = stage_root.join("declared-outputs").join(&artifact.path);
    materialize_fake_run_output(&fake_run_path, stage_id, artifact)?;
    Ok(BenchLocalStageFakeRunOutputEntry {
        artifact_id: artifact.artifact_id.clone(),
        declared_path: artifact.path.clone(),
        fake_run_path: path_relative_to_repo(repo_root, &fake_run_path),
        role: artifact.role.clone(),
        optional: artifact.optional,
        exists: fake_run_path.exists(),
    })
}

fn materialize_fake_run_output(
    path: &Path,
    stage_id: &str,
    artifact: &BenchLocalStageArtifactEntry,
) -> Result<()> {
    if output_path_is_directory(artifact, path) {
        fs::create_dir_all(path).with_context(|| format!("create {}", path.display()))?;
        let sentinel = path.join(".bijux-fake-run-placeholder");
        fs::write(
            &sentinel,
            format!(
                "fake-run directory placeholder\nstage_id={stage_id}\nartifact_id={}\nrole={}\n",
                artifact.artifact_id, artifact.role
            ),
        )
        .with_context(|| format!("write {}", sentinel.display()))?;
        return Ok(());
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(path, fake_output_bytes(stage_id, artifact, path)?)
        .with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

fn output_path_is_directory(artifact: &BenchLocalStageArtifactEntry, path: &Path) -> bool {
    artifact.artifact_id.ends_with("_dir")
        || path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| matches!(name, "multiqc_data" | "plots"))
}

fn fake_output_bytes(
    stage_id: &str,
    artifact: &BenchLocalStageArtifactEntry,
    path: &Path,
) -> Result<Vec<u8>> {
    if binary_output_extension(path) {
        return Ok(Vec::new());
    }
    if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
        return serde_json::to_vec_pretty(&serde_json::json!({
            "schema_version": "bijux.bench.fake_output.v1",
            "stage_id": stage_id,
            "artifact_id": artifact.artifact_id,
            "role": artifact.role,
            "declared_path": artifact.path,
        }))
        .context("serialize fake JSON output");
    }
    if path.extension().and_then(|ext| ext.to_str()) == Some("tsv") {
        return Ok(format!(
            "stage_id\tartifact_id\trole\tdeclared_path\n{stage_id}\t{}\t{}\t{}\n",
            artifact.artifact_id, artifact.role, artifact.path
        )
        .into_bytes());
    }
    if path.extension().and_then(|ext| ext.to_str()) == Some("html") {
        return Ok(format!(
            "<html><body><h1>fake local benchmark output</h1><p>{stage_id}</p><p>{}</p></body></html>\n",
            artifact.artifact_id
        )
        .into_bytes());
    }

    Ok(format!(
        "fake local benchmark output\nstage_id={stage_id}\nartifact_id={}\nrole={}\ndeclared_path={}\n",
        artifact.artifact_id, artifact.role, artifact.path
    )
    .into_bytes())
}

fn binary_output_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| matches!(ext, "bam" | "bai" | "bcf" | "gz" | "pdf" | "zip"))
}

fn select_stage_commands<'a>(
    commands: &'a [BenchLocalStageCommandEntry],
    stage_ids: &[String],
) -> Result<Vec<&'a BenchLocalStageCommandEntry>> {
    if stage_ids.is_empty() {
        return Ok(commands.iter().collect());
    }

    let selected = commands
        .iter()
        .filter(|command| stage_ids.iter().any(|stage_id| stage_id == &command.stage_id))
        .collect::<Vec<_>>();
    if selected.len() != stage_ids.len() {
        let known = commands.iter().map(|command| command.stage_id.as_str()).collect::<Vec<_>>();
        let missing = stage_ids
            .iter()
            .filter(|stage_id| !known.contains(&stage_id.as_str()))
            .cloned()
            .collect::<Vec<_>>();
        return Err(anyhow!(
            "unknown local benchmark stage id(s): {}; known stages: {}",
            missing.join(", "),
            known.join(", ")
        ));
    }
    Ok(selected)
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).display().to_string()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        fake_run_local_stage_commands, fake_run_local_stage_failures,
        DEFAULT_LOCAL_STAGE_FAKE_FAILURE_ROOT, DEFAULT_LOCAL_STAGE_FAKE_RUN_ROOT,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[cfg(feature = "bam_downstream")]
    #[test]
    fn fake_run_local_stage_commands_cover_governed_51_stage_slice() {
        let root = repo_root();
        let fake_runs =
            fake_run_local_stage_commands(&root, PathBuf::from(DEFAULT_LOCAL_STAGE_FAKE_RUN_ROOT))
                .expect("fake-run local stage commands");

        assert_eq!(fake_runs.schema_version, "bijux.bench.local_stage_fake_runs.v1");
        assert_eq!(fake_runs.fake_run_root, "target/local-fake-runs/stages");
        assert_eq!(fake_runs.stage_count, 51);
        assert_eq!(fake_runs.stages.len(), 51);
        assert!(fake_runs.stages.iter().all(|stage| {
            stage.declared_output_count >= 1
                && stage.created_output_count == stage.declared_output_count
                && root.join(&stage.stage_manifest_path).is_file()
                && stage
                    .outputs
                    .iter()
                    .all(|artifact| artifact.exists && root.join(&artifact.fake_run_path).exists())
        }));
    }

    #[cfg(feature = "bam_downstream")]
    #[test]
    fn fake_run_local_stage_failures_cover_governed_51_stage_slice() {
        let root = repo_root();
        let failures = fake_run_local_stage_failures(
            &root,
            PathBuf::from(DEFAULT_LOCAL_STAGE_FAKE_FAILURE_ROOT),
            &[],
            1,
        )
        .expect("fake-run local stage failures");

        assert_eq!(failures.schema_version, "bijux.bench.local_stage_fake_failures.v1");
        assert_eq!(failures.failure_root, "target/local-fake-runs/failures");
        assert_eq!(failures.stage_count, 51);
        assert_eq!(failures.failures.len(), 51);
        assert!(failures.failures.iter().all(|failure| {
            failure.exit_code == 1
                && failure.failed_output_count >= 1
                && root.join(&failure.stderr_path).is_file()
                && root.join(&failure.failure_record_path).is_file()
        }));
    }

    #[cfg(not(feature = "bam_downstream"))]
    #[test]
    fn fake_run_local_stage_commands_explain_downstream_feature_requirement() {
        let root = repo_root();
        let error =
            fake_run_local_stage_commands(&root, PathBuf::from(DEFAULT_LOCAL_STAGE_FAKE_RUN_ROOT))
                .expect_err("fake-run without bam_downstream should explain the missing feature");

        assert!(
            error.to_string().contains("requires the `bam_downstream` feature"),
            "missing-feature error should stay explicit: {error:#}"
        );
    }

    #[cfg(not(feature = "bam_downstream"))]
    #[test]
    fn fake_run_local_stage_failures_explain_downstream_feature_requirement() {
        let root = repo_root();
        let error = fake_run_local_stage_failures(
            &root,
            PathBuf::from(DEFAULT_LOCAL_STAGE_FAKE_FAILURE_ROOT),
            &[],
            1,
        )
        .expect_err("fake-run failures without bam_downstream should explain the missing feature");

        assert!(
            error.to_string().contains("requires the `bam_downstream` feature"),
            "missing-feature error should stay explicit: {error:#}"
        );
    }
}
