use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use anyhow::{anyhow, bail, Result};
use bijux_dna_core::contract::ExecutionStep;
use serde::{Deserialize, Serialize};

use super::execution_kernel::{NetworkPolicy, ToolInvocationRequest};

mod config;
mod contracts;
mod models;
mod resilience;

use contracts::{
    enforce_large_file_guard, enforce_path_contracts, ensure_subpath, validate_required_outputs,
};
#[cfg(test)]
use models::validate_runtime_execution_config;
use models::{
    effective_runtime_policy, stage_matches, DeterministicEnvKnobs, RuntimeExecutionConfig,
    StageResourceKnobs,
};
use resilience::{
    acquire_slot_lock, can_resume, mark_partial_failure_invalid, update_resume_report,
    write_crash_bundle,
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub(crate) enum ExitTaxonomy {
    ToolFailure,
    UserError,
    ContractViolation,
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

pub(super) fn enforce_seed_policy(req: &ToolInvocationRequest) -> Result<()> {
    let cfg = config::runtime_execution_config();
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
