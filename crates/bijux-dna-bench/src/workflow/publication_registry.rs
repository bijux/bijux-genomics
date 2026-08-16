//! Owner: bijux-dna-bench
//! Deterministic benchmark publication lifecycle registry.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::contract::{
    validate_benchmark_data_portfolio_audit, validate_benchmark_publication_registry,
    validate_benchmark_run_index, validate_stage_report,
};
use bijux_dna_bench_model::{
    benchmark_publication_registry_schema, BenchmarkCurrentPublication,
    BenchmarkDataPortfolioAudit, BenchmarkPublicationRegistry, BenchmarkPublicationRegistrySummary,
    BenchmarkQuarantinedPublicationSurface, BenchmarkRunIndex, BenchmarkRunReportDisposition,
    BenchmarkRunReportIndexEntry, BenchmarkStagePublication, BenchmarkStagePublicationDisposition,
    BenchmarkStageReport,
};

use super::data_portfolio_audit::render_benchmark_data_portfolio_audit_markdown;
use super::stage_report::report_source_is_current;

const REGISTRY_JSON: &str = "benchmarks/reports/publication-registry.json";
const REGISTRY_MARKDOWN: &str = "benchmarks/reports/publication-registry.md";
const REPORT_INDEX_JSON: &str = "benchmarks/reports/REPORT_INDEX.json";
const REPORT_INDEX_MARKDOWN: &str = "benchmarks/reports/REPORT_INDEX.md";
const REGISTRY_SCHEMA: &str = "benchmarks/contracts/schemas/publication-registry.schema.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BenchmarkPublicationRegistryWriteMode {
    Write,
    Check,
}

/// Derive current and quarantined benchmark publication ownership.
///
/// # Errors
/// Returns an error when report paths, source identities, tracked readiness files, or file digests
/// cannot be resolved into one unambiguous lifecycle classification.
pub fn derive_benchmark_publication_registry(
    repository_root: &Path,
) -> Result<BenchmarkPublicationRegistry> {
    let current_products = current_products(repository_root)?;
    let run_reports = permanent_run_reports(repository_root)?;
    let repository_commit = bijux_dna_containers::api::repository_head_commit(repository_root)?;
    let mut stage_reports = active_stage_reports(repository_root, &repository_commit)?;
    stage_reports.sort_by(|left, right| {
        left.run_id
            .cmp(&right.run_id)
            .then_with(|| left.source_commit.cmp(&right.source_commit))
            .then_with(|| left.json_path.cmp(&right.json_path))
    });
    let quarantined_surfaces: Vec<BenchmarkQuarantinedPublicationSurface> =
        quarantined_readiness_surface(repository_root)?.into_iter().collect();
    let summary = BenchmarkPublicationRegistrySummary {
        current_product_count: current_products.len(),
        current_run_report_count: count_run_reports(
            &run_reports,
            BenchmarkRunReportDisposition::Current,
        ),
        missing_run_report_count: count_run_reports(
            &run_reports,
            BenchmarkRunReportDisposition::Missing,
        ),
        stale_run_report_count: count_run_reports(
            &run_reports,
            BenchmarkRunReportDisposition::Stale,
        ),
        failed_run_report_count: count_run_reports(
            &run_reports,
            BenchmarkRunReportDisposition::Failed,
        ),
        superseded_run_report_count: count_run_reports(
            &run_reports,
            BenchmarkRunReportDisposition::Superseded,
        ),
        active_stage_report_count: stage_reports
            .iter()
            .filter(|row| row.disposition == BenchmarkStagePublicationDisposition::Active)
            .count(),
        historical_stage_report_count: stage_reports
            .iter()
            .filter(|row| row.disposition == BenchmarkStagePublicationDisposition::Historical)
            .count(),
        quarantined_surface_count: quarantined_surfaces.len(),
        quarantined_file_count: quarantined_surfaces
            .iter()
            .map(|surface| surface.tracked_file_count)
            .sum(),
    };
    let registry = BenchmarkPublicationRegistry::v2(
        summary,
        current_products,
        run_reports,
        stage_reports,
        quarantined_surfaces,
    );
    validate_benchmark_publication_registry(&registry)
        .context("validate benchmark publication registry")?;
    Ok(registry)
}

