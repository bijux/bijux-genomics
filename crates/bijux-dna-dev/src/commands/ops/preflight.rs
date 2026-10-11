use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use anyhow::{anyhow, bail, ensure, Context, Result};
use chrono::{SecondsFormat, Utc};
use serde::Serialize;
use sha2::{Digest, Sha256};

use bijux_dna_containers::api::{
    ContainerGovernance as SiteCyclePolicy, ContainerPathPolicy as PathPolicy,
};
#[cfg(test)]
use bijux_dna_containers::api::{
    ContainerPreflightPolicy as PreflightPolicy, ContainerRuntimePolicy as RuntimePolicy,
    ContainerSitePolicy as SitePolicy, ContainerSlurmPolicy as SlurmPolicy,
};

use crate::model::container::NativeContainerCommandKey;
use crate::model::ops::OpsCommandOutcome;
use crate::runtime::process::ProcessRunner;
use crate::runtime::workspace::Workspace;

const PREFLIGHT_SCHEMA_V1: &str = "bijux.site_cycle_preflight.v1";
const DATA_INDEX_SCHEMAS: [&str; 2] = ["bijux.data_index.v1", "bijux.data_index.v2"];
const USAGE: &str = "Usage: cargo run -p bijux-dna-dev -- hpc run site-cycle-preflight -- --cycle <container-build|container-qualification|benchmark> --expected-commit <sha> --expected-catalog-sha256 <sha256> [--policy <path>] [--output-dir <path>] [--data-index <path> --data-index-sha256 <sha256>] [--image-inventory <path> --image-inventory-sha256 <sha256>]";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum CycleKind {
    ContainerBuild,
    ContainerQualification,
    Benchmark,
}

impl CycleKind {
    const fn requires_execution_authorities(self) -> bool {
        matches!(self, Self::ContainerQualification | Self::Benchmark)
    }
}

impl FromStr for CycleKind {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self> {
        match value {
            "container-build" => Ok(Self::ContainerBuild),
            "container-qualification" => Ok(Self::ContainerQualification),
            "benchmark" => Ok(Self::Benchmark),
            _ => bail!("unsupported cycle kind `{value}`"),
        }
    }
}

#[derive(Debug, Clone)]
struct AuthorityInput {
    path: PathBuf,
    expected_sha256: String,
}

