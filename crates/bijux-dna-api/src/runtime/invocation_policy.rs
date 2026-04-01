use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::Duration;

use anyhow::{anyhow, bail, Context, Result};
use bijux_dna_core::contract::ExecutionStep;
use serde::Serialize;

use super::execution_kernel::{NetworkPolicy, ToolInvocationRequest};
use crate::writers::ArtifactWriter;

mod config;
mod models;

use models::{
    effective_runtime_policy, stage_matches, DeterministicEnvKnobs, EffectiveRuntimePolicy,
    RuntimeExecutionConfig, StageResourceKnobs,
};
#[cfg(test)]
use models::validate_runtime_execution_config;

pub(super) fn acquire_slot_lock(
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

pub(super) fn canonicalize_existing(path: &Path) -> Result<PathBuf> {
    if path.exists() {
        return path
            .canonicalize()
            .with_context(|| format!("canonicalize {}", path.display()));
    }
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .context("resolve cwd for relative path contracts")?
            .join(path)
    };

    let mut missing_components = Vec::new();
    let mut ancestor = absolute.as_path();
    while !ancestor.exists() {
        let name = ancestor.file_name().ok_or_else(|| {
            anyhow!(
                "cannot resolve non-existent path without existing ancestor: {}",
                absolute.display()
            )
        })?;
        missing_components.push(name.to_os_string());
        ancestor = ancestor.parent().ok_or_else(|| {
            anyhow!(
                "cannot resolve parent for non-existent path: {}",
                absolute.display()
            )
        })?;
    }

    let mut resolved = ancestor
        .canonicalize()
        .with_context(|| format!("canonicalize ancestor {}", ancestor.display()))?;
    for component in missing_components.iter().rev() {
        resolved.push(component);
    }
    Ok(resolved)
}

