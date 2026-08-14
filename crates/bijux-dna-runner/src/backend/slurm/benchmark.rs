//! Owner: bijux-dna-runner
//! Immutable Slurm materialization for executable benchmark specifications.

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    validate_execution_spec, validate_stage_plan_manifest, BenchmarkExecutionSpec,
    BenchmarkStagePlanManifest,
};

use super::{SlurmDependencyCondition, SlurmSubmitRequest};

#[derive(Debug, serde::Deserialize)]
struct ExecutableStagePlan {
    stage_instance_id: Option<String>,
    command: ExecutableStagePlanCommand,
    io: ExecutableStagePlanIo,
    out_dir: PathBuf,
}

#[derive(Debug, serde::Deserialize)]
struct ExecutableStagePlanCommand {
    template: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
struct ExecutableStagePlanIo {
    outputs: Vec<ExecutableStagePlanOutput>,
}

#[derive(Debug, serde::Deserialize)]
struct ExecutableStagePlanOutput {
    path: PathBuf,
}

#[derive(Debug)]
struct PreparedStagePlan {
    stage_instance_id: String,
    command: Vec<String>,
    resource_path: String,
}

const MAX_SPEC_BYTES: usize = 16 * 1024 * 1024;

/// Exact scheduler material prepared from one immutable benchmark execution specification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedBenchmarkSlurmJob {
    pub request: SlurmSubmitRequest,
    pub execution_spec_sha256: String,
    pub stage_plan_manifest_sha256: String,
    pub script_sha256: String,
}

/// Validate executable authorities and publish one restart-safe Slurm script without submitting it.
///
/// Existing byte-identical scripts are accepted. Conflicting scripts, mutated authorities,
/// symlinks, pre-existing benchmark outputs, and malformed scheduler tokens fail closed.
///
/// # Errors
/// Returns an error when the specification or any on-disk authority is invalid, or scheduler
/// material cannot be published without replacing existing evidence.
pub fn prepare_benchmark_slurm_job(
    spec: &BenchmarkExecutionSpec,
    submitted_at_utc: &str,
) -> Result<PreparedBenchmarkSlurmJob> {
    validate_execution_spec(spec).context("validate benchmark execution specification")?;
    require_timestamp_text(submitted_at_utc)?;
    require_scheduler_token(&spec.resources.account, "account")?;
    require_scheduler_token(&spec.resources.partition, "partition")?;
    require_shell_commands(spec)?;

    let repository_path = require_canonical_directory(
        Path::new(&spec.source.repository_path),
        "benchmark source repository",
    )?;
    bijux_dna_containers::api::verify_frozen_repository(
        &repository_path,
        &spec.source.repository_commit,
    )
    .context("admit immutable benchmark source repository")?;
    let result_root =
        require_canonical_directory(Path::new(&spec.outputs.result_root), "benchmark result root")?;
    let image_path =
        require_exact_file(Path::new(&spec.image.path), &spec.image.sha256, "benchmark image")?;
    for input in &spec.inputs {
        require_bind_safe(&input.path, "benchmark input path")?;
        require_bind_safe(&input.mount_path, "benchmark input mount")?;
        require_exact_file(Path::new(&input.path), &input.sha256, "benchmark input")?;
    }
    require_bind_safe(repository_path.to_string_lossy().as_ref(), "repository bind path")?;
    require_bind_safe(result_root.to_string_lossy().as_ref(), "result bind path")?;
    require_bind_safe(image_path.to_string_lossy().as_ref(), "image path")?;
    reject_existing_outputs(spec, &result_root)?;
    ensure_output_directories(spec, &result_root)?;

    let execution_spec_path = result_root.join("execution-spec.json");
    let execution_spec_sha256 = require_persisted_spec(&execution_spec_path, spec)?;
    let (stage_plan_manifest_sha256, stage_plans) =
        require_stage_plan_manifest(&result_root, spec, &execution_spec_sha256)?;
    let scheduler_directory = ensure_owned_directory(&result_root.join("scheduler"))?;
    let script_path = scheduler_directory.join("job.sbatch");
    let stdout_path = scheduler_directory.join("stdout.log");
    let stderr_path = scheduler_directory.join("stderr.log");
    let script =
        render_script(spec, &execution_spec_sha256, &stage_plan_manifest_sha256, &stage_plans);
    ensure_file_identity(&script_path, script.as_bytes())?;
    let script_sha256 = bijux_dna_infra::hash_file_sha256(&script_path)
        .context("hash prepared benchmark Slurm script")?;

    Ok(PreparedBenchmarkSlurmJob {
        request: SlurmSubmitRequest {
            run_id: format!(
                "{}:{}:{}",
                spec.identity.run.run_id, spec.identity.tool_id, spec.identity.attempt_id
            ),
            dependencies: Vec::new(),
            dependency_condition: SlurmDependencyCondition::AfterOk,
            array: None,
            script_path,
            working_directory: repository_path,
            stdout_path,
            stderr_path,
            submitted_at_utc: submitted_at_utc.to_string(),
        },
        execution_spec_sha256,
        stage_plan_manifest_sha256,
        script_sha256,
    })
}