fn count_run_reports(
    reports: &[BenchmarkRunReportIndexEntry],
    disposition: BenchmarkRunReportDisposition,
) -> usize {
    reports.iter().filter(|report| report.disposition == disposition).count()
}

fn permanent_run_reports(repository_root: &Path) -> Result<Vec<BenchmarkRunReportIndexEntry>> {
    let path = repository_root.join("benchmarks/runs/RUN_INDEX.json");
    let bytes = fs::read(&path).context("read permanent benchmark run index")?;
    let index: BenchmarkRunIndex =
        serde_json::from_slice(&bytes).context("parse permanent benchmark run index")?;
    validate_benchmark_run_index(&index).context("validate permanent benchmark run index")?;
    Ok(index
        .runs
        .into_iter()
        .map(|run| match run.selected_publication {
            Some(publication) => BenchmarkRunReportIndexEntry {
                run_id: run.run_id,
                stage_id: run.stage_id,
                disposition: BenchmarkRunReportDisposition::Current,
                report: Some(publication.report),
                publication_receipt: Some(publication.publication_receipt),
                limitations: publication.limitations,
            },
            None => BenchmarkRunReportIndexEntry {
                run_id: run.run_id,
                stage_id: run.stage_id,
                disposition: BenchmarkRunReportDisposition::Missing,
                report: None,
                publication_receipt: None,
                limitations: run.limitations,
            },
        })
        .collect())
}

/// Write or check the checked-in publication registry, review report, and generated schema.
///
/// # Errors
/// Returns an error when derivation fails or a checked-in generated output has drifted.
pub fn publish_checked_in_benchmark_publication_registry(
    repository_root: &Path,
    mode: BenchmarkPublicationRegistryWriteMode,
) -> Result<BenchmarkPublicationRegistry> {
    let registry = derive_benchmark_publication_registry(repository_root)?;
    let json = serde_json::to_string_pretty(&registry)? + "\n";
    let markdown = render_benchmark_publication_registry_markdown(&registry);
    let schema = serde_json::to_string_pretty(&benchmark_publication_registry_schema())? + "\n";
    for (relative, content) in [
        (REGISTRY_JSON, json.as_bytes()),
        (REGISTRY_MARKDOWN, markdown.as_bytes()),
        (REPORT_INDEX_JSON, json.as_bytes()),
        (REPORT_INDEX_MARKDOWN, markdown.as_bytes()),
        (REGISTRY_SCHEMA, schema.as_bytes()),
    ] {
        let path = repository_root.join(relative);
        match mode {
            BenchmarkPublicationRegistryWriteMode::Write => {
                bijux_dna_infra::atomic_write_bytes(&path, content)
                    .with_context(|| format!("write generated publication registry {relative}"))?;
            }
            BenchmarkPublicationRegistryWriteMode::Check => {
                let current = fs::read(&path)
                    .with_context(|| format!("read generated publication registry {relative}"))?;
                if current != content {
                    bail!("generated benchmark publication registry drifted: {relative}");
                }
            }
        }
    }
    Ok(registry)
}

