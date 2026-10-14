//! Adapt benchmark taxonomy requests and present repository-owned judgments.
use crate::commands::cli::{parse, render};
use anyhow::{anyhow, Context, Result};
use bijux_dna_api::v1::api::bench::{
    publish_edna_taxonomy_output_judgment, TaxonomyObservedReportArg,
};
use std::path::{Path, PathBuf};

pub(crate) const DEFAULT_TAXONOMY_OUTPUT_JUDGMENT_PATH: &str =
    "artifacts/benchmark-smoke/readiness/local-ready/corpus-02-edna-taxonomy-judgment.json";

pub(crate) fn run_judge_taxonomy_output(
    args: &parse::BenchLocalJudgeTaxonomyOutputArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let manifest_path = if args.manifest.is_absolute() {
        args.manifest.clone()
    } else {
        repo_root.join(&args.manifest)
    };
    let output_path =
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_TAXONOMY_OUTPUT_JUDGMENT_PATH));
    let report = publish_edna_taxonomy_output_judgment(
        &repo_root,
        manifest_path,
        parse_report_args(&args.reports)?,
        output_path.clone(),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        let absolute_output_path =
            if output_path.is_absolute() { output_path } else { repo_root.join(output_path) };
        println!("{}", path_relative_to_repo(&repo_root, &absolute_output_path));
    }
    Ok(())
}

fn parse_report_args(values: &[String]) -> Result<Vec<TaxonomyObservedReportArg>> {
    if values.is_empty() {
        return Err(anyhow!(
            "taxonomy output judgment requires at least one `--report sample_id=path` mapping"
        ));
    }
    values
        .iter()
        .map(|value| {
            let (sample_id, report_path) = value.split_once('=').ok_or_else(|| {
                anyhow!(
                    "taxonomy output judgment report mapping must use `sample_id=path`, found `{value}`"
                )
            })?;
            if sample_id.trim().is_empty() || report_path.trim().is_empty() {
                return Err(anyhow!(
                    "taxonomy output judgment report mapping must use non-empty `sample_id=path`, found `{value}`"
                ));
            }
            Ok(TaxonomyObservedReportArg {
                sample_id: sample_id.trim().to_string(),
                report_path: PathBuf::from(report_path.trim()),
            })
        })
        .collect()
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).display().to_string()
}