fn render_script(
    spec: &BenchmarkExecutionSpec,
    execution_spec_sha256: &str,
    stage_plan_manifest_sha256: &str,
    stage_plans: &[PreparedStagePlan],
) -> String {
    let mut lines = script_preflight(spec, execution_spec_sha256, stage_plan_manifest_sha256);
    append_validation_commands(&mut lines, spec);
    append_warmup_commands(&mut lines, spec);
    append_stage_plan_commands(&mut lines, spec, stage_plans);
    append_measurement_commands(&mut lines, spec);
    append_completion_commands(&mut lines, spec, execution_spec_sha256);
    lines.push(String::new());
    lines.join("\n")
}

fn script_preflight(
    spec: &BenchmarkExecutionSpec,
    execution_spec_sha256: &str,
    stage_plan_manifest_sha256: &str,
) -> Vec<String> {
    let result_root = shell_quote(&spec.outputs.result_root);
    let repository_path = shell_quote(&spec.source.repository_path);
    let image_path = shell_quote(&spec.image.path);
    let mut lines = vec![
        "#!/usr/bin/env bash".to_string(),
        format!("#SBATCH --job-name={}", benchmark_job_name(spec)),
        format!("#SBATCH --account={}", spec.resources.account),
        format!("#SBATCH --partition={}", spec.resources.partition),
        format!("#SBATCH --cpus-per-task={}", spec.resources.cpus),
        format!("#SBATCH --mem={}M", spec.resources.memory_mb),
        format!("#SBATCH --time={}", spec.resources.walltime),
        String::new(),
        "set -euo pipefail".to_string(),
        "umask 077".to_string(),
        format!("readonly RESULT_ROOT={result_root}"),
        format!("readonly REPOSITORY_ROOT={repository_path}"),
        format!("readonly IMAGE_PATH={image_path}"),
        "readonly SCHEDULER_ROOT=\"${RESULT_ROOT}/scheduler\"".to_string(),
        "readonly VALIDATION_ROOT=\"${RESULT_ROOT}/validation\"".to_string(),
        "readonly SCRATCH_ROOT=\"${SLURM_TMPDIR:-${SCHEDULER_ROOT}/scratch}\"".to_string(),
        "mkdir \"${SCHEDULER_ROOT}/execution-lock\"".to_string(),
        "test -d \"${VALIDATION_ROOT}\" && test ! -L \"${VALIDATION_ROOT}\"".to_string(),
        "test -d \"${RESULT_ROOT}/repetitions\" && test ! -L \"${RESULT_ROOT}/repetitions\""
            .to_string(),
        "mkdir -p \"${SCRATCH_ROOT}\"".to_string(),
        "cleanup() { rm -rf \"${SCRATCH_ROOT}\"; }".to_string(),
        "trap cleanup EXIT".to_string(),
        "command -v apptainer >/dev/null".to_string(),
        "test -x /usr/bin/time".to_string(),
        "command -v find >/dev/null".to_string(),
        "command -v sha256sum >/dev/null".to_string(),
        "command -v sort >/dev/null".to_string(),
        "command -v git >/dev/null".to_string(),
        "test \"$(git -C \"${REPOSITORY_ROOT}\" rev-parse HEAD)\" = ".to_string()
            + &shell_quote(&spec.source.repository_commit),
        "test -z \"$(git -C \"${REPOSITORY_ROOT}\" status --porcelain --untracked-files=all)\""
            .to_string(),
        format!(
            "test \"$(sha256sum \"${{RESULT_ROOT}}/execution-spec.json\" | awk '{{print $1}}')\" = {}",
            shell_quote(execution_spec_sha256)
        ),
        format!(
            "test \"$(sha256sum \"${{RESULT_ROOT}}/stage-plan-manifest.json\" | awk '{{print $1}}')\" = {}",
            shell_quote(stage_plan_manifest_sha256)
        ),
        format!(
            "test \"$(sha256sum \"${{IMAGE_PATH}}\" | awk '{{print $1}}')\" = {}",
            shell_quote(&spec.image.sha256)
        ),
    ];
    for input in &spec.inputs {
        lines.push(format!(
            "test \"$(sha256sum {} | awk '{{print $1}}')\" = {}",
            shell_quote(&input.path),
            shell_quote(&input.sha256)
        ));
    }
    for output in &spec.outputs.required_files {
        let path = result_shell_path(output);
        lines.push(format!("test ! -e {path}"));
        lines.push(format!("test ! -L {path}"));
    }
    lines
}