#[must_use]
pub fn render_benchmark_publication_registry_markdown(
    registry: &BenchmarkPublicationRegistry,
) -> String {
    let summary = &registry.summary;
    let mut lines = vec![
        "<!-- GENERATED FILE - DO NOT EDIT -->".to_string(),
        "<!-- source: bijux-dna-bench::workflow::publication_registry -->".to_string(),
        "# Benchmark publication registry".to_string(),
        String::new(),
        "Path presence does not confer current benchmark authority. Every permanent run is explicit below: current requires an admitted selected report and publication receipt, while absent evidence remains missing. Historical publications remain immutable in STUDY and are not copied into this current-only report namespace; tracked legacy readiness output is quarantined until removed.".to_string(),
        String::new(),
        format!("- Current governance products: **{}**", summary.current_product_count),
        format!("- Current run reports: **{}**", summary.current_run_report_count),
        format!("- Missing run reports: **{}**", summary.missing_run_report_count),
        format!("- Stale run reports: **{}**", summary.stale_run_report_count),
        format!("- Failed run reports: **{}**", summary.failed_run_report_count),
        format!("- Superseded run reports: **{}**", summary.superseded_run_report_count),
        format!("- Active stage reports: **{}**", summary.active_stage_report_count),
        format!("- Historical stage reports: **{}**", summary.historical_stage_report_count),
        format!("- Quarantined tracked files: **{}**", summary.quarantined_file_count),
        String::new(),
        "## Current products".to_string(),
        String::new(),
        "| Product | JSON | Markdown |".to_string(),
        "|---|---|---|".to_string(),
    ];
    for product in &registry.current_products {
        lines.push(format!(
            "| `{}` | `{}` | `{}` |",
            product.product_id, product.json_path, product.markdown_path
        ));
    }
    lines.extend([
        String::new(),
        "## Permanent run reports".to_string(),
        String::new(),
        "| Run | Stage | Disposition | Report | Publication receipt |".to_string(),
        "|---|---|---|---|---|".to_string(),
    ]);
    for report in &registry.run_reports {
        let disposition = match report.disposition {
            BenchmarkRunReportDisposition::Current => "current",
            BenchmarkRunReportDisposition::Missing => "missing",
            BenchmarkRunReportDisposition::Stale => "stale",
            BenchmarkRunReportDisposition::Failed => "failed",
            BenchmarkRunReportDisposition::Superseded => "superseded",
        };
        let report_uri = report.report.as_ref().map_or("—", |reference| reference.uri.as_str());
        let receipt_uri =
            report.publication_receipt.as_ref().map_or("—", |reference| reference.uri.as_str());
        lines.push(format!(
            "| `{}` | `{}` | {} | `{report_uri}` | `{receipt_uri}` |",
            report.run_id, report.stage_id, disposition
        ));
    }
    lines.extend([
        String::new(),
        "## Stage reports".to_string(),
        String::new(),
        "| Run | Stage | Source commit | Disposition | JSON |".to_string(),
        "|---|---|---|---|---|".to_string(),
    ]);
    for report in &registry.stage_reports {
        let disposition = match report.disposition {
            BenchmarkStagePublicationDisposition::Active => "active",
            BenchmarkStagePublicationDisposition::Historical => "historical",
        };
        lines.push(format!(
            "| `{}` | `{}` | `{}` | {} | `{}` |",
            report.run_id, report.stage_id, report.source_commit, disposition, report.json_path
        ));
    }
    lines.extend([
        String::new(),
        "## Quarantined surfaces".to_string(),
        String::new(),
        "| Prefix | Files | Tree SHA-256 | Policy |".to_string(),
        "|---|---:|---|---|".to_string(),
    ]);
    for surface in &registry.quarantined_surfaces {
        lines.push(format!(
            "| `{}` | {} | `{}` | {} |",
            surface.path_prefix,
            surface.tracked_file_count,
            surface.tree_sha256,
            surface.authority_policy
        ));
    }
    lines.join("\n") + "\n"
}

fn current_products(repository_root: &Path) -> Result<Vec<BenchmarkCurrentPublication>> {
    validate_current_data_portfolio_product(repository_root)?;
    [
        (
            "permanent-run-registry",
            "benchmarks/contracts/run-registry.json",
            "benchmarks/reports/run-registry.md",
        ),
        ("paper-run-index", "benchmarks/runs/RUN_INDEX.json", "benchmarks/runs/RUN_INDEX.md"),
        (
            "stage-universe",
            "benchmarks/contracts/stage-universe.json",
            "benchmarks/reports/stage-universe.md",
        ),
        (
            "stage-benchmark-coverage",
            "benchmarks/reports/stage-benchmark-coverage.json",
            "benchmarks/reports/stage-benchmark-coverage.md",
        ),
        (
            "stage-tool-closure",
            "benchmarks/reports/STAGE_TOOL_CLOSURE.json",
            "benchmarks/reports/STAGE_TOOL_CLOSURE.md",
        ),
        (
            "scientific-benchmark-readiness",
            "benchmarks/reports/scientific-benchmark-readiness.json",
            "benchmarks/reports/scientific-benchmark-readiness.md",
        ),
        (
            "data-portfolio",
            "benchmarks/reports/DATA_PORTFOLIO.json",
            "benchmarks/reports/DATA_PORTFOLIO.md",
        ),
    ]
    .into_iter()
    .map(|(product_id, json_path, markdown_path)| {
        Ok(BenchmarkCurrentPublication {
            product_id: product_id.to_string(),
            json_path: json_path.to_string(),
            json_sha256: file_sha256(&repository_root.join(json_path))?,
            markdown_path: markdown_path.to_string(),
            markdown_sha256: file_sha256(&repository_root.join(markdown_path))?,
        })
    })
    .collect()
}

