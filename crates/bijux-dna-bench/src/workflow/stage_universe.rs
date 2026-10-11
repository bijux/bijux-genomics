//! Owner: bijux-dna-bench
//! Canonical domain-stage disposition derivation and publication.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::contract::{validate_benchmark_portfolio, validate_stage_universe};
use bijux_dna_bench_model::{
    benchmark_stage_universe_schema, BenchmarkDisposition, BenchmarkPortfolio,
    BenchmarkStageUniverse, BenchmarkStageUniverseRow, BenchmarkStageUniverseSummary,
};
use serde::Deserialize;

use super::suite_load::load_benchmark_portfolio_from_repo;

const UNIVERSE_JSON: &str = "benchmarks/contracts/stage-universe.json";
const UNIVERSE_MARKDOWN: &str = "benchmarks/reports/stage-universe.md";
const UNIVERSE_SCHEMA: &str = "benchmarks/contracts/schemas/stage-universe.schema.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BenchmarkUniverseWriteMode {
    Write,
    Check,
}

#[derive(Debug, Deserialize)]
struct StageDispositionSource {
    stage_id: String,
    status: String,
    domain: String,
    benchmark_disposition: BenchmarkDisposition,
    benchmark_disposition_reason: String,
    #[serde(flatten)]
    _remaining_contract: serde_yaml::Mapping,
}

/// Derive the canonical stage universe from domain stage contracts and benchmark run reservations.
///
/// # Errors
/// Returns an error when a stage is malformed, duplicated, or disagrees with the benchmark
/// portfolio disposition and run-ID closure.
pub fn derive_stage_universe(
    repository_root: &Path,
    portfolio: &BenchmarkPortfolio,
) -> Result<BenchmarkStageUniverse> {
    validate_benchmark_portfolio(portfolio).context("validate benchmark portfolio")?;
    let mut rows = Vec::new();
    let mut seen_stages = BTreeSet::new();

    for domain in ["bam", "fastq", "vcf"] {
        let stages_dir = repository_root.join("domain").join(domain).join("stages");
        let mut paths = fs::read_dir(&stages_dir)
            .with_context(|| format!("read domain stage directory {}", stages_dir.display()))?
            .map(|entry| entry.map(|value| value.path()))
            .collect::<std::io::Result<Vec<_>>>()?;
        paths.sort();
        for path in paths {
            let file_name = path.file_name().and_then(|value| value.to_str()).unwrap_or_default();
            if path.extension().and_then(|value| value.to_str()) != Some("yaml")
                || file_name.starts_with('_')
            {
                continue;
            }
            let raw = fs::read_to_string(&path)
                .with_context(|| format!("read domain stage contract {}", path.display()))?;
            let source: StageDispositionSource = serde_yaml::from_str(&raw)
                .with_context(|| format!("parse domain stage disposition {}", path.display()))?;
            if source.domain != domain {
                bail!(
                    "domain stage contract {} declares domain {} instead of {domain}",
                    path.display(),
                    source.domain
                );
            }
            let expected_file = format!(
                "{}.yaml",
                source.stage_id.strip_prefix(&format!("{domain}.")).ok_or_else(|| {
                    anyhow::anyhow!("stage id has wrong domain prefix: {}", source.stage_id)
                })?
            );
            if file_name != expected_file {
                bail!(
                    "domain stage file name disagrees with stage id: {file_name} != {expected_file}"
                );
            }
            if !seen_stages.insert(source.stage_id.clone()) {
                bail!("domain stage id is duplicated: {}", source.stage_id);
            }

            let reserved_run_id = portfolio.stage_runs.get(&source.stage_id).cloned();
            match (source.benchmark_disposition, reserved_run_id.as_ref()) {
                (BenchmarkDisposition::BenchmarkRequired, None) => {
                    bail!("benchmark-required stage is absent from portfolio: {}", source.stage_id)
                }
                (BenchmarkDisposition::BenchmarkRequired, Some(_)) | (_, None) => {}
                (_, Some(run_id)) => bail!(
                    "portfolio stage {} reserves {run_id} but has disposition {:?}",
                    source.stage_id,
                    source.benchmark_disposition
                ),
            }
            rows.push(BenchmarkStageUniverseRow {
                stage_id: source.stage_id,
                domain: source.domain,
                status: source.status,
                benchmark_disposition: source.benchmark_disposition,
                benchmark_disposition_reason: source.benchmark_disposition_reason,
                reserved_run_id,
            });
        }
    }
    rows.sort_by(|left, right| left.stage_id.cmp(&right.stage_id));

    let benchmark_required = rows
        .iter()
        .filter(|row| row.benchmark_disposition == BenchmarkDisposition::BenchmarkRequired)
        .map(|row| row.stage_id.as_str())
        .collect::<BTreeSet<_>>();
    let reserved = portfolio.stage_runs.keys().map(String::as_str).collect::<BTreeSet<_>>();
    if benchmark_required != reserved {
        let missing = reserved.difference(&benchmark_required).copied().collect::<Vec<_>>();
        let extra = benchmark_required.difference(&reserved).copied().collect::<Vec<_>>();
        bail!("benchmark stage universe closure failed: missing={missing:?}, extra={extra:?}");
    }

    let count =
        |disposition| rows.iter().filter(|row| row.benchmark_disposition == disposition).count();
    let summary = BenchmarkStageUniverseSummary {
        stage_count: rows.len(),
        benchmark_required_count: count(BenchmarkDisposition::BenchmarkRequired),
        qualification_only_count: count(BenchmarkDisposition::QualificationOnly),
        orchestration_only_count: count(BenchmarkDisposition::OrchestrationOnly),
        test_only_count: count(BenchmarkDisposition::TestOnly),
        retired_count: count(BenchmarkDisposition::Retired),
        excluded_count: count(BenchmarkDisposition::Excluded),
    };
    let universe = BenchmarkStageUniverse::v1(summary, rows);
    validate_stage_universe(&universe).context("validate derived benchmark stage universe")?;
    Ok(universe)
}

