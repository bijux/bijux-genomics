use std::collections::{BTreeSet, HashMap, HashSet};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use anyhow::{anyhow, Context, Result};
use bijux_dna_api::v1::api::env::{
    available_runners, cache_dir, docker_image_exists, resolve_image, run_shell_capture,
    run_smoke_script, run_smoke_script_batch, PlatformSpec, RuntimeKind, ToolImageSpec,
};
use regex::Regex;
use serde::Serialize;
use sha2::{Digest, Sha256};

/// # Errors
/// Returns an error if image resolution fails.
pub fn print_env_images<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
) -> Result<()> {
    let mut entries: Vec<_> = catalog.iter().collect();
    entries.sort_by_key(|(name, _)| *name);
    for (name, spec) in entries {
        let resolved = resolve_image(spec, platform)?;
        let digest = spec.digest.as_deref().unwrap_or("no digest");
        println!("{name}: {} ({digest})", resolved.full_name);
    }
    Ok(())
}

/// # Errors
/// Returns an error if registry cannot be read.
pub fn print_env_registry_list(registry_path: &Path) -> Result<()> {
    let raw = std::fs::read_to_string(registry_path)
        .with_context(|| format!("read {}", registry_path.display()))?;
    println!("tool\thas_docker\thas_apptainer\thas_smoke\tpinned");
    for row in parse_tools_registry_rows(&raw)? {
        let has_docker = row.runtimes.iter().any(|v| v == "docker") && row.dockerfile.is_some();
        let has_apptainer =
            row.runtimes.iter().any(|v| v == "apptainer") && row.apptainer_def.is_some();
        let has_smoke = row.version_cmd.is_some();
        let pinned = row
            .pinned_commit
            .as_deref()
            .is_some_and(|s| s.len() == 40 && s.chars().all(|c| c.is_ascii_hexdigit()));
        println!("{}\t{has_docker}\t{has_apptainer}\t{has_smoke}\t{pinned}", row.id);
    }
    Ok(())
}

/// # Errors
/// Returns an error if smoke script execution fails.
pub fn run_env_smoke(runtime: &str, tool: &str) -> Result<()> {
    let tools = selected_tools(tool);
    if runtime == "apptainer" {
        let registry_path = current_registry_path()?;
        ensure_apptainer_tools(
            &registry_path,
            &resolved_apptainer_hpc_root()?,
            &tools,
            true,
            false,
        )?;
        return Ok(());
    }
    if tools.len() == 1 {
        return run_smoke_script(runtime, &tools[0]);
    }
    run_env_with_tools(runtime, &tools, "contract")
}

fn normalize_stage_id(stage: &str) -> String {
    if stage.contains('.') {
        stage.to_string()
    } else {
        format!("fastq.{stage}")
    }
}

fn parse_registry(path: &Path) -> Result<String> {
    let raw = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    Ok(raw)
}

/// # Errors
/// Returns an error if registry cannot be parsed.
pub fn registry_tools_for_stage(
    registry_path: &Path,
    stage: &str,
    scenario: Option<&str>,
    kind: &str,
) -> Result<Vec<String>> {
    let parsed = parse_registry(registry_path)?;
    let stage_id = normalize_stage_id(stage);
    let Some(stage_entry) =
        parse_stage_registry_rows(&parsed)?.into_iter().find(|entry| entry.id == stage_id)
    else {
        return Err(anyhow!("stage not found in registry: {stage_id}"));
    };

    let mut result = match kind {
        "all" => {
            let mut all = Vec::new();
            all.extend(stage_entry.primary_tools);
            all.extend(stage_entry.optional_alternatives);
            all.extend(stage_entry.validation_tools);
            all.extend(stage_entry.reporting_tools);
            all
        }
        "primary" => stage_entry.primary_tools,
        "optional" => stage_entry.optional_alternatives,
        "validation" => stage_entry.validation_tools,
        "reporting" => stage_entry.reporting_tools,
        "benchmark" => benchmark_tools_for_stage(&stage_id, scenario)?,
        other => {
            return Err(anyhow!(
                "unsupported registry tool kind `{other}`; expected one of all, primary, optional, validation, reporting, benchmark"
            ))
        }
    };
    result.sort();
    result.dedup();
    Ok(result)
}

