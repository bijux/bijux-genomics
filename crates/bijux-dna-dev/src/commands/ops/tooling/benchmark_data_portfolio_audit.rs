//! Owner: bijux-dna-dev
//! Thin operator composition for governed benchmark data-portfolio audits.

use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::{bail, Context};

use super::{OpsCommandOutcome, Result, Workspace};

const USAGE: &str = concat!(
    "Usage:\n",
    "  cargo run -p bijux-dna-dev -- tooling run benchmark-data-portfolio-audit -- schema --output <path>\n",
    "  cargo run -p bijux-dna-dev -- tooling run benchmark-data-portfolio-audit -- publish --data-index <path> --output-dir <path> --source-commit <sha> --study-root <path>\n",
);

pub(in super::super) fn tooling_benchmark_data_portfolio_audit(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    let Some((operation, remaining)) = args.split_first() else {
        return Ok(OpsCommandOutcome::failure(USAGE));
    };
    let options = parse_options(remaining)?;
    match operation.as_str() {
        "schema" => publish_schema(workspace, &options),
        "publish" => publish_audit(workspace, &options),
        _ => Ok(OpsCommandOutcome::failure(USAGE)),
    }
}

fn publish_schema(
    workspace: &Workspace,
    options: &BTreeMap<String, String>,
) -> Result<OpsCommandOutcome> {
    require_options(options, &["--output"])?;
    let output = resolve_path(workspace, required(options, "--output")?);
    if output.exists() || output.is_symlink() {
        bail!("refusing to replace data-portfolio audit schema: {}", output.display());
    }
    let mut bytes =
        serde_json::to_vec_pretty(&bijux_dna_bench::benchmark_data_portfolio_audit_schema())?;
    bytes.push(b'\n');
    bijux_dna_infra::atomic_create_bytes(&output, &bytes)
        .with_context(|| format!("publish data-portfolio audit schema {}", output.display()))?;
    Ok(OpsCommandOutcome::success(format!(
        "published benchmark data-portfolio audit schema to {}\n",
        output.display()
    )))
}

fn publish_audit(
    workspace: &Workspace,
    options: &BTreeMap<String, String>,
) -> Result<OpsCommandOutcome> {
    require_options(options, &["--data-index", "--output-dir", "--source-commit", "--study-root"])?;
    let output_dir = resolve_path(workspace, required(options, "--output-dir")?);
    std::fs::create_dir_all(&output_dir)
        .with_context(|| format!("create audit output directory {}", output_dir.display()))?;
    let audit = bijux_dna_bench::publish_benchmark_data_portfolio_audit(
        &bijux_dna_bench::BenchmarkDataPortfolioAuditPublicationRequest {
            audit: bijux_dna_bench::BenchmarkDataPortfolioAuditRequest {
                repository_root: workspace.root.clone(),
                study_root: resolve_path(workspace, required(options, "--study-root")?),
                data_index_path: resolve_path(workspace, required(options, "--data-index")?),
                source_commit: required(options, "--source-commit")?.to_string(),
            },
            json_path: output_dir.join("data-portfolio-audit.json"),
            markdown_path: output_dir.join("data-portfolio-audit.md"),
        },
    )?;
    Ok(OpsCommandOutcome::success(format!(
        "published benchmark data-portfolio audit: {} corpora, {} ready, {} unbound catalogs, {} contamination findings\n",
        audit.summary.corpus_count,
        audit.summary.publishable_data_ready_count,
        audit.summary.unbound_catalog_count,
        audit.summary.publishable_contamination_count,
    )))
}

fn parse_options(args: &[String]) -> Result<BTreeMap<String, String>> {
    if !args.len().is_multiple_of(2) {
        bail!("benchmark data-portfolio audit options require flag/value pairs");
    }
    let mut options = BTreeMap::new();
    for pair in args.chunks_exact(2) {
        if !pair[0].starts_with("--") || options.insert(pair[0].clone(), pair[1].clone()).is_some()
        {
            bail!("invalid or duplicate benchmark data-portfolio audit option");
        }
    }
    Ok(options)
}

fn require_options(options: &BTreeMap<String, String>, expected: &[&str]) -> Result<()> {
    let actual = options.keys().map(String::as_str).collect::<Vec<_>>();
    let mut expected = expected.to_vec();
    expected.sort_unstable();
    if actual != expected {
        bail!("benchmark data-portfolio audit options differ from the operation contract");
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