fn require_stage_plan_manifest(
    result_root: &Path,
    spec: &BenchmarkExecutionSpec,
    execution_spec_sha256: &str,
) -> Result<(String, Vec<PreparedStagePlan>)> {
    let manifest_path = result_root.join("stage-plan-manifest.json");
    let manifest: BenchmarkStagePlanManifest = read_json_bounded(&manifest_path)?;
    validate_stage_plan_manifest(&manifest).context("validate benchmark stage-plan manifest")?;
    if manifest.identity != spec.identity || manifest.execution_spec_sha256 != execution_spec_sha256
    {
        bail!("benchmark stage-plan manifest differs from execution specification");
    }
    let mut plans = Vec::with_capacity(manifest.plans.len());
    for reference in &manifest.plans {
        let path = result_root.join(&reference.path);
        require_exact_file(&path, &reference.sha256, "benchmark stage plan")?;
        let plan: ExecutableStagePlan = read_json_bounded(&path)?;
        plans.push(validate_executable_stage_plan(result_root, spec, reference, plan)?);
    }
    let digest = bijux_dna_infra::hash_file_sha256(&manifest_path)
        .context("hash benchmark stage-plan manifest")?;
    Ok((digest, plans))
}

fn validate_executable_stage_plan(
    result_root: &Path,
    spec: &BenchmarkExecutionSpec,
    reference: &bijux_dna_bench_model::BenchmarkStagePlanReference,
    plan: ExecutableStagePlan,
) -> Result<PreparedStagePlan> {
    if plan.stage_instance_id.as_deref() != Some(reference.stage_instance_id.as_str())
        || plan.out_dir != result_root.join("repetitions").join(&reference.stage_instance_id)
        || plan.command.template.is_empty()
        || plan
            .command
            .template
            .iter()
            .any(|argument| argument.is_empty() || argument.contains('\0'))
        || plan.io.outputs.is_empty()
    {
        bail!("benchmark stage plan is not independently executable");
    }
    let container_root =
        Path::new(&spec.outputs.mount_path).join("repetitions").join(&reference.stage_instance_id);
    let required = spec
        .outputs
        .required_files
        .iter()
        .map(String::as_str)
        .collect::<std::collections::BTreeSet<_>>();
    for output in plan.io.outputs {
        let relative = output
            .path
            .strip_prefix(&spec.outputs.mount_path)
            .context("benchmark stage-plan output escapes the result mount")?;
        if !output.path.starts_with(&container_root)
            || relative.components().any(|component| {
                matches!(component, std::path::Component::CurDir | std::path::Component::ParentDir)
            })
            || !required.contains(relative.to_string_lossy().as_ref())
        {
            bail!("benchmark stage-plan output differs from required output authority");
        }
    }
    ensure_owned_directory(&plan.out_dir)?;
    let resource_path = format!("repetitions/{}/resource.tsv", reference.stage_instance_id);
    if !required.contains(resource_path.as_str()) {
        bail!("benchmark stage-plan resource evidence is not a required output");
    }
    Ok(PreparedStagePlan {
        stage_instance_id: reference.stage_instance_id.clone(),
        command: plan.command.template,
        resource_path,
    })
}