fn benchmark_tools_for_stage(stage_id: &str, scenario: Option<&str>) -> Result<Vec<String>> {
    bijux_dna_api::v1::api::fastq::benchmark_tools_for_stage(stage_id, scenario)
}

/// # Errors
/// Returns an error if stage cannot be resolved.
pub fn run_env_smoke_for_stage(registry_path: &Path, runtime: &str, stage: &str) -> Result<()> {
    let tools = registry_tools_for_stage(registry_path, stage, None, "all")?;
    if tools.is_empty() {
        return Err(anyhow!("no tools found for stage {stage}"));
    }
    if runtime == "apptainer" {
        ensure_apptainer_tools(
            registry_path,
            &resolved_apptainer_hpc_root()?,
            &tools,
            true,
            false,
        )?;
        return Ok(());
    }
    run_env_with_tools(runtime, &tools, "contract")
}

/// # Errors
/// Returns an error if prep script execution fails.
pub fn run_env_prep(
    registry_path: &Path,
    runtime: &str,
    tool: Option<&str>,
    stage: Option<&str>,
) -> Result<()> {
    if let Some(tool) = tool {
        let tools = selected_tools(tool);
        if runtime == "apptainer" {
            ensure_apptainer_tools(
                registry_path,
                &resolved_apptainer_hpc_root()?,
                &tools,
                false,
                false,
            )?;
            return Ok(());
        }
        return run_env_with_tools(runtime, &tools, "version");
    }
    if let Some(stage) = stage {
        let tools = registry_tools_for_stage(registry_path, stage, None, "all")?;
        if tools.is_empty() {
            return Err(anyhow!("no tools found for stage {stage}"));
        }
        if runtime == "apptainer" {
            ensure_apptainer_tools(
                registry_path,
                &resolved_apptainer_hpc_root()?,
                &tools,
                false,
                false,
            )?;
            return Ok(());
        }
        return run_env_with_tools(runtime, &tools, "version");
    }
    run_env_with_tools(runtime, &[], "version")
}

fn run_env_with_tools(runtime: &str, tools: &[String], smoke_level: &str) -> Result<()> {
    run_smoke_script_batch(runtime, tools, smoke_level)
}

