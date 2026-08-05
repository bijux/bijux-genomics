//! Thin operator composition for crate-owned benchmark campaign planning and persistence.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context};

use super::{OpsCommandOutcome, Result, Workspace};

const USAGE: &str = "Usage:\n  cargo run -p bijux-dna-dev -- tooling run benchmark-campaign-cycle -- schema --output <path>\n  cargo run -p bijux-dna-dev -- tooling run benchmark-campaign-cycle -- execution-schema --output <path>\n  cargo run -p bijux-dna-dev -- tooling run benchmark-campaign-cycle -- plan --run-id <run-id> --campaign-id <campaign-id> --attempt-id <attempt-id> --observed-at-utc <timestamp> --output <path>\n  cargo run -p bijux-dna-dev -- tooling run benchmark-campaign-cycle -- initialize --plan <path> --benchmark-root <path>\n";
const MAX_PLAN_BYTES: usize = 16 * 1024 * 1024;

pub(in super::super) fn tooling_benchmark_campaign_cycle(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    let Some((operation, remaining)) = args.split_first() else {
        return Ok(OpsCommandOutcome::failure(USAGE));
    };
    let options = parse_options(remaining).context("parse benchmark campaign cycle options")?;
    match operation.as_str() {
        "schema" => schema(workspace, &options),
        "execution-schema" => execution_schema(workspace, &options),
        "plan" => plan(workspace, &options),
        "initialize" => initialize(workspace, &options),
        _ => Ok(OpsCommandOutcome::failure(USAGE)),
    }
}

fn execution_schema(
    workspace: &Workspace,
    options: &BTreeMap<String, String>,
) -> Result<OpsCommandOutcome> {
    require_exact_options(options, &["--output"])?;
    let output = resolve_path(workspace, required(options, "--output")?);
    let schema = serde_json::to_value(bijux_dna_bench::benchmark_execution_spec_schema())?;
    ensure_json_identity(&output, &schema)?;
    Ok(OpsCommandOutcome::success(format!(
        "published benchmark execution specification schema at {}\n",
        output.display()
    )))
}

fn schema(workspace: &Workspace, options: &BTreeMap<String, String>) -> Result<OpsCommandOutcome> {
    require_exact_options(options, &["--output"])?;
    let output = resolve_path(workspace, required(options, "--output")?);
    let schema = serde_json::to_value(bijux_dna_bench::benchmark_campaign_plan_schema())?;
    ensure_json_identity(&output, &schema)?;
    Ok(OpsCommandOutcome::success(format!(
        "published benchmark campaign plan schema at {}\n",
        output.display()
    )))
}

fn plan(workspace: &Workspace, options: &BTreeMap<String, String>) -> Result<OpsCommandOutcome> {
    require_exact_options(
        options,
        &["--attempt-id", "--campaign-id", "--observed-at-utc", "--output", "--run-id"],
    )?;
    let (repository_commit, campaign_registry_sha256) = source_identity(workspace)?;
    let (registry, _, _) = bijux_dna_bench::load_checked_in_benchmark_planning_contracts()?;
    let cycle = bijux_dna_bench::plan_stage_campaign_attempts(
        &registry,
        &repository_commit,
        &campaign_registry_sha256,
        required(options, "--run-id")?,
        required(options, "--campaign-id")?,
        required(options, "--attempt-id")?,
        required(options, "--observed-at-utc")?,
    )?;
    let output = resolve_path(workspace, required(options, "--output")?);
    ensure_json_identity(&output, &cycle)?;
    Ok(OpsCommandOutcome::success(format!(
        "planned {} benchmark attempts at {}\n",
        cycle.cycle.attempts.len(),
        output.display()
    )))
}

fn initialize(
    workspace: &Workspace,
    options: &BTreeMap<String, String>,
) -> Result<OpsCommandOutcome> {
    require_exact_options(options, &["--benchmark-root", "--plan"])?;
    let plan_path = resolve_path(workspace, required(options, "--plan")?);
    let bytes = bijux_dna_infra::read_to_end_bounded(&plan_path, MAX_PLAN_BYTES)
        .with_context(|| format!("read benchmark campaign plan {}", plan_path.display()))?;
    let cycle: bijux_dna_bench::BenchmarkCampaignPlan = serde_json::from_slice(&bytes)
        .with_context(|| format!("parse benchmark campaign plan {}", plan_path.display()))?;
    let (repository_commit, campaign_registry_sha256) = source_identity(workspace)?;
    bijux_dna_bench::verify_campaign_plan_source(
        &cycle,
        &repository_commit,
        &campaign_registry_sha256,
    )?;
    let benchmark_root = resolve_path(workspace, required(options, "--benchmark-root")?);
    let directories = bijux_dna_bench::ensure_stage_campaign_attempts(&benchmark_root, &cycle)?;
    Ok(OpsCommandOutcome::success(format!(
        "initialized {} benchmark attempts under {}\n",
        directories.len(),
        benchmark_root.display()
    )))
}