fn append_validation_commands(lines: &mut Vec<String>, spec: &BenchmarkExecutionSpec) {
    let version = container_invocation(spec, &spec.commands.version);
    let help = container_invocation(spec, &spec.commands.help);
    let negative = container_invocation(spec, &spec.commands.negative);
    lines.extend([
        format!("{version} > \"${{VALIDATION_ROOT}}/version.stdout\" 2> \"${{VALIDATION_ROOT}}/version.stderr\""),
        "cat \"${VALIDATION_ROOT}/version.stdout\" \"${VALIDATION_ROOT}/version.stderr\" > \"${VALIDATION_ROOT}/version.txt\"".to_string(),
        format!(
            "grep -E -- {} \"${{VALIDATION_ROOT}}/version.txt\" >/dev/null",
            shell_quote(&spec.acceptance.version_regex)
        ),
        format!("{help} > \"${{VALIDATION_ROOT}}/help.stdout\" 2> \"${{VALIDATION_ROOT}}/help.stderr\""),
        format!("if {negative} > \"${{VALIDATION_ROOT}}/negative.stdout\" 2> \"${{VALIDATION_ROOT}}/negative.stderr\"; then exit 70; fi"),
    ]);
}

fn append_warmup_commands(lines: &mut Vec<String>, spec: &BenchmarkExecutionSpec) {
    for _ in 0..spec.measurement.warmup_repetitions {
        lines.push(container_invocation(spec, &spec.commands.warmup));
    }
}

fn append_stage_plan_commands(
    lines: &mut Vec<String>,
    spec: &BenchmarkExecutionSpec,
    stage_plans: &[PreparedStagePlan],
) {
    let resource_format = "wall_seconds\tuser_seconds\tsystem_seconds\tmax_rss_kb\tfilesystem_inputs\tfilesystem_outputs\texit_code\n%e\t%U\t%S\t%M\t%I\t%O\t%x";
    for plan in stage_plans {
        lines.push(format!(
            "test -d {} && test ! -L {}",
            result_shell_path(&format!("repetitions/{}", plan.stage_instance_id)),
            result_shell_path(&format!("repetitions/{}", plan.stage_instance_id)),
        ));
        lines.push(format!(
            "'/usr/bin/time' '-o' {} '-f' {} {}",
            result_shell_path(&format!("{}.partial", plan.resource_path)),
            shell_quote(resource_format),
            container_invocation(spec, &plan.command),
        ));
        lines.push(format!(
            "mv {} {}",
            result_shell_path(&format!("{}.partial", plan.resource_path)),
            result_shell_path(&plan.resource_path),
        ));
    }
}