fn selected_tools(value: &str) -> Vec<String> {
    let tools = value
        .split(',')
        .map(str::trim)
        .filter(|tool| !tool.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    if tools.is_empty() {
        vec![value.trim().to_string()]
    } else {
        tools
    }
}

fn current_registry_path() -> Result<PathBuf> {
    let cwd = std::env::current_dir().context("resolve current working directory")?;
    Ok(bijux_dna_infra::configs_file(&cwd, "ci/registry/tool_registry.toml"))
}

fn resolved_apptainer_hpc_root() -> Result<PathBuf> {
    if let Ok(config) = crate::commands::hpc::load_hpc_config() {
        return Ok(config.resolve_paths().root);
    }
    Err(anyhow!("unable to resolve Apptainer HPC root: declare BIJUX_HPC_CONFIG"))
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod env_registry_query_tests {
    use super::{registry_tools_for_stage, selected_tools};

    fn registry_path() -> std::path::PathBuf {
        crate::commands::support::workspace_root::resolve_repo_root()
            .expect("repo root")
            .join("configs/ci/registry/tool_registry.toml")
    }

    #[test]
    fn registry_benchmark_tools_follow_trim_fairness_cohort() {
        let tools =
            registry_tools_for_stage(&registry_path(), "fastq.trim_reads", None, "benchmark")
                .expect("trim benchmark tools");
        assert_eq!(
            tools,
            vec![
                "adapterremoval",
                "alientrimmer",
                "atropos",
                "bbduk",
                "cutadapt",
                "fastp",
                "fastx_clipper",
                "leehom",
                "prinseq",
                "seqkit",
                "skewer",
                "trim_galore",
                "trimmomatic",
            ]
        );
    }

    #[test]
    fn registry_benchmark_tools_follow_polyg_fairness_cohort() {
        let tools =
            registry_tools_for_stage(&registry_path(), "fastq.trim_polyg_tails", None, "benchmark")
                .expect("polyg benchmark tools");
        assert_eq!(tools, vec!["bbduk", "fastp"]);
    }

    #[test]
    fn registry_rejects_unknown_tool_kind() {
        let error = registry_tools_for_stage(&registry_path(), "fastq.trim_reads", None, "bogus")
            .expect_err("unknown kind should fail");
        assert!(error.to_string().contains("unsupported registry tool kind"));
    }

    #[test]
    fn registry_benchmark_tools_follow_explicit_profile_read_length_cohort() {
        let tools = registry_tools_for_stage(
            &registry_path(),
            "fastq.profile_read_lengths",
            Some("read_length_fairness"),
            "benchmark",
        )
        .expect("read-length benchmark tools");
        assert_eq!(tools, vec!["seqkit_stats"]);
    }

    #[test]
    fn selected_tools_supports_csv_batches() {
        assert_eq!(selected_tools("fastp, seqkit ,cutadapt"), vec!["fastp", "seqkit", "cutadapt"]);
    }

    #[test]
    fn selected_tools_preserves_single_tool_input() {
        assert_eq!(selected_tools("fastp"), vec!["fastp"]);
    }
}

#[derive(Default, Serialize)]
struct RegistryRow {
    id: String,
    status: String,
    domain: Option<String>,
    domains: Vec<String>,
    stage_ids: Vec<String>,
    bindings: Vec<String>,
    tool_role: Option<String>,
    version: Option<String>,
    upstream: Option<String>,
    runtimes: Vec<String>,
    dockerfile: Option<String>,
    apptainer_def: Option<String>,
    version_cmd: Option<String>,
    help_cmd: Option<String>,
    expected_bin: Option<String>,
    pinned_commit: Option<String>,
    container_ref: Option<String>,
    expected_version_regex: Option<String>,
    healthcheck_cmd: Option<String>,
    smoke_version_cmd: Option<String>,
    smoke_help_cmd: Option<String>,
    smoke_require_help: Option<bool>,
    smoke_probes: Vec<String>,
    java_heap_mb: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct EnsureImagesReport {
    pub schema_version: &'static str,
    pub domain: String,
    pub stages: Vec<String>,
    pub tools: Vec<EnsureToolReport>,
    pub built: usize,
    pub reused: usize,
    pub quick_smoked: usize,
    pub failed: usize,
}

#[derive(Debug, Serialize)]
pub struct EnsureToolReport {
    pub tool_id: String,
    pub stage_id: String,
    pub sif_path: String,
    pub registry_digest: String,
    pub sif_sha256: String,
    pub built: bool,
    pub quick_smoked: bool,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct SifInventoryReport {
    pub schema_version: &'static str,
    pub containers_dir: String,
    pub entries: Vec<SifInventoryEntry>,
}

#[derive(Debug, Serialize)]
pub struct SifInventoryEntry {
    pub tool_id: String,
    pub sif_path: String,
    pub sha256: String,
    pub smoke_manifest_path: Option<String>,
    pub smoke_status: Option<String>,
}

#[derive(Debug, Serialize)]
struct SmokeProbeResult {
    command: String,
    applied_command: String,
    ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    output_sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    output_first_line: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct SmokeManifest {
    schema_version: &'static str,
    tool_id: String,
    stage_id: String,
    status: String,
    registry_digest: String,
    sif_sha256: String,
    version_cmd: String,
    help_cmd: String,
    version: String,
    version_output_first_line: String,
    help_ok: bool,
    quick_smoke: bool,
    probe_commands: Vec<String>,
    #[serde(default)]
    probe_results: Vec<SmokeProbeResult>,
    java_heap_mb: Option<u64>,
    upstream: String,
    image_build_timestamp_unix_s: u64,
    checked_at_unix_s: u64,
}

fn parse_tools_registry_rows(raw: &str) -> Result<Vec<RegistryRow>> {
    let mut rows = Vec::new();
    let mut current: Option<RegistryRow> = None;

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if trimmed == "[[tools]]" {
            if let Some(row) = current.take() {
                rows.push(row);
            }
            current = Some(RegistryRow::default());
            continue;
        }
        let Some(tool_entry) = current.as_mut() else {
            continue;
        };
        if let Some(value) = parse_toml_string(trimmed, "id") {
            tool_entry.id = value;
        } else if let Some(value) = parse_toml_string(trimmed, "status") {
            tool_entry.status = value;
        } else if let Some(value) = parse_toml_string(trimmed, "domain") {
            tool_entry.domain = Some(value);
        } else if let Some(values) = parse_toml_array(trimmed, "domains") {
            tool_entry.domains = values;
        } else if let Some(values) = parse_toml_array(trimmed, "stage_ids") {
            tool_entry.stage_ids = values;
        } else if let Some(values) = parse_toml_array(trimmed, "bindings") {
            tool_entry.bindings = values;
        } else if let Some(value) = parse_toml_string(trimmed, "tool_role") {
            tool_entry.tool_role = Some(value);
        } else if let Some(value) = parse_toml_string(trimmed, "version") {
            tool_entry.version = Some(value);
        } else if let Some(value) = parse_toml_string(trimmed, "upstream") {
            tool_entry.upstream = Some(value);
        } else if let Some(value) = parse_toml_string(trimmed, "dockerfile") {
            tool_entry.dockerfile = Some(value);
        } else if let Some(value) = parse_toml_string(trimmed, "apptainer_def") {
            tool_entry.apptainer_def = Some(value);
        } else if let Some(value) = parse_toml_string(trimmed, "version_cmd") {
            tool_entry.version_cmd = Some(value);
        } else if let Some(value) = parse_toml_string(trimmed, "help_cmd") {
            tool_entry.help_cmd = Some(value);
        } else if let Some(value) = parse_toml_string(trimmed, "expected_bin") {
            tool_entry.expected_bin = Some(value);
        } else if let Some(value) = parse_toml_string(trimmed, "pinned_commit") {
            tool_entry.pinned_commit = Some(value);
        } else if let Some(value) = parse_toml_string(trimmed, "container_ref") {
            tool_entry.container_ref = Some(value);
        } else if let Some(value) = parse_toml_string(trimmed, "expected_version_regex") {
            tool_entry.expected_version_regex = Some(value);
        } else if let Some(value) = parse_toml_string(trimmed, "healthcheck_cmd") {
            tool_entry.healthcheck_cmd = Some(value);
        } else if let Some(value) = parse_toml_string(trimmed, "smoke_version_cmd") {
            tool_entry.smoke_version_cmd = Some(value);
        } else if let Some(value) = parse_toml_string(trimmed, "smoke_help_cmd") {
            tool_entry.smoke_help_cmd = Some(value);
        } else if let Some(value) = parse_toml_bool(trimmed, "smoke_require_help") {
            tool_entry.smoke_require_help = Some(value);
        } else if let Some(values) = parse_toml_array(trimmed, "smoke_probes") {
            tool_entry.smoke_probes = values;
        } else if let Some(value) = parse_toml_u64(trimmed, "java_heap_mb") {
            tool_entry.java_heap_mb = Some(value);
        } else if let Some(values) = parse_toml_array(trimmed, "runtimes") {
            tool_entry.runtimes = values;
        }
    }
    if let Some(row) = current {
        rows.push(row);
    }
    if rows.is_empty() {
        return Err(anyhow!("missing [[tools]] entries"));
    }
    Ok(rows)
}
