//! Deterministic paper-facing index over container report publications.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};
use schemars::{JsonSchema, Schema};
use serde::{Deserialize, Serialize};

pub const CONTAINER_REPORT_PORTFOLIO_INDEX_SCHEMA_V1: &str =
    "bijux.container.report_portfolio_index.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ContainerReportFreshness {
    Current,
    HistoricalUnvalidated,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContainerIndexedReportFile {
    pub path: String,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContainerIndexedReport {
    pub report_id: String,
    pub freshness: ContainerReportFreshness,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub json: Option<ContainerIndexedReportFile>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub markdown: Option<ContainerIndexedReportFile>,
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContainerReportPortfolioSummary {
    pub report_count: usize,
    pub current_count: usize,
    pub historical_unvalidated_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContainerReportPortfolioIndex {
    pub schema_version: String,
    pub summary: ContainerReportPortfolioSummary,
    pub reports: Vec<ContainerIndexedReport>,
}

#[must_use]
pub fn container_report_portfolio_index_schema() -> Schema {
    schemars::schema_for!(ContainerReportPortfolioIndex)
}

/// Derive the current and historical report portfolio from repository-owned files.
///
/// # Errors
/// Returns an error when the current run index drifted, a report cannot be hashed, or the derived
/// index violates identity and classification invariants.
pub fn derive_container_report_portfolio_index(
    repository_root: &Path,
) -> Result<ContainerReportPortfolioIndex> {
    let current_run_index = super::derive_container_run_index(
        repository_root,
        &repository_root.join("containers/contracts/qualification-runs.toml"),
    )?;
    let expected_json = super::render_container_run_index_json(&current_run_index)?;
    let expected_markdown = super::render_container_run_index_markdown(&current_run_index);
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
        json: Some(indexed_file(repository_root, "containers/runs/RUN_INDEX.json")?),
        markdown: Some(indexed_file(repository_root, "containers/runs/RUN_INDEX.md")?),
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

/// Validate report identity, file pairs, digests, counts, and conservative freshness policy.
///
/// # Errors
/// Returns an error when a report is duplicated, incomplete, or promoted without current proof.
pub fn validate_container_report_portfolio_index(
    index: &ContainerReportPortfolioIndex,
) -> Result<()> {
    if index.schema_version != CONTAINER_REPORT_PORTFOLIO_INDEX_SCHEMA_V1
        || index.summary.report_count != index.reports.len()
    {
        bail!("container report portfolio index summary is invalid");
    }
    let current = index
        .reports
        .iter()
        .filter(|report| report.freshness == ContainerReportFreshness::Current)
        .count();
    if index.summary.current_count != current
        || index.summary.historical_unvalidated_count != index.reports.len() - current
    {
        bail!("container report portfolio freshness counts drifted");
    }
    let mut report_ids = BTreeSet::new();
    let mut paths = BTreeSet::new();
    for report in &index.reports {
        if report.report_id.trim().is_empty()
            || !report_ids.insert(report.report_id.as_str())
            || (report.json.is_none() && report.markdown.is_none())
            || report.limitations.is_empty()
        {
            bail!("container report portfolio entry is incomplete");
        }
        if report.freshness == ContainerReportFreshness::Current
            && report.report_id != "permanent-run-index"
        {
            bail!("container report was promoted without complete current hash closure");
        }
        for file in [report.json.as_ref(), report.markdown.as_ref()].into_iter().flatten() {
            if file.sha256.len() != 64
                || !file.sha256.bytes().all(|byte| byte.is_ascii_hexdigit())
                || !paths.insert(file.path.as_str())
            {
                bail!("container report file identity is invalid");
            }
        }
    }
    Ok(())
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
        "<!-- source: bijux-dna-containers::evidence::report_registry -->".to_string(),
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