fn append_measurement_commands(lines: &mut Vec<String>, spec: &BenchmarkExecutionSpec) {
    for repetition in 1..=spec.measurement.measured_repetitions {
        let measured_command = spec
            .commands
            .measured
            .iter()
            .map(|argument| argument.replace("{repetition}", &repetition.to_string()))
            .collect::<Vec<_>>();
        let resource_path = spec
            .measurement
            .resource_file_template
            .replace("{repetition}", &repetition.to_string());
        let resource_format = "wall_seconds\tuser_seconds\tsystem_seconds\tmax_rss_kb\tfilesystem_inputs\tfilesystem_outputs\texit_code\n%e\t%U\t%S\t%M\t%I\t%O\t%x";
        lines.push(format!(
            "'/usr/bin/time' '-o' {} '-f' {} {}",
            result_shell_path(&format!("{resource_path}.partial")),
            shell_quote(resource_format),
            container_invocation(spec, &measured_command),
        ));
        lines.push(format!(
            "mv {} {}",
            result_shell_path(&format!("{resource_path}.partial")),
            result_shell_path(&resource_path)
        ));
    }
}

fn append_completion_commands(
    lines: &mut Vec<String>,
    spec: &BenchmarkExecutionSpec,
    execution_spec_sha256: &str,
) {
    lines.push(": > \"${SCHEDULER_ROOT}/output-sha256.tsv.partial\"".to_string());
    for output in &spec.outputs.required_files {
        let path = result_shell_path(output);
        lines.push(format!("if test -f {path}; then"));
        lines.push(format!("  test ! -L {path}"));
        lines.push(format!("  output_digest=$(sha256sum {path} | awk '{{print $1}}')"));
        lines.push(format!(
            "  printf '%s\\t%s\\n' {} \"${{output_digest}}\" >> \"${{SCHEDULER_ROOT}}/output-sha256.tsv.partial\"",
            shell_quote(output)
        ));
        lines.push(format!("elif test -d {path}; then"));
        lines.push(format!("  test ! -L {path}"));
        lines.push(format!("  test -n \"$(find {path} -type f -print -quit)\""));
        lines.push(format!("  test -z \"$(find {path} -type l -print -quit)\""));
        lines.push(format!(
            "  test -z \"$(find {path} ! -type d ! -type f ! -type l -print -quit)\""
        ));
        lines.push(format!(
            "  find {path} -type f -print0 | LC_ALL=C sort -z | while IFS= read -r -d '' discovered_output; do"
        ));
        lines.extend([
            "    relative_output=${discovered_output#\"${RESULT_ROOT}/\"}".to_string(),
            "    case \"${relative_output}\" in *$'\\n'*|*$'\\t'*) exit 70 ;; esac".to_string(),
            "    output_digest=$(sha256sum \"${discovered_output}\" | awk '{print $1}')".to_string(),
            "    printf '%s\\t%s\\n' \"${relative_output}\" \"${output_digest}\" >> \"${SCHEDULER_ROOT}/output-sha256.tsv.partial\"".to_string(),
            "  done".to_string(),
            "else".to_string(),
            format!("  printf 'required benchmark output is absent: %s\\n' {} >&2", shell_quote(output)),
            "  exit 70".to_string(),
            "fi".to_string(),
        ]);
    }
    lines.extend([
        "mv \"${SCHEDULER_ROOT}/output-sha256.tsv.partial\" \"${SCHEDULER_ROOT}/output-sha256.tsv\"".to_string(),
        format!(
            "printf '{{\"schema_version\":\"bijux.runner.benchmark_completion.v1\",\"execution_spec_sha256\":\"{execution_spec_sha256}\",\"output_manifest\":\"scheduler/output-sha256.tsv\"}}\\n' > \"${{SCHEDULER_ROOT}}/completed.json.partial\""
        ),
        "mv \"${SCHEDULER_ROOT}/completed.json.partial\" \"${SCHEDULER_ROOT}/completed.json\""
            .to_string(),
    ]);
}