/// Derive and deterministically write or check the checked-in stage-universe publication.
///
/// # Errors
/// Returns an error when source contracts are invalid or generated outputs drift.
pub fn publish_checked_in_stage_universe(
    repository_root: &Path,
    mode: BenchmarkUniverseWriteMode,
) -> Result<BenchmarkStageUniverse> {
    let portfolio = load_benchmark_portfolio_from_repo(
        &repository_root.join("benchmarks/contracts/portfolio.toml"),
        repository_root,
    )?;
    let universe = derive_stage_universe(repository_root, &portfolio)?;
    let json = serde_json::to_string_pretty(&universe)? + "\n";
    let markdown = render_stage_universe_markdown(&universe);
    let schema = serde_json::to_string_pretty(&benchmark_stage_universe_schema())? + "\n";
    for (relative, content) in [
        (UNIVERSE_JSON, json.as_bytes()),
        (UNIVERSE_MARKDOWN, markdown.as_bytes()),
        (UNIVERSE_SCHEMA, schema.as_bytes()),
    ] {
        let path = repository_root.join(relative);
        match mode {
            BenchmarkUniverseWriteMode::Write => {
                bijux_dna_infra::atomic_write_bytes(&path, content)
                    .with_context(|| format!("write generated stage universe {relative}"))?;
            }
            BenchmarkUniverseWriteMode::Check => {
                let current = fs::read(&path)
                    .with_context(|| format!("read generated stage universe {relative}"))?;
                if current != content {
                    bail!("generated benchmark stage universe drifted: {relative}");
                }
            }
        }
    }
    Ok(universe)
}

#[must_use]
pub fn render_stage_universe_markdown(universe: &BenchmarkStageUniverse) -> String {
    let summary = &universe.summary;
    let mut lines = vec![
        "<!-- GENERATED FILE - DO NOT EDIT -->".to_string(),
        "<!-- source: bijux-dna-bench::workflow::stage_universe -->".to_string(),
        "# Canonical stage universe".to_string(),
        String::new(),
        "Every domain stage has an explicit benchmark disposition. The domain YAML stage contract is the authority; this index binds benchmark-required stages to permanent run IDs.".to_string(),
        String::new(),
        format!("- Domain stages: **{}**", summary.stage_count),
        format!("- Benchmark required: **{}**", summary.benchmark_required_count),
        format!("- Qualification only: **{}**", summary.qualification_only_count),
        format!("- Orchestration only: **{}**", summary.orchestration_only_count),
        format!("- Test only: **{}**", summary.test_only_count),
        format!("- Retired: **{}**", summary.retired_count),
        format!("- Excluded: **{}**", summary.excluded_count),
        String::new(),
        "| Stage | Status | Disposition | Run | Rationale |".to_string(),
        "|---|---|---|---|---|".to_string(),
    ];
    for row in &universe.stages {
        lines.push(format!(
            "| `{}` | `{}` | `{}` | {} | {} |",
            row.stage_id,
            row.status,
            row.benchmark_disposition.as_str(),
            row.reserved_run_id.as_deref().map_or("-".to_string(), |run| format!("`{run}`")),
            row.benchmark_disposition_reason.replace('|', "\\|")
        ));
    }
    lines.push(String::new());
    lines.join("\n")
}
