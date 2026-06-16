use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};

use super::executable_resolution::{collect_executable_resolution_rows, ExecutableResolutionRow};
use super::tool_execution_modes::load_runtime_probe_with_source;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_VERSION_PROBES_PATH: &str =
    "benchmarks/readiness/tools/version-probes.json";
const VERSION_PROBES_SCHEMA_VERSION: &str = "bijux.bench.readiness.version_probes.v1";
const VERSION_PROBE_STATUS_READY: &str = "ready";
const VERSION_PROBE_STATUS_UNAVAILABLE: &str = "unavailable_with_reason";
const VERSION_PARSER_KIND_FIRST_DOTTED_NUMERIC_TOKEN: &str = "first_dotted_numeric_token";
const GOVERNED_TOOL_REGISTRY_PATHS: [&str; 2] =
    ["configs/ci/registry/tool_registry.toml", "configs/ci/registry/tool_registry_vcf.toml"];

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VersionProbeRow {
    pub(crate) tool_id: String,
    pub(crate) domains: Vec<String>,
    pub(crate) active_stage_ids: Vec<String>,
    pub(crate) resolution_kind: String,
    pub(crate) resolution_target: String,
    pub(crate) version_probe_status: String,
    pub(crate) command_entrypoint: Option<String>,
    pub(crate) version_cmd: Option<String>,
    pub(crate) help_cmd: Option<String>,
    pub(crate) version_parser_kind: Option<String>,
    pub(crate) expected_version_regex: Option<String>,
    pub(crate) expected_bin: Option<String>,
    pub(crate) declared_version: Option<String>,
    pub(crate) runtime_probe_paths: Vec<String>,
    pub(crate) registry_paths: Vec<String>,
    pub(crate) unavailable_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VersionProbesReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) row_count: usize,
    pub(crate) ready_count: usize,
    pub(crate) unavailable_count: usize,
    pub(crate) parser_kind_counts: BTreeMap<String, usize>,
    pub(crate) rows: Vec<VersionProbeRow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RuntimeVersionProbeSignature {
    version_cmd: Option<String>,
    help_cmd: Option<String>,
}

#[derive(Debug, Clone)]
struct RuntimeVersionProbeContract {
    signature: RuntimeVersionProbeSignature,
    runtime_probe_paths: Vec<String>,
}