fn container_invocation(spec: &BenchmarkExecutionSpec, command: &[String]) -> String {
    let mut arguments = vec![
        "apptainer".to_string(),
        "exec".to_string(),
        "--containall".to_string(),
        "--cleanenv".to_string(),
        "--net".to_string(),
        "--network".to_string(),
        "none".to_string(),
        "--pwd".to_string(),
        "/repository".to_string(),
        "--env".to_string(),
        "LANG=C".to_string(),
        "--env".to_string(),
        "LC_ALL=C".to_string(),
        "--env".to_string(),
        "TZ=UTC".to_string(),
        "--env".to_string(),
        "TMPDIR=/scratch".to_string(),
        "--bind".to_string(),
        format!("{}:/repository:ro", spec.source.repository_path),
    ];
    for input in &spec.inputs {
        arguments.extend(["--bind".to_string(), format!("{}:{}:ro", input.path, input.mount_path)]);
    }
    arguments.extend([
        "--bind".to_string(),
        format!("{}:/result:rw", spec.outputs.result_root),
        "--bind".to_string(),
        "${SCRATCH_ROOT}:/scratch:rw".to_string(),
        spec.image.path.clone(),
    ]);
    arguments.extend_from_slice(command);
    arguments
        .iter()
        .map(|argument| {
            if argument == "${SCRATCH_ROOT}:/scratch:rw" {
                "\"${SCRATCH_ROOT}:/scratch:rw\"".to_string()
            } else {
                shell_quote(argument)
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn reject_existing_outputs(spec: &BenchmarkExecutionSpec, result_root: &Path) -> Result<()> {
    for relative in &spec.outputs.required_files {
        let path = result_root.join(relative);
        require_absent(&path, "benchmark required output")?;
    }
    let completion = result_root.join("scheduler/completed.json");
    require_absent(&completion, "benchmark completion evidence")
}

fn require_absent(path: &Path, label: &str) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(_) => bail!("{label} already exists: {}", path.display()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error).with_context(|| format!("inspect {label} {}", path.display())),
    }
}

fn ensure_output_directories(spec: &BenchmarkExecutionSpec, result_root: &Path) -> Result<()> {
    ensure_owned_directory(&result_root.join("validation"))?;
    for relative in &spec.outputs.required_files {
        let parent = Path::new(relative)
            .parent()
            .context("benchmark required output has no parent directory")?;
        let mut current = result_root.to_path_buf();
        for component in parent.components() {
            current.push(component.as_os_str());
            ensure_owned_directory(&current)?;
        }
    }
    Ok(())
}

fn require_persisted_spec(path: &Path, expected: &BenchmarkExecutionSpec) -> Result<String> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("inspect persisted execution specification {}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("persisted execution specification is not a regular file");
    }
    let bytes = bijux_dna_infra::read_to_end_bounded(path, MAX_SPEC_BYTES)
        .with_context(|| format!("read persisted execution specification {}", path.display()))?;
    let actual: BenchmarkExecutionSpec = serde_json::from_slice(&bytes)
        .with_context(|| format!("parse persisted execution specification {}", path.display()))?;
    validate_execution_spec(&actual).context("validate persisted execution specification")?;
    if actual != *expected {
        bail!("persisted execution specification differs from requested benchmark execution");
    }
    bijux_dna_infra::hash_file_sha256(path).context("hash persisted execution specification")
}

fn read_json_bounded<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("inspect immutable JSON {}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("immutable JSON is not a regular file: {}", path.display());
    }
    let bytes = bijux_dna_infra::read_to_end_bounded(path, MAX_SPEC_BYTES)
        .with_context(|| format!("read immutable JSON {}", path.display()))?;
    serde_json::from_slice(&bytes)
        .with_context(|| format!("parse immutable JSON {}", path.display()))
}

