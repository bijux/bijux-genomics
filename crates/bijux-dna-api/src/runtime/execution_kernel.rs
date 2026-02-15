use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::Duration;

use anyhow::{anyhow, bail, Context, Result};
use bijux_dna_core::contract::ExecutionStep;
use bijux_dna_environment::api::RuntimeKind;
use bijux_dna_runner::execute::{execute_step, StageResultV1};
use bijux_dna_runtime::recording::write_execution_logs_bounded;
use serde::{Deserialize, Serialize};

use crate::writers::{ArtifactWriter, MetricsWriter};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum NetworkPolicy {
    Allow,
    Forbid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ToolContext {
    pub run_id: String,
    pub stage_id: String,
    pub tool_id: String,
    pub sample_id: Option<String>,
    pub stage_root: PathBuf,
    pub input_root: PathBuf,
    pub output_root: PathBuf,
    pub tmp_root: PathBuf,
    pub threads: u32,
    pub memory_hint_mb: Option<u64>,
    pub compression_threads: Option<u32>,
    pub seed: Option<u64>,
    pub network_policy: NetworkPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ToolInvocationRequest {
    pub step: ExecutionStep,
    pub runner: RuntimeKind,
    pub context: ToolContext,
    pub timeout: Option<Duration>,
    #[serde(default)]
    pub mode: ToolExecMode,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ToolExecMode {
    #[default]
    Execute,
    DryRun,
    DryRunExplain,
    PrintCommands,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct ToolInvocationResult {
    pub stage_result: StageResultV1,
    pub runtime_provenance_path: PathBuf,
    pub stage_manifest_path: PathBuf,
    pub stdout_path: PathBuf,
    pub stderr_path: PathBuf,
    pub summary_path: PathBuf,
}

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct ToolExec;

impl ToolExec {
    /// # Errors
    /// Returns an error if path contracts fail, tool execution fails, or artifacts cannot be written.
    pub fn invoke(req: &ToolInvocationRequest) -> Result<ToolInvocationResult> {
        invoke_tool(req)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StageResourceKnobs {
    threads: Option<u32>,
    memory_mb: Option<u64>,
    compression_threads: Option<u32>,
    timeout_s: Option<u64>,
    temp_root: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DeterministicEnvKnobs {
    lc_all: Option<String>,
    lang: Option<String>,
    tz: Option<String>,
    umask: Option<String>,
    path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RuntimeExecutionConfig {
    default_threads: Option<u32>,
    default_memory_mb: Option<u64>,
    default_compression_threads: Option<u32>,
    default_timeout_s: Option<u64>,
    default_temp_root: Option<String>,
    heavy_stage_patterns: Option<Vec<String>>,
    max_local_heavy_parallel: Option<u32>,
    bgzip_tabix_max_parallel: Option<u32>,
    cache_root: Option<String>,
    deterministic_env: Option<DeterministicEnvKnobs>,
    per_stage: Option<std::collections::BTreeMap<String, StageResourceKnobs>>,
}

#[derive(Debug, Clone)]
struct EffectiveRuntimePolicy {
    threads: u32,
    memory_mb: Option<u64>,
    compression_threads: Option<u32>,
    timeout: Option<Duration>,
    temp_root: Option<PathBuf>,
    cache_root: Option<PathBuf>,
    heavy_patterns: Vec<String>,
    max_local_heavy_parallel: u32,
    bgzip_tabix_max_parallel: u32,
    deterministic_env: DeterministicEnvKnobs,
}

static EXEC_POLICY: OnceLock<RuntimeExecutionConfig> = OnceLock::new();

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

fn load_runtime_execution_config() -> RuntimeExecutionConfig {
    let path = workspace_root().join("configs/runtime/execution_kernel.toml");
    let Ok(raw) = std::fs::read_to_string(&path) else {
        return RuntimeExecutionConfig {
            default_threads: None,
            default_memory_mb: None,
            default_compression_threads: Some(1),
            default_timeout_s: None,
            default_temp_root: None,
            heavy_stage_patterns: Some(vec![
                "bam.align".to_string(),
                "vcf.impute".to_string(),
                "vcf.phasing".to_string(),
            ]),
            max_local_heavy_parallel: Some(1),
            bgzip_tabix_max_parallel: Some(1),
            cache_root: None,
            deterministic_env: Some(DeterministicEnvKnobs {
                lc_all: Some("C".to_string()),
                lang: Some("C".to_string()),
                tz: Some("UTC".to_string()),
                umask: Some("027".to_string()),
                path: None,
            }),
            per_stage: None,
        };
    };
    toml::from_str::<RuntimeExecutionConfig>(&raw)
        .unwrap_or_else(|err| panic!("invalid runtime execution config {}: {err}", path.display()))
}

fn runtime_execution_config() -> &'static RuntimeExecutionConfig {
    EXEC_POLICY.get_or_init(load_runtime_execution_config)
}

fn stage_matches(pattern: &str, stage_id: &str) -> bool {
    if let Some(prefix) = pattern.strip_suffix('*') {
        stage_id.starts_with(prefix)
    } else {
        stage_id == pattern
    }
}

fn effective_runtime_policy(req: &ToolInvocationRequest) -> EffectiveRuntimePolicy {
    let cfg = runtime_execution_config();
    let root = workspace_root();
    let mut stage_knobs = StageResourceKnobs {
        threads: None,
        memory_mb: None,
        compression_threads: None,
        timeout_s: None,
        temp_root: None,
    };
    if let Some(per_stage) = &cfg.per_stage {
        for (pattern, knobs) in per_stage {
            if stage_matches(pattern, &req.context.stage_id) {
                stage_knobs = knobs.clone();
            }
        }
    }
    let threads = stage_knobs
        .threads
        .or(cfg.default_threads)
        .unwrap_or(req.context.threads)
        .max(1);
    let memory_mb = stage_knobs
        .memory_mb
        .or(cfg.default_memory_mb)
        .or(req.context.memory_hint_mb);
    let compression_threads = stage_knobs
        .compression_threads
        .or(cfg.default_compression_threads)
        .or(req.context.compression_threads)
        .map(|v| v.max(1));
    let timeout = req.timeout.or_else(|| {
        stage_knobs
            .timeout_s
            .or(cfg.default_timeout_s)
            .map(Duration::from_secs)
    });
    let temp_root = stage_knobs
        .temp_root
        .or_else(|| cfg.default_temp_root.clone())
        .map(PathBuf::from)
        .map(|path| {
            if path.is_absolute() {
                path
            } else {
                root.join(path)
            }
        });
    let cache_root = cfg.cache_root.clone().map(PathBuf::from).map(|path| {
        if path.is_absolute() {
            path
        } else {
            root.join(path)
        }
    });
    let deterministic_env = cfg
        .deterministic_env
        .clone()
        .unwrap_or(DeterministicEnvKnobs {
            lc_all: Some("C".to_string()),
            lang: Some("C".to_string()),
            tz: Some("UTC".to_string()),
            umask: Some("027".to_string()),
            path: None,
        });
    EffectiveRuntimePolicy {
        threads,
        memory_mb,
        compression_threads,
        timeout,
        temp_root,
        cache_root,
        heavy_patterns: cfg.heavy_stage_patterns.clone().unwrap_or_default(),
        max_local_heavy_parallel: cfg.max_local_heavy_parallel.unwrap_or(1).max(1),
        bgzip_tabix_max_parallel: cfg.bgzip_tabix_max_parallel.unwrap_or(1).max(1),
        deterministic_env,
    }
}

fn acquire_slot_lock(
    base: &Path,
    prefix: &str,
    slots: u32,
) -> Result<Option<bijux_dna_infra::FileLock>> {
    if slots <= 1 {
        let lock = bijux_dna_infra::FileLock::acquire(
            &base.join(format!("{prefix}.lock")),
            Duration::from_secs(300),
        )
        .map_err(|err| anyhow!("acquire {prefix} lock: {err}"))?;
        return Ok(Some(lock));
    }
    for slot in 0..slots {
        let path = base.join(format!("{prefix}.{slot}.lock"));
        if let Ok(lock) = bijux_dna_infra::FileLock::acquire(&path, Duration::from_millis(150)) {
            return Ok(Some(lock));
        }
    }
    let lock = bijux_dna_infra::FileLock::acquire(
        &base.join(format!("{prefix}.0.lock")),
        Duration::from_secs(300),
    )
    .map_err(|err| anyhow!("acquire {prefix} lock: {err}"))?;
    Ok(Some(lock))
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub(crate) enum ExitTaxonomy {
    ToolFailure,
    UserError,
    ContractViolation,
}

fn canonicalize_existing(path: &Path) -> Result<PathBuf> {
    if path.exists() {
        return path
            .canonicalize()
            .with_context(|| format!("canonicalize {}", path.display()));
    }
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(std::env::current_dir()
            .context("resolve cwd for relative path contracts")?
            .join(path))
    }
}

fn ensure_subpath(path: &Path, root: &Path, label: &str) -> Result<()> {
    let cpath = canonicalize_existing(path)?;
    let croot = canonicalize_existing(root)?;
    if !cpath.starts_with(&croot) {
        bail!(
            "{label} path contract violated: {} not under {}",
            cpath.display(),
            croot.display()
        );
    }
    Ok(())
}

fn enforce_path_contracts(req: &ToolInvocationRequest) -> Result<()> {
    ensure_subpath(
        &req.context.stage_root,
        &req.context.output_root,
        "stage_root",
    )?;
    ensure_subpath(&req.context.tmp_root, &req.context.output_root, "tmp_root")?;
    for artifact in &req.step.io.outputs {
        ensure_subpath(&artifact.path, &req.context.output_root, "output")?;
    }
    for artifact in &req.step.io.inputs {
        ensure_subpath(&artifact.path, &req.context.input_root, "input")?;
    }
    Ok(())
}

fn require_pinned_digest(step: &ExecutionStep) -> Result<()> {
    let digest = step.image.digest.as_deref().ok_or_else(|| {
        anyhow!(
            "tool resolution failed: missing image digest for {}",
            step.image.image
        )
    })?;
    if !digest.starts_with("sha256:") || digest.len() < 16 {
        bail!(
            "tool resolution failed: unpinned/invalid digest `{digest}` for {}",
            step.image.image
        );
    }
    Ok(())
}

fn classify_exit_code(exit_code: i32) -> ExitTaxonomy {
    match exit_code {
        2 | 64..=78 => ExitTaxonomy::UserError,
        126 | 127 => ExitTaxonomy::ContractViolation,
        _ => ExitTaxonomy::ToolFailure,
    }
}

fn infer_tool_version_from_image(image: &str) -> String {
    let without_digest = image.split('@').next().unwrap_or(image);
    if let Some((_, tag)) = without_digest.rsplit_once(':') {
        if !tag.is_empty() && tag != "latest" {
            return tag.to_string();
        }
    }
    "unknown".to_string()
}

fn infer_version_line(stdout: &str, stderr: &str) -> Option<String> {
    let pick = |text: &str| {
        text.lines().find_map(|line| {
            let lower = line.to_ascii_lowercase();
            if lower.contains("version") {
                Some(line.trim().to_string())
            } else {
                None
            }
        })
    };
    pick(stdout).or_else(|| pick(stderr))
}

fn network_policy_violation(policy: &NetworkPolicy) -> bool {
    matches!(policy, NetworkPolicy::Forbid)
        && std::env::var("BIJUX_ALLOW_NETWORK")
            .ok()
            .is_some_and(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
}

fn validate_required_outputs(step: &ExecutionStep) -> Result<()> {
    for artifact in &step.io.outputs {
        if artifact.optional {
            continue;
        }
        if !artifact.path.exists() {
            bail!(
                "stage contract violation: required output '{}' was not produced at {}",
                artifact.name,
                artifact.path.display()
            );
        }
    }
    Ok(())
}

fn output_checksums(step: &ExecutionStep) -> Result<serde_json::Value> {
    ArtifactWriter::write_output_checksums(&step.out_dir, &step.io.outputs)
}

fn can_resume(req: &ToolInvocationRequest) -> Result<bool> {
    let manifest_path = req.context.stage_root.join("stage_manifest.json");
    if !manifest_path.exists() {
        return Ok(false);
    }
    let all_outputs_exist = req
        .step
        .io
        .outputs
        .iter()
        .all(|artifact| artifact.optional || artifact.path.exists());
    if !all_outputs_exist {
        return Ok(false);
    }
    let raw = std::fs::read_to_string(&manifest_path)?;
    let manifest: serde_json::Value = serde_json::from_str(&raw)?;
    let stored = manifest
        .get("output_checksums")
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    let current = output_checksums(&req.step)?;
    Ok(stored == current && stored != serde_json::Value::Null)
}

fn write_crash_bundle(
    req: &ToolInvocationRequest,
    stderr_tail: &str,
    exit_code: i32,
    command: &str,
) -> Result<PathBuf> {
    let crash_path = req.context.stage_root.join("crash_provenance.json");
    let mut input_list = Vec::new();
    for artifact in &req.step.io.inputs {
        if artifact.path.exists() {
            let checksum = bijux_dna_infra::hash_file_sha256(&artifact.path)
                .unwrap_or_else(|_| "unknown".to_string());
            input_list.push(serde_json::json!({
                "name": artifact.name,
                "path": artifact.path,
                "sha256": checksum
            }));
        }
    }
    let stderr_last_lines = stderr_tail
        .lines()
        .rev()
        .take(50)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .map(std::string::ToString::to_string)
        .collect::<Vec<_>>();
    let env_subset = serde_json::json!({
        "LC_ALL": std::env::var("LC_ALL").ok(),
        "LANG": std::env::var("LANG").ok(),
        "TZ": std::env::var("TZ").ok(),
        "TMPDIR": std::env::var("TMPDIR").ok(),
        "HOME": std::env::var("HOME").ok(),
    });
    let payload = serde_json::json!({
        "schema_version": "bijux.stage_crash.v1",
        "run_id": req.context.run_id,
        "stage_id": req.context.stage_id,
        "tool_id": req.context.tool_id,
        "exit_code": exit_code,
        "command": command,
        "stderr_last_lines": stderr_last_lines,
        "inputs": input_list,
        "env_summary": env_subset,
        "tool_digest": req.step.image.digest,
    });
    bijux_dna_infra::atomic_write_json(&crash_path, &payload)?;
    Ok(crash_path)
}

/// # Errors
/// Returns an error if path contracts fail, tool execution fails, or artifacts cannot be written.
pub fn invoke_tool(req: &ToolInvocationRequest) -> Result<ToolInvocationResult> {
    enforce_path_contracts(req)?;
    require_pinned_digest(&req.step)?;
    crate::input_validation::validate_stage_inputs(&req.step)?;
    let policy = effective_runtime_policy(req);
    if network_policy_violation(&req.context.network_policy) {
        let _ = bijux_dna_infra::atomic_write_json(
            &req.context.stage_root.join("stage_result_status.json"),
            &serde_json::json!({
                "schema_version": "bijux.stage_result_status.v1",
                "status": "refused",
                "reason_code": "NETWORK_FORBIDDEN",
            }),
        );
        let _ = bijux_dna_runtime::recording::write_run_artifact_envelope(
            &req.context.stage_root,
            &req.context.stage_id,
            bijux_dna_runtime::recording::StageResultStatus::Refused,
            "NETWORK_FORBIDDEN",
        );
        bail!(
            "network policy violation: {} forbids network but BIJUX_ALLOW_NETWORK is enabled",
            req.context.stage_id
        );
    }
    bijux_dna_infra::ensure_dir(&req.context.stage_root)?;
    let effective_tmp_root = policy
        .temp_root
        .clone()
        .unwrap_or_else(|| req.context.tmp_root.clone());
    bijux_dna_infra::ensure_dir(&effective_tmp_root)?;
    let work_dir = req
        .context
        .output_root
        .join("work")
        .join(&req.context.run_id)
        .join(&req.context.stage_id)
        .join(req.step.step_id.as_str());
    bijux_dna_infra::ensure_dir(&work_dir)?;
    let cache_root = policy
        .cache_root
        .clone()
        .unwrap_or_else(|| req.context.output_root.join("cache"));
    bijux_dna_infra::ensure_dir(&cache_root)?;
    let home_dir = work_dir.join("home");
    bijux_dna_infra::ensure_dir(&home_dir)?;
    std::env::set_var(
        "LC_ALL",
        policy.deterministic_env.lc_all.as_deref().unwrap_or("C"),
    );
    std::env::set_var(
        "LANG",
        policy.deterministic_env.lang.as_deref().unwrap_or("C"),
    );
    std::env::set_var(
        "TZ",
        policy.deterministic_env.tz.as_deref().unwrap_or("UTC"),
    );
    if let Some(path) = policy.deterministic_env.path.as_deref() {
        std::env::set_var("PATH", path);
    }
    if let Some(umask) = policy.deterministic_env.umask.as_deref() {
        std::env::set_var("BIJUX_UMASK", umask);
    }
    std::env::set_var("BIJUX_STAGE_THREADS", policy.threads.to_string());
    if let Some(memory_hint_mb) = policy.memory_mb {
        std::env::set_var("BIJUX_STAGE_MEMORY_MB", memory_hint_mb.to_string());
    }
    if let Some(compression_threads) = policy.compression_threads {
        std::env::set_var("BIJUX_COMPRESSION_THREADS", compression_threads.to_string());
    }
    if let Some(seed) = req.context.seed {
        std::env::set_var("BIJUX_STAGE_SEED", seed.to_string());
    }
    std::env::set_var("TMPDIR", &effective_tmp_root);
    std::env::set_var("HOME", &home_dir);
    std::env::set_var("XDG_CACHE_HOME", &cache_root);
    std::env::set_var("BIJUX_CACHE_ROOT", &cache_root);
    for var in ["XDG_CACHE_HOME", "BIJUX_CACHE_ROOT"] {
        if let Ok(value) = std::env::var(var) {
            let path = PathBuf::from(value);
            if !path.starts_with(&cache_root) {
                bail!(
                    "cache policy violation: {var} must be under {}",
                    cache_root.display()
                );
            }
        }
    }

    let lock_root = req.context.output_root.join(".runtime_locks");
    bijux_dna_infra::ensure_dir(&lock_root)?;
    let is_heavy = policy
        .heavy_patterns
        .iter()
        .any(|pattern| stage_matches(pattern, &req.context.stage_id));
    let _heavy_lock = if is_heavy {
        acquire_slot_lock(&lock_root, "heavy", policy.max_local_heavy_parallel)?
    } else {
        None
    };
    let command_line = req.step.command.template.join(" ").to_ascii_lowercase();
    let _io_lock = if command_line.contains("bgzip") || command_line.contains("tabix") {
        acquire_slot_lock(&lock_root, "bgzip_tabix", policy.bgzip_tabix_max_parallel)?
    } else {
        None
    };

    if req.mode == ToolExecMode::PrintCommands
        || req.mode == ToolExecMode::DryRun
        || req.mode == ToolExecMode::DryRunExplain
    {
        let dry_path = req.context.stage_root.join("print_commands.txt");
        let image = req.step.image.image.clone();
        let digest = req.step.image.digest.clone().unwrap_or_default();
        let cmd = format!(
            "{} run {} {}",
            req.runner,
            image,
            req.step.command.template.join(" ")
        );
        bijux_dna_infra::atomic_write_bytes(&dry_path, cmd.as_bytes())?;
        if req.mode == ToolExecMode::DryRun || req.mode == ToolExecMode::DryRunExplain {
            let summary_path = req.context.stage_root.join("stage_human_summary.json");
            let stage_status_path = req.context.stage_root.join("stage_result_status.json");
            let summary = serde_json::json!({
                "schema_version": "bijux.stage_summary.v1",
                "stage_id": req.context.stage_id,
                "tool_id": req.context.tool_id,
                "status": "dry_run",
                "command": cmd,
                "tool_digest": digest,
            });
            bijux_dna_infra::atomic_write_json(&summary_path, &summary)?;
            bijux_dna_infra::atomic_write_json(
                &stage_status_path,
                &serde_json::json!({
                    "schema_version": "bijux.stage_result_status.v1",
                    "status": "ok",
                    "reason_code": "DRY_RUN"
                }),
            )?;
            let _ = bijux_dna_runtime::recording::write_run_artifact_envelope(
                &req.context.stage_root,
                &req.context.stage_id,
                bijux_dna_runtime::recording::StageResultStatus::Ok,
                "DRY_RUN",
            );
            if req.mode == ToolExecMode::DryRunExplain {
                let explain_path = req.context.stage_root.join("dry_run_explain.json");
                let payload = serde_json::json!({
                    "schema_version": "bijux.dry_run_explain.v1",
                    "stage_id": req.context.stage_id,
                    "tool_id": req.context.tool_id,
                    "runner": req.runner,
                    "command": req.step.command.template,
                    "io": req.step.io,
                    "resources": {
                        "threads": policy.threads,
                        "memory_mb": policy.memory_mb,
                        "compression_threads": policy.compression_threads,
                        "timeout_s": policy.timeout.map(|d| d.as_secs()),
                        "temp_root": effective_tmp_root,
                        "cache_root": cache_root,
                    },
                });
                bijux_dna_infra::atomic_write_json(&explain_path, &payload)?;
            }
        }
        return Ok(ToolInvocationResult {
            stage_result: StageResultV1 {
                run_id: req.context.run_id.clone(),
                exit_code: 0,
                runtime_s: 0.0,
                memory_mb: 0.0,
                outputs: vec![],
                metrics_path: None,
                stdout: String::new(),
                stderr: String::new(),
                command: cmd,
            },
            runtime_provenance_path: req.context.stage_root.join("runtime_provenance.json"),
            stage_manifest_path: req.context.stage_root.join("stage_manifest.json"),
            stdout_path: req.context.stage_root.join("logs").join("tool.stdout.log"),
            stderr_path: req.context.stage_root.join("logs").join("tool.stderr.log"),
            summary_path: req.context.stage_root.join("stage_human_summary.json"),
        });
    }
    if can_resume(req)? {
        let stage_status_path = req.context.stage_root.join("stage_result_status.json");
        bijux_dna_infra::atomic_write_json(
            &stage_status_path,
            &serde_json::json!({
                "schema_version": "bijux.stage_result_status.v1",
                "status": "skipped_cached",
                "reason_code": "CACHE_HIT"
            }),
        )?;
        let _ = bijux_dna_runtime::recording::write_run_artifact_envelope(
            &req.context.stage_root,
            &req.context.stage_id,
            bijux_dna_runtime::recording::StageResultStatus::SkippedCached,
            "CACHE_HIT",
        );
        return Ok(ToolInvocationResult {
            stage_result: StageResultV1 {
                run_id: req.context.run_id.clone(),
                exit_code: 0,
                runtime_s: 0.0,
                memory_mb: 0.0,
                outputs: req.step.io.outputs.iter().map(|a| a.path.clone()).collect(),
                metrics_path: None,
                stdout: String::new(),
                stderr: String::new(),
                command: "resume-skip".to_string(),
            },
            runtime_provenance_path: req.context.stage_root.join("runtime_provenance.json"),
            stage_manifest_path: req.context.stage_root.join("stage_manifest.json"),
            stdout_path: req.context.stage_root.join("logs").join("tool.stdout.log"),
            stderr_path: req.context.stage_root.join("logs").join("tool.stderr.log"),
            summary_path: req.context.stage_root.join("stage_human_summary.json"),
        });
    }

    let logs_dir = req.context.stage_root.join("logs");
    bijux_dna_infra::ensure_dir(&logs_dir)?;
    let started_at = chrono::Utc::now();
    let stage_log_path = req.context.stage_root.join("stage_events.jsonl");
    let start_event = serde_json::json!({
        "schema_version": "bijux.stage_events.v1",
        "event": "stage_start",
        "run_id": req.context.run_id,
        "stage_id": req.context.stage_id,
        "tool_id": req.context.tool_id,
        "chunk_id": "global",
        "timestamp": started_at,
    });
    bijux_dna_runtime::recording::append_jsonl_line(
        &stage_log_path,
        &serde_json::to_string(&start_event)?,
    )?;
    let chunk_start_event = serde_json::json!({
        "schema_version": "bijux.stage_events.v1",
        "event": "chunk_start",
        "run_id": req.context.run_id,
        "stage_id": req.context.stage_id,
        "tool_id": req.context.tool_id,
        "chunk_id": "global",
        "timestamp": started_at,
    });
    bijux_dna_runtime::recording::append_jsonl_line(
        &stage_log_path,
        &serde_json::to_string(&chunk_start_event)?,
    )?;
    let tool_event = serde_json::json!({
        "schema_version": "bijux.stage_events.v1",
        "event": "tool_invoked",
        "run_id": req.context.run_id,
        "stage_id": req.context.stage_id,
        "tool_id": req.context.tool_id,
        "timestamp": started_at,
        "chunk_id": "global",
    });
    bijux_dna_runtime::recording::append_jsonl_line(
        &stage_log_path,
        &serde_json::to_string(&tool_event)?,
    )?;

    let stage_result = execute_step(&req.step, req.runner, policy.timeout)?;
    validate_required_outputs(&req.step)?;
    let stage_metrics_path = req.context.stage_root.join("stage.metrics.json");
    let log_paths =
        write_execution_logs_bounded(&logs_dir, &stage_result.stdout, &stage_result.stderr)
            .context("write stage logs")?;
    let stdout_path = logs_dir.join("tool.stdout.log");
    let stderr_path = logs_dir.join("tool.stderr.log");
    let stderr_tail = std::fs::read_to_string(&stderr_path).unwrap_or_default();
    let stdout_tail = std::fs::read_to_string(&stdout_path).unwrap_or_default();

    let exit_taxonomy = classify_exit_code(stage_result.exit_code);
    if stage_result.exit_code != 0 {
        let _ = write_crash_bundle(
            req,
            &stderr_tail,
            stage_result.exit_code,
            &stage_result.command,
        );
        let _ = bijux_dna_infra::atomic_write_json(
            &req.context.stage_root.join("stage_result_status.json"),
            &serde_json::json!({
                "schema_version": "bijux.stage_result_status.v1",
                "status": "failed",
                "reason_code": format!("{exit_taxonomy:?}"),
            }),
        );
        let _ = bijux_dna_runtime::recording::write_run_artifact_envelope(
            &req.context.stage_root,
            &req.context.stage_id,
            bijux_dna_runtime::recording::StageResultStatus::Failed,
            &format!("{exit_taxonomy:?}"),
        );
        let message = format!(
            "tool {} failed with exit code {} ({exit_taxonomy:?})\nstdout_tail:\n{}\nstderr_tail:\n{}",
            req.context.tool_id,
            stage_result.exit_code,
            stdout_tail,
            stderr_tail
        );
        return Err(anyhow!(message));
    }

    let finished_at = chrono::Utc::now();
    let duration_ms = finished_at
        .signed_duration_since(started_at)
        .num_milliseconds()
        .max(0);
    let inferred_tool_version = infer_tool_version_from_image(&req.step.image.image);
    let runtime_provenance_path = req.context.stage_root.join("runtime_provenance.json");
    let env_summary = serde_json::json!({
        "hostname": std::env::var("HOSTNAME").ok(),
        "tz": std::env::var("TZ").ok(),
        "lc_all": std::env::var("LC_ALL").ok(),
    });
    let runtime_provenance = serde_json::json!({
        "schema_version": "bijux.runtime_provenance.v1",
        "run_id": req.context.run_id,
        "stage_id": req.context.stage_id,
        "tool_id": req.context.tool_id,
        "runner": req.runner.to_string(),
        "image": req.step.image.image,
        "tool_digest": req.step.image.digest,
        "tool_version": inferred_tool_version,
        "tool_version_probe_cmd": format!("{} --version", req.context.tool_id),
        "tool_version_probe_output": infer_version_line(&stage_result.stdout, &stage_result.stderr),
        "command": stage_result.command,
        "env_summary": env_summary,
        "started_at": started_at,
        "finished_at": finished_at,
        "duration_ms": duration_ms,
        "exit_code": stage_result.exit_code,
    });
    bijux_dna_infra::atomic_write_json(&runtime_provenance_path, &runtime_provenance)?;

    let stage_manifest_path = req.context.stage_root.join("stage_manifest.json");
    let stage_manifest = serde_json::json!({
        "schema_version": "bijux.stage_manifest.v1",
        "run_id": req.context.run_id,
        "stage_id": req.context.stage_id,
        "tool_id": req.context.tool_id,
        "sample_id": req.context.sample_id,
        "threads": policy.threads,
        "memory_hint_mb": policy.memory_mb,
        "compression_threads": policy.compression_threads,
        "seed": req.context.seed,
        "network_policy": req.context.network_policy,
        "inputs": req.step.io.inputs,
        "outputs": req.step.io.outputs,
        "runtime": {
            "runtime_s": stage_result.runtime_s,
            "duration_ms": duration_ms,
            "memory_mb": stage_result.memory_mb,
            "exit_code": stage_result.exit_code,
        },
        "logs": log_paths,
        "runtime_provenance": runtime_provenance_path,
    });
    let (output_checksums, _manifest_path) = ArtifactWriter::write_stage_outputs_and_manifest(
        &req.context.stage_root,
        &req.step.io.outputs,
        &stage_manifest_path,
        stage_manifest,
    )
    .context("write stage artifact checksums + stage manifest")?;

    let mut stage_metrics = serde_json::json!({
        "schema_version": "bijux.stage.metrics.v1",
        "stage_id": req.context.stage_id,
        "tool_id": req.context.tool_id,
        "runtime_s": stage_result.runtime_s,
        "wall_time_ms": duration_ms,
        "memory_mb": stage_result.memory_mb,
        "exit_code": stage_result.exit_code,
        "threads": policy.threads,
        "output_checksums": output_checksums,
    });
    if req.context.stage_id.starts_with("vcf.") {
        stage_metrics["records_in"] = serde_json::json!(0);
        stage_metrics["records_out"] = serde_json::json!(0);
    }
    if req.context.stage_id.starts_with("fastq.") {
        stage_metrics["reads_in"] = serde_json::json!(0);
        stage_metrics["reads_out"] = serde_json::json!(0);
    }
    MetricsWriter::write_stage_metrics(&stage_metrics_path, &req.context.stage_id, &stage_metrics)
        .context("write stage metrics with schema-backed required keys")?;

    let end_event = serde_json::json!({
        "schema_version": "bijux.stage_events.v1",
        "event": "chunk_end",
        "run_id": req.context.run_id,
        "stage_id": req.context.stage_id,
        "tool_id": req.context.tool_id,
        "chunk_id": "global",
        "timestamp": finished_at,
        "runtime_s": stage_result.runtime_s,
        "exit_code": stage_result.exit_code,
    });
    bijux_dna_runtime::recording::append_jsonl_line(
        &stage_log_path,
        &serde_json::to_string(&end_event)?,
    )?;
    let stage_end_event = serde_json::json!({
        "schema_version": "bijux.stage_events.v1",
        "event": "stage_end",
        "run_id": req.context.run_id,
        "stage_id": req.context.stage_id,
        "tool_id": req.context.tool_id,
        "chunk_id": "global",
        "timestamp": finished_at,
        "runtime_s": stage_result.runtime_s,
        "exit_code": stage_result.exit_code,
    });
    bijux_dna_runtime::recording::append_jsonl_line(
        &stage_log_path,
        &serde_json::to_string(&stage_end_event)?,
    )?;

    let summary_path = req.context.stage_root.join("stage_human_summary.json");
    let summary = serde_json::json!({
        "schema_version": "bijux.stage_summary.v1",
        "stage_id": req.context.stage_id,
        "tool_id": req.context.tool_id,
        "status": "ok",
        "runtime_s": stage_result.runtime_s,
        "exit_code": stage_result.exit_code,
        "stdout_path": stdout_path,
        "stderr_path": stderr_path,
    });
    bijux_dna_infra::atomic_write_json(&summary_path, &summary)?;
    bijux_dna_infra::atomic_write_json(
        &req.context.stage_root.join("stage_result_status.json"),
        &serde_json::json!({
            "schema_version": "bijux.stage_result_status.v1",
            "status": "ok",
            "reason_code": "SUCCESS",
        }),
    )?;
    let _ = bijux_dna_runtime::recording::write_run_artifact_envelope(
        &req.context.stage_root,
        &req.context.stage_id,
        bijux_dna_runtime::recording::StageResultStatus::Ok,
        "SUCCESS",
    );

    Ok(ToolInvocationResult {
        stage_result,
        runtime_provenance_path,
        stage_manifest_path,
        stdout_path,
        stderr_path,
        summary_path,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use bijux_dna_core::contract::{ArtifactRole, ArtifactSpec, StageIO, ToolConstraints};
    use bijux_dna_core::ids::{ArtifactId, StageId, StepId};
    use bijux_dna_core::prelude::{CommandSpecV1, ContainerImageRefV1};
    use tempfile::TempDir;

    #[test]
    fn tool_exec_bcftools_version_in_container() -> Result<()> {
        if std::env::var("BIJUX_E2E").is_err() {
            return Ok(());
        }
        let tmp = TempDir::new()?;
        let stage_root = tmp.path().join("artifacts").join("stage");
        let out_root = tmp.path().join("out");
        let in_root = tmp.path().join("in");
        bijux_dna_infra::ensure_dir(&stage_root)?;
        bijux_dna_infra::ensure_dir(&out_root)?;
        bijux_dna_infra::ensure_dir(&in_root)?;
        let out_file = out_root.join("bcftools.version.txt");
        let step = ExecutionStep {
            step_id: StepId::new("vcf.stats.bcftools_version"),
            stage_id: StageId::new("vcf.stats"),
            command: CommandSpecV1 {
                template: vec![
                    "sh".to_string(),
                    "-lc".to_string(),
                    "bcftools --version > /data/output/bcftools.version.txt".to_string(),
                ],
            },
            image: ContainerImageRefV1 {
                image: "quay.io/biocontainers/bcftools:1.20--h8b25389_0".to_string(),
                digest: Some(
                    "sha256:67f54df47f501f6ddef08e3b9ad89cf693952f9a89de0d74df6e39fce15f1ff6"
                        .to_string(),
                ),
            },
            resources: ToolConstraints::default(),
            io: StageIO {
                inputs: vec![],
                outputs: vec![ArtifactSpec::required(
                    ArtifactId::new("version"),
                    out_file.clone(),
                    ArtifactRole::Log,
                )],
            },
            out_dir: out_root.clone(),
            aux_images: std::collections::BTreeMap::new(),
            expected_artifact_ids: vec![],
            metrics_schema_ids: vec![],
        };
        let req = ToolInvocationRequest {
            step,
            runner: RuntimeKind::Docker,
            context: ToolContext {
                run_id: "run-e2e-tool-exec".to_string(),
                stage_id: "vcf.stats".to_string(),
                tool_id: "bcftools".to_string(),
                sample_id: None,
                stage_root: stage_root.clone(),
                input_root: in_root,
                output_root: out_root.clone(),
                tmp_root: stage_root.join("tmp"),
                threads: 1,
                memory_hint_mb: Some(512),
                compression_threads: Some(1),
                seed: Some(7),
                network_policy: NetworkPolicy::Forbid,
            },
            timeout: None,
            mode: ToolExecMode::Execute,
        };
        let result = ToolExec::invoke(&req)?;
        assert_eq!(result.stage_result.exit_code, 0);
        let version = std::fs::read_to_string(out_file)?;
        assert!(
            version.to_ascii_lowercase().contains("bcftools"),
            "unexpected bcftools --version output: {version}"
        );
        Ok(())
    }

    #[test]
    fn dry_run_explain_emits_plan_and_resource_details() -> Result<()> {
        let tmp = TempDir::new()?;
        let stage_root = tmp.path().join("artifacts").join("stage");
        let out_root = tmp.path().join("out");
        let in_root = tmp.path().join("in");
        bijux_dna_infra::ensure_dir(&stage_root)?;
        bijux_dna_infra::ensure_dir(&out_root)?;
        bijux_dna_infra::ensure_dir(&in_root)?;
        let step = ExecutionStep {
            step_id: StepId::new("vcf.qc.dry_run_explain"),
            stage_id: StageId::new("vcf.qc"),
            command: CommandSpecV1 {
                template: vec!["bcftools".to_string(), "--version".to_string()],
            },
            image: ContainerImageRefV1 {
                image: "quay.io/biocontainers/bcftools:1.20--h8b25389_0".to_string(),
                digest: Some(
                    "sha256:67f54df47f501f6ddef08e3b9ad89cf693952f9a89de0d74df6e39fce15f1ff6"
                        .to_string(),
                ),
            },
            resources: ToolConstraints::default(),
            io: StageIO {
                inputs: vec![],
                outputs: vec![],
            },
            out_dir: out_root.clone(),
            aux_images: std::collections::BTreeMap::new(),
            expected_artifact_ids: vec![],
            metrics_schema_ids: vec![],
        };
        let req = ToolInvocationRequest {
            step,
            runner: RuntimeKind::Docker,
            context: ToolContext {
                run_id: "run-dry-run-explain".to_string(),
                stage_id: "vcf.qc".to_string(),
                tool_id: "bcftools".to_string(),
                sample_id: None,
                stage_root: stage_root.clone(),
                input_root: in_root,
                output_root: out_root.clone(),
                tmp_root: stage_root.join("tmp"),
                threads: 1,
                memory_hint_mb: Some(256),
                compression_threads: Some(1),
                seed: Some(11),
                network_policy: NetworkPolicy::Forbid,
            },
            timeout: None,
            mode: ToolExecMode::DryRunExplain,
        };
        let result = ToolExec::invoke(&req)?;
        assert_eq!(result.stage_result.exit_code, 0);
        let explain_path = stage_root.join("dry_run_explain.json");
        assert!(explain_path.exists());
        let payload: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(explain_path)?)?;
        assert_eq!(
            payload
                .get("schema_version")
                .and_then(serde_json::Value::as_str),
            Some("bijux.dry_run_explain.v1")
        );
        Ok(())
    }
}