#[derive(Debug, Clone)]
struct RegistryVersionContract {
    tool_id: String,
    domains: Vec<String>,
    stage_ids: Vec<String>,
    version_cmd: Option<String>,
    help_cmd: Option<String>,
    expected_bin: Option<String>,
    expected_version_regex: Option<String>,
    declared_version: Option<String>,
    registry_paths: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct RegistryVersionContractFile {
    tools: Vec<RegistryVersionContractSourceRow>,
}

#[derive(Debug, Deserialize)]
struct RegistryVersionContractSourceRow {
    id: String,
    #[serde(default)]
    domain: Option<String>,
    #[serde(default)]
    domains: Vec<String>,
    #[serde(default)]
    stage_ids: Vec<String>,
    #[serde(default)]
    version_cmd: Option<String>,
    #[serde(default)]
    help_cmd: Option<String>,
    #[serde(default)]
    expected_bin: Option<String>,
    #[serde(default)]
    expected_version_regex: Option<String>,
    #[serde(default, rename = "version")]
    declared_version: Option<String>,
}

pub(crate) fn run_render_version_probes(
    args: &parse::BenchReadinessRenderVersionProbesArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_version_probes(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_VERSION_PROBES_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_version_probes(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<VersionProbesReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let rows = collect_version_probe_rows(repo_root)?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let rendered = serde_json::to_vec_pretty(&VersionProbesReport::from_rows(
        path_relative_to_repo(repo_root, &output_path),
        rows.clone(),
    ))?;
    fs::write(&output_path, rendered)
        .with_context(|| format!("write {}", output_path.display()))?;

    Ok(VersionProbesReport::from_rows(path_relative_to_repo(repo_root, &output_path), rows))
}

impl VersionProbesReport {
    fn from_rows(output_path: String, rows: Vec<VersionProbeRow>) -> Self {
        let ready_count = rows
            .iter()
            .filter(|row| row.version_probe_status == VERSION_PROBE_STATUS_READY)
            .count();
        let unavailable_count = rows
            .iter()
            .filter(|row| row.version_probe_status == VERSION_PROBE_STATUS_UNAVAILABLE)
            .count();
        let mut parser_kind_counts = BTreeMap::<String, usize>::new();
        for row in &rows {
            if let Some(parser_kind) = &row.version_parser_kind {
                *parser_kind_counts.entry(parser_kind.clone()).or_default() += 1;
            }
        }

        Self {
            schema_version: VERSION_PROBES_SCHEMA_VERSION,
            output_path,
            row_count: rows.len(),
            ready_count,
            unavailable_count,
            parser_kind_counts,
            rows,
        }
    }
}

fn collect_version_probe_rows(repo_root: &Path) -> Result<Vec<VersionProbeRow>> {
    let executable_rows = collect_executable_resolution_rows(repo_root)?;
    let registry_contracts = load_registry_version_contracts(repo_root)?;
    let mut rows = executable_rows
        .iter()
        .map(|row| resolve_version_probe_row(repo_root, row, &registry_contracts))
        .collect::<Result<Vec<_>>>()?;
    rows.sort_by(|left, right| left.tool_id.cmp(&right.tool_id));
    ensure_version_probe_contract(&executable_rows, &rows)?;
    Ok(rows)
}

fn resolve_version_probe_row(
    repo_root: &Path,
    executable_row: &ExecutableResolutionRow,
    registry_contracts: &[RegistryVersionContract],
) -> Result<VersionProbeRow> {
    let runtime_contract = collect_runtime_version_probe_contract(repo_root, executable_row)?;
    let registry_contract = match_registry_version_contract(executable_row, registry_contracts)?;

    let version_cmd = reconcile_optional_string(
        &executable_row.tool_id,
        "version_cmd",
        runtime_contract.signature.version_cmd.as_deref(),
        registry_contract.version_cmd.as_deref(),
    )?;
    let help_cmd = reconcile_optional_string(
        &executable_row.tool_id,
        "help_cmd",
        runtime_contract.signature.help_cmd.as_deref(),
        registry_contract.help_cmd.as_deref(),
    )?;

    let expected_version_regex = registry_contract
        .expected_version_regex
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);
    if let Some(regex_text) = &expected_version_regex {
        Regex::new(regex_text).with_context(|| {
            format!(
                "tool `{}` declares an invalid expected_version_regex `{regex_text}`",
                executable_row.tool_id
            )
        })?;
    }

    let is_unavailable = executable_row.resolution_kind == "unavailable_with_reason";
    if !is_unavailable && version_cmd.is_none() {
        return Err(anyhow!(
            "retained tool `{}` is missing a governed version command",
            executable_row.tool_id
        ));
    }
    if !is_unavailable && expected_version_regex.is_none() {
        return Err(anyhow!(
            "retained tool `{}` is missing a governed version parser regex",
            executable_row.tool_id
        ));
    }

    Ok(VersionProbeRow {
        tool_id: executable_row.tool_id.clone(),
        domains: executable_row.domains.clone(),
        active_stage_ids: executable_row.active_stage_ids.clone(),
        resolution_kind: executable_row.resolution_kind.clone(),
        resolution_target: executable_row.resolution_target.clone(),
        version_probe_status: if is_unavailable {
            VERSION_PROBE_STATUS_UNAVAILABLE.to_string()
        } else {
            VERSION_PROBE_STATUS_READY.to_string()
        },
        command_entrypoint: executable_row.command_entrypoint.clone(),
        version_cmd,
        help_cmd,
        version_parser_kind: if is_unavailable {
            None
        } else {
            Some(VERSION_PARSER_KIND_FIRST_DOTTED_NUMERIC_TOKEN.to_string())
        },
        expected_version_regex,
        expected_bin: registry_contract.expected_bin.clone(),
        declared_version: registry_contract.declared_version.clone(),
        runtime_probe_paths: runtime_contract.runtime_probe_paths,
        registry_paths: registry_contract.registry_paths.clone(),
        unavailable_reason: if is_unavailable {
            executable_row.unavailable_reason.clone()
        } else {
            None
        },
    })
}

fn collect_runtime_version_probe_contract(
    repo_root: &Path,
    executable_row: &ExecutableResolutionRow,
) -> Result<RuntimeVersionProbeContract> {
    let mut runtime_probe_paths = Vec::<String>::new();
    let mut resolved_signature = None::<RuntimeVersionProbeSignature>;

    for domain in &executable_row.domains {
        let loaded = load_runtime_probe_with_source(repo_root, domain, &executable_row.tool_id)
            .with_context(|| {
                format!(
                    "load runtime probe for retained tool `{}` in domain `{}`",
                    executable_row.tool_id, domain
                )
            })?;
        let signature = RuntimeVersionProbeSignature {
            version_cmd: loaded.probe.version_cmd(),
            help_cmd: loaded.probe.help_cmd(),
        };
        let probe_path = path_relative_to_repo(repo_root, &loaded.path);
        if !runtime_probe_paths.iter().any(|existing| existing == &probe_path) {
            runtime_probe_paths.push(probe_path);
        }

        if let Some(previous) = &resolved_signature {
            if previous != &signature {
                return Err(anyhow!(
                    "retained tool `{}` declares inconsistent version probe commands across governed runtime probes",
                    executable_row.tool_id
                ));
            }
        } else {
            resolved_signature = Some(signature);
        }
    }

    let signature = resolved_signature.ok_or_else(|| {
        anyhow!(
            "retained tool `{}` must resolve at least one governed runtime probe",
            executable_row.tool_id
        )
    })?;
    runtime_probe_paths.sort();

    Ok(RuntimeVersionProbeContract { signature, runtime_probe_paths })
}

fn load_registry_version_contracts(repo_root: &Path) -> Result<Vec<RegistryVersionContract>> {
    let mut contracts = Vec::<RegistryVersionContract>::new();

    for relative_path in GOVERNED_TOOL_REGISTRY_PATHS {
        let path = repo_root.join(relative_path);
        let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        let parsed: RegistryVersionContractFile =
            toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
        let relative_path = path_relative_to_repo(repo_root, &path);

        for row in parsed.tools {
            let tool_id = row.id.trim().to_string();
            if tool_id.is_empty() {
                return Err(anyhow!(
                    "governed tool registry `{}` contains an empty tool id",
                    relative_path
                ));
            }

            let mut domains = row
                .domains
                .into_iter()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
                .collect::<Vec<_>>();
            if let Some(domain) = row
                .domain
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
            {
                domains.push(domain);
            }
            domains.sort();
            domains.dedup();

            let candidate = RegistryVersionContract {
                tool_id: tool_id.clone(),
                domains,
                stage_ids: row
                    .stage_ids
                    .into_iter()
                    .map(|value| value.trim().to_string())
                    .filter(|value| !value.is_empty())
                    .collect(),
                version_cmd: normalize_optional_string(row.version_cmd),
                help_cmd: normalize_optional_string(row.help_cmd),
                expected_bin: normalize_optional_string(row.expected_bin),
                expected_version_regex: normalize_optional_string(row.expected_version_regex),
                declared_version: normalize_optional_string(row.declared_version),
                registry_paths: vec![relative_path.clone()],
            };
            contracts.push(candidate);
        }
    }

    Ok(contracts)
}

fn match_registry_version_contract(
    executable_row: &ExecutableResolutionRow,
    registry_contracts: &[RegistryVersionContract],
) -> Result<RegistryVersionContract> {
    let executable_domains =
        executable_row.domains.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let executable_stage_ids =
        executable_row.active_stage_ids.iter().map(String::as_str).collect::<BTreeSet<_>>();

    let candidates = registry_contracts
        .iter()
        .filter(|contract| contract.tool_id == executable_row.tool_id)
        .filter(|contract| {
            contract
                .domains
                .iter()
                .map(String::as_str)
                .any(|domain| executable_domains.contains(domain))
                || contract
                    .stage_ids
                    .iter()
                    .map(String::as_str)
                    .any(|stage_id| executable_stage_ids.contains(stage_id))
        })
        .cloned()
        .collect::<Vec<_>>();

    let mut candidates = if candidates.is_empty() {
        registry_contracts
            .iter()
            .filter(|contract| contract.tool_id == executable_row.tool_id)
            .cloned()
            .collect::<Vec<_>>()
    } else {
        candidates
    };

    let Some(mut merged) = candidates.pop() else {
        return Err(anyhow!(
            "retained tool `{}` is missing from the governed tool registries",
            executable_row.tool_id
        ));
    };
    for candidate in candidates {
        merge_registry_contract(&mut merged, candidate)?;
    }
    Ok(merged)
}

fn merge_registry_contract(
    existing: &mut RegistryVersionContract,
    candidate: RegistryVersionContract,
) -> Result<()> {
    merge_registry_field(
        &existing.tool_id,
        "version_cmd",
        &mut existing.version_cmd,
        candidate.version_cmd,
    )?;
    merge_registry_field(
        &existing.tool_id,
        "help_cmd",
        &mut existing.help_cmd,
        candidate.help_cmd,
    )?;
    merge_registry_field(
        &existing.tool_id,
        "expected_bin",
        &mut existing.expected_bin,
        candidate.expected_bin,
    )?;
    merge_registry_field(
        &existing.tool_id,
        "expected_version_regex",
        &mut existing.expected_version_regex,
        candidate.expected_version_regex,
    )?;
    merge_registry_field(
        &existing.tool_id,
        "declared_version",
        &mut existing.declared_version,
        candidate.declared_version,
    )?;
    existing.registry_paths.extend(candidate.registry_paths);
    existing.registry_paths.sort();
    existing.registry_paths.dedup();
    Ok(())
}

fn merge_registry_field(
    tool_id: &str,
    field_name: &str,
    target: &mut Option<String>,
    candidate: Option<String>,
) -> Result<()> {
    match (target.as_deref(), candidate.as_deref()) {
        (Some(existing), Some(next)) if existing != next => Err(anyhow!(
            "retained tool `{tool_id}` declares conflicting registry values for `{field_name}`"
        )),
        (None, Some(_)) => {
            *target = candidate;
            Ok(())
        }
        _ => Ok(()),
    }
}

fn reconcile_optional_string(
    tool_id: &str,
    field_name: &str,
    runtime_value: Option<&str>,
    registry_value: Option<&str>,
) -> Result<Option<String>> {
    match (runtime_value, registry_value) {
        (Some(runtime), Some(registry)) if runtime.trim() != registry.trim() => Err(anyhow!(
            "retained tool `{tool_id}` declares conflicting `{field_name}` values between runtime probe and governed registry"
        )),
        (Some(runtime), _) => Ok(Some(runtime.trim().to_string())),
        (None, Some(registry)) => Ok(Some(registry.trim().to_string())),
        (None, None) => Ok(None),
    }
}

fn ensure_version_probe_contract(
    executable_rows: &[ExecutableResolutionRow],
    rows: &[VersionProbeRow],
) -> Result<()> {
    if rows.len() != executable_rows.len() {
        return Err(anyhow!("version probe coverage must keep exactly one row per retained tool"));
    }

    let executable_tool_ids =
        executable_rows.iter().map(|row| row.tool_id.as_str()).collect::<BTreeSet<_>>();
    let version_probe_tool_ids =
        rows.iter().map(|row| row.tool_id.as_str()).collect::<BTreeSet<_>>();
    if executable_tool_ids != version_probe_tool_ids {
        return Err(anyhow!(
            "version probe coverage drifted from the governed executable resolution inventory"
        ));
    }

    for row in rows {
        if row.tool_id.trim().is_empty()
            || row.domains.is_empty()
            || row.active_stage_ids.is_empty()
            || row.resolution_kind.trim().is_empty()
            || row.runtime_probe_paths.is_empty()
            || row.registry_paths.is_empty()
        {
            return Err(anyhow!("version probe row `{}` is missing a required field", row.tool_id));
        }
        match row.version_probe_status.as_str() {
            VERSION_PROBE_STATUS_READY => {
                if row.version_cmd.as_deref().map(str::trim).is_none_or(str::is_empty)
                    || row
                        .expected_version_regex
                        .as_deref()
                        .map(str::trim)
                        .is_none_or(str::is_empty)
                    || row.version_parser_kind.as_deref().map(str::trim).is_none_or(str::is_empty)
                    || row.unavailable_reason.is_some()
                {
                    return Err(anyhow!(
                        "ready version probe row `{}` must keep command, parser, regex, and no unavailable reason",
                        row.tool_id
                    ));
                }
            }
            VERSION_PROBE_STATUS_UNAVAILABLE => {
                if row.unavailable_reason.as_deref().map(str::trim).is_none_or(str::is_empty) {
                    return Err(anyhow!(
                        "unavailable version probe row `{}` must keep an explicit unavailable reason",
                        row.tool_id
                    ));
                }
            }
            other => {
                return Err(anyhow!(
                    "tool `{}` resolved to unsupported version_probe_status `{other}`",
                    row.tool_id
                ));
            }
        }
    }

    Ok(())
}

fn normalize_optional_string(value: Option<String>) -> Option<String> {
    value.as_deref().map(str::trim).filter(|value| !value.is_empty()).map(ToOwned::to_owned)
}

fn repo_relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}
