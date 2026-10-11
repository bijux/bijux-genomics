//! Owner: bijux-dna-bench
//! Deterministic derivation and rendering of paper-facing container report publications.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    validate_container_report_portfolio_index, ContainerIndexedReport, ContainerIndexedReportFile,
    ContainerReportFreshness, ContainerReportPortfolioIndex, ContainerReportPortfolioSummary,
    CONTAINER_REPORT_PORTFOLIO_INDEX_SCHEMA_V1,
};
use bijux_dna_containers::api::{
    derive_container_run_index, render_container_run_index_json,
    render_container_run_index_markdown,
};

/// Derive the current and historical report portfolio from repository-owned files.
///
/// # Errors
/// Returns an error when the current run index drifted, a report cannot be hashed, or the derived
/// index violates identity and classification invariants.
pub fn derive_container_report_portfolio_index(
    repository_root: &Path,
) -> Result<ContainerReportPortfolioIndex> {
    let current_run_index = derive_container_run_index(
        repository_root,
        &repository_root.join("containers/contracts/qualification-runs.toml"),
    )?;
    let expected_json = render_container_run_index_json(&current_run_index)?;
    let expected_markdown = render_container_run_index_markdown(&current_run_index);
    let run_json_path = repository_root.join("containers/runs/RUN_INDEX.json");
    let run_markdown_path = repository_root.join("containers/runs/RUN_INDEX.md");
    if fs::read(&run_json_path).context("read container RUN_INDEX.json")?
        != expected_json.as_bytes()
        || fs::read(&run_markdown_path).context("read container RUN_INDEX.md")?
            != expected_markdown.as_bytes()
    {
        bail!("container report index requires a current permanent run index");
    }

    let mut reports = vec![ContainerIndexedReport {
        report_id: "permanent-run-index".to_string(),
        freshness: ContainerReportFreshness::Current,
        json: Some(indexed_file(
            repository_root,
            "containers/runs/RUN_INDEX.json",
        )?),
        markdown: Some(indexed_file(
            repository_root,
            "containers/runs/RUN_INDEX.md",
        )?),
        limitations: vec![
            "Current protocol identity does not imply current execution, science, security, or release evidence."
                .to_string(),
        ],
    }];

    let reports_root = repository_root.join("containers/reports");
    let mut historical: BTreeMap<String, (Option<String>, Option<String>)> = BTreeMap::new();
    for entry in fs::read_dir(&reports_root).context("read container reports directory")? {
        let entry = entry.context("read container report entry")?;
        if !entry.file_type().context("inspect container report entry")?.is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if matches!(name.as_str(), "README.md" | "REPORT_INDEX.json" | "REPORT_INDEX.md") {
            continue;
        }
        let (stem, slot) = if let Some(stem) = name.strip_suffix(".json") {
            (stem, 0_u8)
        } else if let Some(stem) = name.strip_suffix(".md") {
            (stem, 1_u8)
        } else {
            bail!("container reports contains an unindexed file: {name}");
        };
        let pair = historical.entry(stem.to_string()).or_default();
        let relative = format!("containers/reports/{name}");
        if slot == 0 {
            pair.0 = Some(relative);
        } else {
            pair.1 = Some(relative);
        }
    }
    for (report_id, (json_path, markdown_path)) in historical {
        reports.push(ContainerIndexedReport {
            report_id,
            freshness: ContainerReportFreshness::HistoricalUnvalidated,
            json: json_path
                .as_deref()
                .map(|path| indexed_file(repository_root, path))
                .transpose()?,
            markdown: markdown_path
                .as_deref()
                .map(|path| indexed_file(repository_root, path))
                .transpose()?,
            limitations: vec![
                "Not current: the complete source, catalog, stage/interface, data/truth, artifact, campaign, and report hash closure has not been re-admitted."
                    .to_string(),
            ],
        });
    }
    let current_count = reports
        .iter()
        .filter(|report| report.freshness == ContainerReportFreshness::Current)
        .count();
    let index = ContainerReportPortfolioIndex {
        schema_version: CONTAINER_REPORT_PORTFOLIO_INDEX_SCHEMA_V1.to_string(),
        summary: ContainerReportPortfolioSummary {
            report_count: reports.len(),
            current_count,
            historical_unvalidated_count: reports.len() - current_count,
        },
        reports,
    };
    validate_container_report_portfolio_index(&index)?;
    Ok(index)
}

/// Render deterministic report portfolio JSON.
///
/// # Errors
/// Returns an error when the model is invalid or serialization fails.
pub fn render_container_report_portfolio_index_json(
    index: &ContainerReportPortfolioIndex,
) -> Result<String> {
    validate_container_report_portfolio_index(index)?;
    Ok(serde_json::to_string_pretty(index)? + "\n")
}

#[must_use]
pub fn render_container_report_portfolio_index_markdown(
    index: &ContainerReportPortfolioIndex,
) -> String {
    let mut lines = vec![
        "<!-- GENERATED FILE - DO NOT EDIT -->".to_string(),
        "<!-- source: bijux-dna-bench::workflow::container_report_portfolio -->".to_string(),
        "# Container report index".to_string(),
        String::new(),
        "A file is current only after its complete dependency hash closure is re-admitted. Historical presence and successful jobs are not promoted by this index.".to_string(),
        String::new(),
        format!("- Indexed reports: **{}**", index.summary.report_count),
        format!("- Current reports: **{}**", index.summary.current_count),
        format!(
            "- Historical or unvalidated reports: **{}**",
            index.summary.historical_unvalidated_count
        ),
        String::new(),
        "| Report | Freshness | JSON | Markdown | Limitation |".to_string(),
        "|---|---|---|---|---|".to_string(),
    ];
    for report in &index.reports {
        let freshness = match report.freshness {
            ContainerReportFreshness::Current => "current",
            ContainerReportFreshness::HistoricalUnvalidated => "historical_unvalidated",
        };
        lines.push(format!(
            "| `{}` | `{freshness}` | {} | {} | {} |",
            report.report_id,
            report.json.as_ref().map_or("-", |file| file.path.as_str()),
            report.markdown.as_ref().map_or("-", |file| file.path.as_str()),
            report.limitations.join("<br>")
        ));
    }
    lines.push(String::new());
    lines.join("\n")
}

fn indexed_file(repository_root: &Path, relative: &str) -> Result<ContainerIndexedReportFile> {
    let path = repository_root.join(relative);
    Ok(ContainerIndexedReportFile {
        path: relative.to_string(),
        sha256: bijux_dna_infra::hash_file_sha256(&path)
            .with_context(|| format!("hash container report {relative}"))?,
    })
}