fn require_exact_file(path: &Path, expected_sha256: &str, label: &str) -> Result<PathBuf> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("inspect {label} {}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("{label} is not a regular non-symlink file");
    }
    let canonical = path.canonicalize().with_context(|| format!("canonicalize {label}"))?;
    if canonical != path {
        bail!("{label} path is not canonical");
    }
    let actual =
        bijux_dna_infra::hash_file_sha256(path).with_context(|| format!("hash {label}"))?;
    if actual != expected_sha256 {
        bail!("{label} digest differs from execution specification");
    }
    Ok(canonical)
}

fn require_canonical_directory(path: &Path, label: &str) -> Result<PathBuf> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("inspect {label} {}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        bail!("{label} is not a non-symlink directory");
    }
    let canonical = path.canonicalize().with_context(|| format!("canonicalize {label}"))?;
    if canonical != path {
        bail!("{label} path is not canonical");
    }
    Ok(canonical)
}

fn ensure_owned_directory(path: &Path) -> Result<PathBuf> {
    match fs::create_dir(path) {
        Ok(()) => {}
        Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
        Err(error) => return Err(error).with_context(|| format!("create {}", path.display())),
    }
    require_canonical_directory(path, "benchmark scheduler directory")
}

fn ensure_file_identity(path: &Path, expected: &[u8]) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                bail!("benchmark scheduler script is not a regular file");
            }
            let actual = bijux_dna_infra::read_to_end_bounded(path, MAX_SPEC_BYTES)
                .context("read existing benchmark scheduler script")?;
            if actual != expected {
                bail!("benchmark scheduler script conflicts with immutable attempt evidence");
            }
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {
            bijux_dna_infra::atomic_create_bytes(path, expected)
                .context("write benchmark scheduler script")?;
        }
        Err(error) => return Err(error).context("inspect benchmark scheduler script"),
    }
    Ok(())
}

fn require_scheduler_token(value: &str, label: &str) -> Result<()> {
    if value.is_empty()
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        bail!("Slurm {label} contains unsupported characters");
    }
    Ok(())
}

fn require_bind_safe(value: &str, label: &str) -> Result<()> {
    if value.chars().any(|character| matches!(character, ':' | ',' | '\n' | '\r')) {
        bail!("{label} cannot be represented as an exact Apptainer bind");
    }
    Ok(())
}

fn require_timestamp_text(value: &str) -> Result<()> {
    if value.trim().is_empty()
        || value != value.trim()
        || value.chars().any(|character| matches!(character, '\n' | '\r' | '\0'))
    {
        bail!("Slurm submission timestamp is empty or malformed");
    }
    Ok(())
}

fn require_shell_commands(spec: &BenchmarkExecutionSpec) -> Result<()> {
    for command in [
        &spec.commands.version,
        &spec.commands.help,
        &spec.commands.warmup,
        &spec.commands.measured,
        &spec.commands.negative,
    ] {
        if command.iter().any(|argument| argument.contains('\0')) {
            bail!("benchmark command contains a null byte");
        }
    }
    if spec
        .acceptance
        .version_regex
        .chars()
        .any(|character| matches!(character, '\0' | '\n' | '\r'))
    {
        bail!("benchmark version regular expression is not shell-safe text");
    }
    Ok(())
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn benchmark_job_name(spec: &BenchmarkExecutionSpec) -> String {
    let identity = format!(
        "{}\0{}\0{}\0{}\0{}\0{}",
        spec.identity.run.run_id,
        spec.identity.run.domain,
        spec.identity.run.stage_id,
        spec.identity.tool_id,
        spec.identity.campaign_id,
        spec.identity.attempt_id
    );
    let digest = bijux_dna_infra::hash_bytes_sha256(identity.as_bytes());
    let mut prefix = format!(
        "bijux-{}-{}-{}",
        spec.identity.run.run_id, spec.identity.tool_id, spec.identity.attempt_id
    );
    prefix.truncate(prefix.len().min(96));
    format!("{prefix}-{}", &digest[..16])
}

fn result_shell_path(relative: &str) -> String {
    format!("\"${{RESULT_ROOT}}/\"{}", shell_quote(relative))
}