fn source_identity(workspace: &Workspace) -> Result<(String, String)> {
    let repository_commit =
        bijux_dna_environment::api::clean_repository_head_commit(&workspace.root)
            .context("require a clean source tree for benchmark campaign operation")?;
    let registry_path = bijux_dna_bench::bench_stage_campaigns_path();
    let campaign_registry_sha256 = bijux_dna_infra::hash_file_sha256(&registry_path)
        .with_context(|| format!("hash benchmark campaign registry {}", registry_path.display()))?;
    Ok((repository_commit, campaign_registry_sha256))
}

fn ensure_json_identity<T>(path: &Path, value: &T) -> Result<()>
where
    T: serde::Serialize + serde::de::DeserializeOwned + PartialEq,
{
    if path.exists() {
        let bytes = bijux_dna_infra::read_to_end_bounded(path, MAX_PLAN_BYTES)
            .with_context(|| format!("read existing benchmark campaign plan {}", path.display()))?;
        let existing: T = serde_json::from_slice(&bytes).with_context(|| {
            format!("parse existing benchmark campaign plan {}", path.display())
        })?;
        if existing != *value {
            bail!("benchmark campaign plan path is bound to different content: {}", path.display());
        }
        return Ok(());
    }
    bijux_dna_infra::atomic_create_json(path, value)
        .with_context(|| format!("write benchmark campaign plan {}", path.display()))
}

fn parse_options(args: &[String]) -> Result<BTreeMap<String, String>> {
    let mut options = BTreeMap::new();
    let mut chunks = args.chunks_exact(2);
    for pair in &mut chunks {
        if !pair[0].starts_with("--") || pair[1].starts_with("--") {
            bail!("benchmark campaign cycle options require --name value pairs\n{USAGE}");
        }
        if options.insert(pair[0].clone(), pair[1].clone()).is_some() {
            bail!("benchmark campaign cycle option is duplicated: {}", pair[0]);
        }
    }
    if !chunks.remainder().is_empty() {
        bail!("benchmark campaign cycle option lacks a value\n{USAGE}");
    }
    Ok(options)
}

fn require_exact_options(options: &BTreeMap<String, String>, expected: &[&str]) -> Result<()> {
    let actual = options.keys().map(String::as_str).collect::<Vec<_>>();
    if actual != expected {
        bail!("benchmark campaign cycle options differ from the operation contract\n{USAGE}");
    }
    Ok(())
}

fn required<'a>(options: &'a BTreeMap<String, String>, name: &str) -> Result<&'a str> {
    let value = options.get(name).with_context(|| format!("missing required option {name}"))?;
    if value.trim().is_empty() || value != value.trim() {
        bail!("benchmark campaign cycle option is empty or malformed: {name}");
    }
    Ok(value)
}

fn resolve_path(workspace: &Workspace, value: &str) -> PathBuf {
    let path = Path::new(value);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        workspace.root.join(path)
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_options, require_exact_options};

    #[test]
    fn option_parser_rejects_duplicates_and_incomplete_pairs() {
        assert!(parse_options(&["--run-id".into(), "run-2042".into()]).is_ok());
        assert!(parse_options(&["--run-id".into()]).is_err());
        assert!(parse_options(&[
            "--run-id".into(),
            "run-2042".into(),
            "--run-id".into(),
            "run-2043".into(),
        ])
        .is_err());
    }

    #[test]
    fn operation_contract_rejects_unknown_options() -> anyhow::Result<()> {
        let options = parse_options(&[
            "--plan".into(),
            "plan.json".into(),
            "--unknown".into(),
            "value".into(),
        ])?;
        assert!(require_exact_options(&options, &["--benchmark-root", "--plan"]).is_err());
        Ok(())
    }
}