fn validate_current_data_portfolio_product(repository_root: &Path) -> Result<()> {
    let json_path = repository_root.join("benchmarks/reports/DATA_PORTFOLIO.json");
    let markdown_path = repository_root.join("benchmarks/reports/DATA_PORTFOLIO.md");
    let json = fs::read(&json_path).context("read current data-portfolio report")?;
    let audit: BenchmarkDataPortfolioAudit =
        serde_json::from_slice(&json).context("parse current data-portfolio report")?;
    validate_benchmark_data_portfolio_audit(&audit)
        .context("validate current data-portfolio report")?;

    let markdown = fs::read_to_string(&markdown_path)
        .context("read current data-portfolio Markdown report")?;
    if markdown != render_benchmark_data_portfolio_audit_markdown(&audit) {
        bail!("current data-portfolio Markdown differs from its typed JSON report");
    }
    if file_sha256(&governed_report_source_path(repository_root, &audit.portfolio_path)?)?
        != audit.portfolio_sha256
    {
        bail!("current data-portfolio report has stale portfolio authority");
    }
    for source in audit.corpora.iter().flat_map(|corpus| &corpus.source_specs) {
        if file_sha256(&governed_report_source_path(repository_root, &source.path)?)?
            != source.sha256
        {
            bail!("current data-portfolio report has stale source authority: {}", source.path);
        }
    }
    let data_index_identity = audit
        .data_index_path
        .strip_prefix("study://data/evidence/indexes/")
        .and_then(|value| value.strip_suffix("/data-index.json"));
    if !data_index_identity.is_some_and(|identity| is_hex(identity, 40)) {
        bail!("current data-portfolio report lacks an immutable STUDY data-index identity");
    }
    Ok(())
}

fn governed_report_source_path(repository_root: &Path, value: &str) -> Result<PathBuf> {
    let path = Path::new(value);
    if path.is_absolute()
        || path.components().any(|component| !matches!(component, Component::Normal(_)))
    {
        bail!("current data-portfolio report contains a non-normalized source path: {value}");
    }
    Ok(repository_root.join(path))
}

fn active_stage_reports(
    repository_root: &Path,
    repository_commit: &str,
) -> Result<Vec<BenchmarkStagePublication>> {
    let reports_root = repository_root.join("benchmarks/reports");
    let mut reports = Vec::new();
    let mut json_stems = BTreeSet::new();
    let mut markdown_stems = BTreeSet::new();
    for entry in fs::read_dir(&reports_root).context("read active benchmark reports")? {
        let path = entry?.path();
        let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if path.is_file() && is_stage_report_file(file_name, ".json") {
            json_stems.insert(file_name.trim_end_matches(".json").to_string());
            let report = stage_publication(
                repository_root,
                &path,
                BenchmarkStagePublicationDisposition::Active,
                None,
            )?;
            if !report_source_is_current(repository_root, &report.source_commit, repository_commit)?
            {
                bail!(
                    "stale stage report remains in active publication namespace: {}",
                    report.json_path
                );
            }
            reports.push(report);
        } else if path.is_file() && is_stage_report_file(file_name, ".md") {
            markdown_stems.insert(file_name.trim_end_matches(".md").to_string());
        }
    }
    if json_stems != markdown_stems {
        bail!("active stage report JSON and Markdown pairs drifted");
    }
    Ok(reports)
}