#[derive(Debug, Clone)]
struct PreflightRequest {
    cycle: CycleKind,
    expected_commit: String,
    expected_catalog_sha256: String,
    policy_path: PathBuf,
    output_dir: Option<PathBuf>,
    data_index: Option<AuthorityInput>,
    image_inventory: Option<AuthorityInput>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RootObservation {
    canonical_path: Option<PathBuf>,
    is_directory: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AuthorityObservation {
    actual_sha256: Option<String>,
    schema_version: Option<String>,
    repository_commit: Option<String>,
    catalog_sha256: Option<String>,
    error: Option<String>,
}

fn authority_schema_matches(schema: &str, prefix: &str, exact_schemas: &[&str]) -> bool {
    if exact_schemas.is_empty() {
        schema.starts_with(prefix)
    } else {
        exact_schemas.contains(&schema)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProbeState {
    Passed,
    Failed,
}

impl ProbeState {
    const fn from_bool(passed: bool) -> Self {
        if passed {
            Self::Passed
        } else {
            Self::Failed
        }
    }

    const fn passed(self) -> bool {
        matches!(self, Self::Passed)
    }
}

#[derive(Debug, Clone)]
struct PreflightObservation {
    hostname: String,
    ssh_connection: ProbeState,
    source_root: Option<PathBuf>,
    source_commit: Option<String>,
    dirty_paths: Vec<String>,
    commands: BTreeMap<String, ProbeState>,
    clock_skew_seconds: Option<i64>,
    utc_probe: ProbeState,
    roots: BTreeMap<String, RootObservation>,
    available_bytes: Option<u64>,
    available_inodes: Option<u64>,
    write_probe: ProbeState,
    apptainer_version: Option<String>,
    machine_architecture: Option<String>,
    quota_status: ProbeState,
    quota_detail: String,
    slurm_associations: BTreeSet<(String, String)>,
    active_partitions: BTreeSet<String>,
    catalog_sha256: Option<String>,
    catalog_schema_version: Option<String>,
    contract_catalog_status: ProbeState,
    qualification_schema_status: ProbeState,
    contract_files: BTreeMap<String, ProbeState>,
    data_index: Option<AuthorityObservation>,
    image_inventory: Option<AuthorityObservation>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum CheckStatus {
    Passed,
    Failed,
    NotApplicable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct PreflightCheck {
    id: String,
    status: CheckStatus,
    mandatory: bool,
    detail: String,
}

#[derive(Debug, Clone, Serialize)]
struct PreflightReport {
    schema_version: &'static str,
    created_at_utc: String,
    site_id: String,
    cycle: CycleKind,
    expected_commit: String,
    expected_catalog_sha256: String,
    policy_sha256: String,
    status: CheckStatus,
    submission_allowed: bool,
    checks: Vec<PreflightCheck>,
}

pub(super) fn hpc_site_cycle_preflight(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    if args.iter().any(|arg| matches!(arg.as_str(), "--help" | "-h")) {
        return Ok(OpsCommandOutcome::success(format!("{USAGE}\n")));
    }
    let request = parse_request(workspace, args)?;
    let policy_raw = std::fs::read_to_string(&request.policy_path)
        .with_context(|| format!("read {}", request.policy_path.display()))?;
    let policy: SiteCyclePolicy = serde_yaml::from_str(&policy_raw)
        .with_context(|| format!("parse {}", request.policy_path.display()))?;
    validate_policy(&policy)?;

    let governed_output =
        policy.paths.finalization_root.join(&request.expected_commit).join("preflight");
    let output_dir = request.output_dir.clone().unwrap_or_else(|| governed_output.clone());
    ensure!(
        output_dir == governed_output,
        "preflight output must be the governed commit-addressed path {}",
        governed_output.display()
    );

    let observation = observe(workspace, &policy, &request, &output_dir);
    let policy_sha256 = sha256_bytes(policy_raw.as_bytes());
    let report = evaluate(&policy, &request, &observation, policy_sha256);
    bijux_dna_infra::ensure_dir(&output_dir)?;
    let json_path = output_dir.join("preflight.json");
    let markdown_path = output_dir.join("preflight.md");
    bijux_dna_infra::atomic_write_json(&json_path, &report)?;
    bijux_dna_infra::atomic_write_bytes(&markdown_path, render_markdown(&report).as_bytes())?;

    let failed = report
        .checks
        .iter()
        .filter(|check| check.mandatory && check.status == CheckStatus::Failed)
        .map(|check| check.id.as_str())
        .collect::<Vec<_>>();
    Ok(OpsCommandOutcome {
        exit_code: i32::from(!report.submission_allowed),
        stdout: format!("{}\n{}\n", json_path.display(), markdown_path.display()),
        stderr: if failed.is_empty() {
            String::new()
        } else {
            format!("preflight refused submission: {}\n", failed.join(", "))
        },
    })
}

fn parse_request(workspace: &Workspace, args: &[String]) -> Result<PreflightRequest> {
    let mut cycle: Option<CycleKind> = None;
    let mut expected_commit = None;
    let mut expected_catalog_sha256 = None;
    let mut policy_path = workspace.path("containers/contracts/sites/lunarc.yml");
    let mut output_dir = None;
    let mut data_index_path = None;
    let mut data_index_sha256 = None;
    let mut image_inventory_path = None;
    let mut image_inventory_sha256 = None;
    let mut index = 0;
    while index < args.len() {
        let flag = &args[index];
        let value = args.get(index + 1).ok_or_else(|| anyhow!("{flag} requires a value"))?;
        match flag.as_str() {
            "--cycle" => cycle = Some(value.parse()?),
            "--expected-commit" => expected_commit = Some(value.clone()),
            "--expected-catalog-sha256" => expected_catalog_sha256 = Some(value.clone()),
            "--policy" => policy_path = resolve_path(workspace, value),
            "--output-dir" => output_dir = Some(resolve_path(workspace, value)),
            "--data-index" => data_index_path = Some(resolve_path(workspace, value)),
            "--data-index-sha256" => data_index_sha256 = Some(value.clone()),
            "--image-inventory" => image_inventory_path = Some(resolve_path(workspace, value)),
            "--image-inventory-sha256" => image_inventory_sha256 = Some(value.clone()),
            _ => bail!("unknown argument `{flag}`\n{USAGE}"),
        }
        index += 2;
    }
    let cycle = cycle.ok_or_else(|| anyhow!("--cycle is required\n{USAGE}"))?;
    let expected_commit =
        expected_commit.ok_or_else(|| anyhow!("--expected-commit is required\n{USAGE}"))?;
    let expected_catalog_sha256 = expected_catalog_sha256
        .ok_or_else(|| anyhow!("--expected-catalog-sha256 is required\n{USAGE}"))?;
    ensure!(is_hex(&expected_commit, 40), "expected commit must be a 40-character hex SHA");
    ensure!(is_hex(&expected_catalog_sha256, 64), "expected catalog identity must be a SHA-256");
    let data_index = paired_authority(data_index_path, data_index_sha256, "data index")?;
    let image_inventory =
        paired_authority(image_inventory_path, image_inventory_sha256, "image inventory")?;
    if cycle.requires_execution_authorities() {
        ensure!(data_index.is_some(), "{cycle:?} requires a checksummed data index");
        ensure!(image_inventory.is_some(), "{cycle:?} requires a checksummed image inventory");
    }
    Ok(PreflightRequest {
        cycle,
        expected_commit,
        expected_catalog_sha256,
        policy_path,
        output_dir,
        data_index,
        image_inventory,
    })
}

fn paired_authority(
    path: Option<PathBuf>,
    expected_sha256: Option<String>,
    label: &str,
) -> Result<Option<AuthorityInput>> {
    match (path, expected_sha256) {
        (None, None) => Ok(None),
        (Some(path), Some(expected_sha256)) => {
            ensure!(is_hex(&expected_sha256, 64), "{label} identity must be a SHA-256");
            Ok(Some(AuthorityInput { path, expected_sha256 }))
        }
        _ => bail!("{label} path and SHA-256 must be supplied together"),
    }
}

fn resolve_path(workspace: &Workspace, value: &str) -> PathBuf {
    let path = PathBuf::from(value);
    if path.is_absolute() {
        path
    } else {
        workspace.root.join(path)
    }
}

fn validate_policy(policy: &SiteCyclePolicy) -> Result<()> {
    bijux_dna_containers::api::validate_container_governance(policy)
}

mod observation;

use observation::{is_hex, observe, sha256_bytes};
mod evaluation;

use evaluation::{evaluate, render_markdown};
fn required_roots(paths: &PathPolicy) -> [(&'static str, &Path); 7] {
    [
        ("study", &paths.study_root),
        ("containers", &paths.containers_root),
        ("data", &paths.data_root),
        ("results", &paths.results_root),
        ("finalization", &paths.finalization_root),
        ("active_repository", &paths.active_repository),
        ("frozen_repositories", &paths.frozen_repositories),
    ]
}

fn governed_study_paths(paths: &PathPolicy) -> [(&'static str, &Path); 4] {
    [
        ("containers", &paths.containers_root),
        ("data", &paths.data_root),
        ("results", &paths.results_root),
        ("finalization", &paths.finalization_root),
    ]
}