pub(super) fn ensure_subpath(path: &Path, root: &Path, label: &str) -> Result<()> {
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

pub(super) fn enforce_path_contracts(req: &ToolInvocationRequest) -> Result<()> {
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

pub(super) fn require_pinned_digest(step: &ExecutionStep) -> Result<()> {
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

pub(super) fn classify_exit_code(exit_code: i32) -> ExitTaxonomy {
    match exit_code {
        2 | 64..=78 => ExitTaxonomy::UserError,
        126 | 127 => ExitTaxonomy::ContractViolation,
        _ => ExitTaxonomy::ToolFailure,
    }
}

pub(super) fn infer_tool_version_from_image(image: &str) -> String {
    let without_digest = image.split('@').next().unwrap_or(image);
    if let Some((_, tag)) = without_digest.rsplit_once(':') {
        if !tag.is_empty() && tag != "latest" {
            return tag.to_string();
        }
    }
    String::new()
}

pub(super) fn infer_version_line(stdout: &str, stderr: &str) -> Option<String> {
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

pub(super) fn network_policy_violation(policy: &NetworkPolicy) -> bool {
    matches!(policy, NetworkPolicy::Forbid)
        && std::env::var("BIJUX_ALLOW_NETWORK")
            .ok()
            .is_some_and(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
}

pub(super) fn validate_required_outputs(step: &ExecutionStep) -> Result<()> {
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

pub(super) fn output_checksums(step: &ExecutionStep) -> Result<serde_json::Value> {
    ArtifactWriter::write_output_checksums(&step.out_dir, &step.io.outputs)
}

pub(super) fn can_resume(req: &ToolInvocationRequest) -> Result<bool> {
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

pub(super) fn update_resume_report(
    stage_root: &Path,
    stage_id: &str,
    status: &str,
    reason: &str,
) -> Result<()> {
    let report_path = stage_root.join("resume_report.json");
    let mut cached = Vec::<serde_json::Value>::new();
    let mut recomputed = Vec::<serde_json::Value>::new();
    if report_path.exists() {
        let existing: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&report_path)?)?;
        cached = existing
            .get("cached")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .ok_or_else(|| anyhow!("resume report missing declared `cached` array"))?;
        recomputed = existing
            .get("recomputed")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .ok_or_else(|| anyhow!("resume report missing declared `recomputed` array"))?;
    }
    let entry = serde_json::json!({
        "stage_id": stage_id,
        "reason": reason,
        "timestamp": chrono::Utc::now(),
    });
    if status == "cached" {
        cached.push(entry);
    } else {
        recomputed.push(entry);
    }
    let payload = serde_json::json!({
        "schema_version": "bijux.resume_report.v1",
        "cached": cached,
        "recomputed": recomputed,
        "summary": {
            "cached_count": cached.len(),
            "recomputed_count": recomputed.len(),
        }
    });
    bijux_dna_infra::atomic_write_json(&report_path, &payload)?;
    Ok(())
}

pub(super) fn mark_partial_failure_invalid(
    stage_root: &Path,
    outputs: &[bijux_dna_core::contract::ArtifactSpec],
) -> Result<()> {
    let invalid_dir = stage_root.join("invalid_artifacts");
    bijux_dna_infra::ensure_dir(&invalid_dir)?;
    let mut invalid = Vec::<serde_json::Value>::new();
    for artifact in outputs {
        if artifact.path.exists() {
            let marker = invalid_dir.join(format!("{}.invalid", artifact.name));
            bijux_dna_infra::atomic_write_bytes(
                &marker,
                format!("invalidated_after_failure={}\n", chrono::Utc::now()).as_bytes(),
            )?;
            invalid.push(serde_json::json!({
                "name": artifact.name,
                "path": artifact.path,
                "marker": marker,
                "status": "invalid_after_failure",
            }));
        }
    }
    let payload = serde_json::json!({
        "schema_version": "bijux.partial_failure_cleanup.v1",
        "policy": "keep_intermediates_mark_invalid",
        "items": invalid,
    });
    bijux_dna_infra::atomic_write_json(&stage_root.join("partial_failure_cleanup.json"), &payload)?;
    Ok(())
}

pub(super) fn enforce_seed_policy(req: &ToolInvocationRequest) -> Result<()> {
    let cfg = runtime_execution_config();
    let seed_required_patterns = cfg.per_stage.as_ref().map_or_else(
        || vec!["vcf.phasing".to_string(), "vcf.impute".to_string()],
        |_| vec!["vcf.phasing".to_string(), "vcf.impute".to_string()],
    );
    let requires_seed = seed_required_patterns
        .iter()
        .any(|pattern| stage_matches(pattern, &req.context.stage_id));
    let random_allowed = std::env::var("BIJUX_RANDOM_ALLOWED")
        .ok()
        .is_some_and(|v| matches!(v.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"));
    if requires_seed && req.context.seed.is_none() && !random_allowed {
        bail!(
            "deterministic randomness policy violation: stage {} requires configured seed or BIJUX_RANDOM_ALLOWED=1",
            req.context.stage_id
        );
    }
    Ok(())
}

pub(super) fn rewrite_long_region_args(
    step: &ExecutionStep,
    work_dir: &Path,
    threshold: usize,
) -> Result<ExecutionStep> {
    let joined = step.command.template.join(" ");
    if joined.len() <= threshold {
        return Ok(step.clone());
    }
    let mut rewritten = step.clone();
    let mut args = rewritten.command.template.clone();
    let mut idx = 0usize;
    while idx + 1 < args.len() {
        let flag = args[idx].as_str();
        let value = args[idx + 1].clone();
        if (flag == "--regions" || flag == "--targets")
            && value.len() > threshold / 3
            && value.contains(',')
        {
            let file = work_dir.join(if flag == "--regions" {
                "regions.list"
            } else {
                "targets.list"
            });
            let body = value
                .split(',')
                .map(str::trim)
                .collect::<Vec<_>>()
                .join("\n");
            bijux_dna_infra::atomic_write_bytes(&file, format!("{body}\n").as_bytes())?;
            args[idx] = if flag == "--regions" {
                "--regions-file".to_string()
            } else {
                "--targets-file".to_string()
            };
            args[idx + 1] = file.display().to_string();
        }
        idx += 1;
    }
    rewritten.command.template = args;
    Ok(rewritten)
}

pub(super) fn enforce_large_file_guard(
    output_root: &Path,
    outputs: &[bijux_dna_core::contract::ArtifactSpec],
) -> Result<()> {
    let max_bytes: u64 = std::env::var("BIJUX_MAX_UNEXPECTED_FILE_BYTES")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(5 * 1024 * 1024 * 1024);
    let allowed = outputs
        .iter()
        .map(|a| canonicalize_existing(&a.path))
        .collect::<Result<Vec<_>>>()?;
    let mut violations = Vec::new();
    for entry in walkdir::WalkDir::new(output_root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        let path = canonicalize_existing(entry.path())?;
        if allowed.iter().any(|allowed_path| allowed_path == &path) {
            continue;
        }
        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
        if size > max_bytes {
            violations.push(format!("{} ({} bytes)", path.display(), size));
        }
    }
    if !violations.is_empty() {
        bail!(
            "large-file guard violation: unexpected files outside contract exceeded limit: {}",
            violations.join(", ")
        );
    }
    Ok(())
}

pub(super) fn write_crash_bundle(
    req: &ToolInvocationRequest,
    stderr_tail: Option<&str>,
    exit_code: i32,
    command: &str,
) -> Result<PathBuf> {
    let crash_path = req.context.stage_root.join("crash_provenance.json");
    let mut input_list = Vec::new();
    for artifact in &req.step.io.inputs {
        if artifact.path.exists() {
            let checksum = bijux_dna_infra::hash_file_sha256(&artifact.path).ok();
            input_list.push(serde_json::json!({
                "name": artifact.name,
                "path": artifact.path,
                "sha256": checksum
            }));
        }
    }
    let stderr_last_lines = stderr_tail
        .map(|stderr_tail| {
            stderr_tail
                .lines()
                .rev()
                .take(50)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_execution_config_rejects_invalid_ranges_and_tmp_roots() {
        let cfg = RuntimeExecutionConfig {
            default_threads: Some(0),
            default_memory_mb: Some(1),
            default_compression_threads: Some(1),
            default_timeout_s: Some(1),
            default_temp_root: Some(format!("/{}", ["tmp", "bijux"].join("/"))),
            heavy_stage_patterns: Some(vec!["bam.align".to_string()]),
            max_local_heavy_parallel: Some(1),
            bgzip_tabix_max_parallel: Some(1),
            cache_root: Some("artifacts/runtime-cache".to_string()),
            deterministic_env: None,
            per_stage: None,
        };
        let Err(err) = validate_runtime_execution_config(&cfg) else {
            panic!("config with zero threads and /tmp root must be rejected");
        };
        assert!(
            err.to_string().contains("default_threads must be > 0")
                || err.to_string().contains("cannot point to system tmp")
        );
    }

    #[test]
    fn runtime_execution_config_accepts_valid_policy() -> Result<()> {
        let mut per_stage = std::collections::BTreeMap::new();
        per_stage.insert(
            "vcf.impute".to_string(),
            StageResourceKnobs {
                threads: Some(4),
                memory_mb: Some(2048),
                compression_threads: Some(2),
                timeout_s: Some(3600),
                temp_root: Some("artifacts/runtime-tmp/impute".to_string()),
            },
        );
        let cfg = RuntimeExecutionConfig {
            default_threads: Some(1),
            default_memory_mb: Some(512),
            default_compression_threads: Some(1),
            default_timeout_s: Some(600),
            default_temp_root: Some("artifacts/runtime-tmp".to_string()),
            heavy_stage_patterns: Some(vec!["vcf.impute".to_string()]),
            max_local_heavy_parallel: Some(1),
            bgzip_tabix_max_parallel: Some(1),
            cache_root: Some("artifacts/runtime-cache".to_string()),
            deterministic_env: Some(DeterministicEnvKnobs {
                lc_all: Some("C".to_string()),
                lang: Some("C".to_string()),
                tz: Some("UTC".to_string()),
                umask: Some("027".to_string()),
                path: None,
            }),
            per_stage: Some(per_stage),
        };
        validate_runtime_execution_config(&cfg)
    }

    #[test]
    fn ensure_subpath_accepts_missing_child_under_canonicalized_root() -> Result<()> {
        let tmp = tempfile::TempDir::new()?;
        let output_root = tmp.path().join("out");
        bijux_dna_infra::ensure_dir(&output_root)?;
        let tmp_root = output_root.join("stage").join("tmp");

        ensure_subpath(&tmp_root, &output_root, "tmp_root")
    }
}