fn stage_publication(
    repository_root: &Path,
    json_path: &Path,
    disposition: BenchmarkStagePublicationDisposition,
    expected_source_commit: Option<&str>,
) -> Result<BenchmarkStagePublication> {
    let json = fs::read(json_path)
        .with_context(|| format!("read stage publication {}", json_path.display()))?;
    let report: BenchmarkStageReport = serde_json::from_slice(&json)
        .with_context(|| format!("parse stage publication {}", json_path.display()))?;
    validate_stage_report(&report)
        .with_context(|| format!("validate stage publication {}", json_path.display()))?;
    let source_commit = report
        .tools
        .first()
        .context("stage publication has no tool evidence")?
        .repository_commit
        .clone();
    if expected_source_commit.is_some_and(|expected| expected != source_commit) {
        bail!("historical report source commit differs from its directory");
    }
    let markdown_path = json_path.with_extension("md");
    if !markdown_path.is_file() {
        bail!("stage publication lacks its Markdown pair: {}", json_path.display());
    }
    Ok(BenchmarkStagePublication {
        run_id: report.run_id,
        stage_id: report.stage_id,
        source_commit,
        disposition,
        json_path: relative_path(repository_root, json_path)?,
        json_sha256: bijux_dna_infra::hash_bytes_sha256(&json),
        markdown_path: relative_path(repository_root, &markdown_path)?,
        markdown_sha256: file_sha256(&markdown_path)?,
    })
}

fn quarantined_readiness_surface(
    repository_root: &Path,
) -> Result<Option<BenchmarkQuarantinedPublicationSurface>> {
    let paths = bijux_dna_containers::api::tracked_repository_files_under(
        repository_root,
        Path::new("benchmarks/readiness"),
    )?;
    if paths.is_empty() {
        return Ok(None);
    }
    let mut tree_basis = Vec::new();
    for path in &paths {
        let digest = file_sha256(&repository_root.join(path))?;
        tree_basis.extend_from_slice(format!("{digest}  {}\n", path.display()).as_bytes());
    }
    Ok(Some(BenchmarkQuarantinedPublicationSurface {
        path_prefix: "benchmarks/readiness/".to_string(),
        disposition: "quarantined_legacy_readiness".to_string(),
        tracked_file_count: paths.len(),
        tree_sha256: bijux_dna_infra::hash_bytes_sha256(&tree_basis),
        authority_policy: "Preserved migration evidence only; never admits campaigns, science, evidence books, or final gates.".to_string(),
    }))
}

fn file_sha256(path: &Path) -> Result<String> {
    let bytes = fs::read(path).with_context(|| format!("read publication {}", path.display()))?;
    Ok(bijux_dna_infra::hash_bytes_sha256(&bytes))
}

fn relative_path(repository_root: &Path, path: &Path) -> Result<String> {
    path.strip_prefix(repository_root)
        .with_context(|| format!("publication path is outside repository: {}", path.display()))?
        .to_str()
        .map(str::to_string)
        .context("publication path is not UTF-8")
}

fn is_hex(value: &str, length: usize) -> bool {
    value.len() == length && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn is_stage_report_file(file_name: &str, extension: &str) -> bool {
    file_name.strip_suffix(extension).is_some_and(|stem| {
        stem.len() > 9
            && stem.starts_with("run-")
            && stem[4..8].bytes().all(|byte| byte.is_ascii_digit())
            && stem.as_bytes()[8] == b'-'
    })
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::governed_report_source_path;

    #[test]
    fn current_data_report_sources_reject_absolute_and_traversing_paths() {
        let root = Path::new("/repository");
        assert_eq!(
            governed_report_source_path(root, "benchmarks/corpora/source.toml")
                .expect("normalized source path"),
            root.join("benchmarks/corpora/source.toml")
        );
        for path in [
            "/workspace/checkout/benchmarks/corpora/source.toml",
            "benchmarks/corpora/../reports/DATA_PORTFOLIO.json",
            "./benchmarks/corpora/source.toml",
        ] {
            assert!(governed_report_source_path(root, path).is_err(), "accepted {path}");
        }
    }
}
