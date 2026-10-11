//! Owner: bijux-dna-dev
//! Thin operator composition for source-addressed governed data catalog snapshots.

use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::{bail, Context};

use super::{OpsCommandOutcome, Result, Workspace};

const USAGE: &str = concat!(
    "Usage:\n",
    "  cargo run -p bijux-dna-dev -- tooling run benchmark-data-catalog-snapshots -- publish ",
    "--data-root <path> --preserved-at-utc <timestamp> --source-commit <sha> ",
    "[--previous-registry <path>]\n",
);

pub(in super::super) fn tooling_benchmark_data_catalog_snapshots(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    let Some((operation, remaining)) = args.split_first() else {
        return Ok(OpsCommandOutcome::failure(USAGE));
    };
    if operation != "publish" {
        return Ok(OpsCommandOutcome::failure(USAGE));
    }
    let options = parse_options(remaining)?;
    require_options(&options)?;
    let data_root = resolve_path(workspace, required(&options, "--data-root")?);
    let source_commit = required(&options, "--source-commit")?.to_string();
    let snapshots = bijux_dna_bench::publish_governed_data_catalog_snapshots(
        &bijux_dna_bench::GovernedDataCatalogSnapshotRequest {
            data_root: data_root.clone(),
            source_commit: source_commit.clone(),
            preserved_at_utc: required(&options, "--preserved-at-utc")?.to_string(),
            previous_registry_path: options
                .get("--previous-registry")
                .map(|path| resolve_path(workspace, path)),
        },
    )?;
    let registry = bijux_dna_bench::publish_governed_data_catalog_registry(
        &bijux_dna_bench::GovernedDataCatalogRegistryPublicationRequest {
            data_root,
            source_commit,
        },
    )?;
    Ok(OpsCommandOutcome::success(format!(
        "published governed data catalog snapshots: {} preserved, {} retained, {} records, registry {}, sha256 {}\n",
        snapshots.preserved_snapshot_count,
        snapshots.retained_snapshot_count,
        snapshots.registry.records.len(),
        registry.registry_path.display(),
        registry.registry_sha256,
    )))
}

fn parse_options(args: &[String]) -> Result<BTreeMap<String, String>> {
    if !args.len().is_multiple_of(2) {
        bail!("benchmark data catalog snapshot options require flag/value pairs");
    }
    let mut options = BTreeMap::new();
    for pair in args.chunks_exact(2) {
        if !pair[0].starts_with("--") || options.insert(pair[0].clone(), pair[1].clone()).is_some()
        {
            bail!("invalid or duplicate benchmark data catalog snapshot option");
        }
    }
    Ok(options)
}

fn require_options(options: &BTreeMap<String, String>) -> Result<()> {
    let actual = options.keys().map(String::as_str).collect::<Vec<_>>();
    let valid_without_prior = ["--data-root", "--preserved-at-utc", "--source-commit"];
    let valid_with_prior =
        ["--data-root", "--preserved-at-utc", "--previous-registry", "--source-commit"];
    if actual != valid_without_prior && actual != valid_with_prior {
        bail!("benchmark data catalog snapshot options differ from the operation contract");
    }
    Ok(())
}

fn required<'a>(options: &'a BTreeMap<String, String>, key: &str) -> Result<&'a str> {
    options
        .get(key)
        .map(String::as_str)
        .filter(|value| !value.is_empty())
        .with_context(|| format!("missing required option {key}"))
}

fn resolve_path(workspace: &Workspace, value: &str) -> PathBuf {
    let path = PathBuf::from(value);
    if path.is_absolute() {
        path
    } else {
        workspace.root.join(path)
    }
}
