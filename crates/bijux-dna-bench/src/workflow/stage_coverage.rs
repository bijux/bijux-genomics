//! Owner: bijux-dna-bench
//! Deterministic stage-campaign coverage derivation and publication.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::contract::{
    validate_defined_campaign_portfolio_alignment, validate_executable_stage_campaign_registry,
    validate_report_admission, validate_stage_coverage, validate_stage_report_alignment,
};
use bijux_dna_bench_model::{
    benchmark_stage_coverage_schema, AdmittedBenchmarkStageReport, BenchmarkEvidenceTier,
    BenchmarkReportVerdict, BenchmarkStageCampaignRegistry, BenchmarkStageCoverage,
    BenchmarkStageCoverageRow, BenchmarkStageCoverageSummary, BenchmarkStageDefinitionStatus,
    BenchmarkStageExecutionStatus,
};
use bijux_dna_containers::api::{
    load_container_catalog, validate_container_catalog, ContainerCatalog,
};

use super::{
    load_stage_campaign_registry, load_stage_interface_catalog_from_repo,
    load_stage_reports_from_dir,
};
use crate::workflow::stage_report::current_report_sources;

use super::suite_load::load_benchmark_portfolio_from_repo;

const COVERAGE_JSON: &str = "benchmarks/reports/stage-benchmark-coverage.json";
const COVERAGE_MARKDOWN: &str = "benchmarks/reports/stage-benchmark-coverage.md";
const COVERAGE_SCHEMA: &str = "benchmarks/contracts/schemas/stage-coverage.schema.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BenchmarkCoverageWriteMode {
    Write,
    Check,
}

/// Derive complete runnable-stage coverage from typed catalog, campaign, and admitted evidence.
///
/// # Errors
/// Returns an error when campaign tools drift from the runnable catalog or report identities drift
/// from their declared campaigns.
pub fn derive_stage_coverage(
    registry: &BenchmarkStageCampaignRegistry,
    catalog: &ContainerCatalog,
    reports: &[AdmittedBenchmarkStageReport],
    required_repository_commit: &str,
) -> Result<BenchmarkStageCoverage> {
    let current_report_sources = BTreeSet::from([required_repository_commit.to_string()]);
    derive_stage_coverage_with_current_sources(
        registry,
        catalog,
        reports,
        required_repository_commit,
        &current_report_sources,
    )
}

fn derive_stage_coverage_with_current_sources(
    registry: &BenchmarkStageCampaignRegistry,
    catalog: &ContainerCatalog,
    reports: &[AdmittedBenchmarkStageReport],
    required_repository_commit: &str,
    current_report_sources: &BTreeSet<String>,
) -> Result<BenchmarkStageCoverage> {
    if required_repository_commit.len() != 40
        || !required_repository_commit.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        bail!("benchmark coverage requires a full repository commit identity");
    }
    validate_executable_stage_campaign_registry(registry)
        .context("validate stage campaign registry")?;
    let runnable_stages = runnable_stage_tools(catalog)?;
    let campaigns = registry
        .campaigns
        .iter()
        .map(|campaign| (campaign.stage_id.as_str(), campaign))
        .collect::<BTreeMap<_, _>>();
    for campaign in &registry.campaigns {
        let runnable_tools = runnable_stages.get(&campaign.stage_id).ok_or_else(|| {
            anyhow::anyhow!(
                "benchmark campaign has no runnable catalog tools: {}",
                campaign.stage_id
            )
        })?;
        let declared_tools = campaign.tool_ids.iter().cloned().collect::<BTreeSet<_>>();
        if &declared_tools != runnable_tools {
            let missing = runnable_tools.difference(&declared_tools).collect::<Vec<_>>();
            let extra = declared_tools.difference(runnable_tools).collect::<Vec<_>>();
            bail!(
                "benchmark campaign tool closure failed for {}: missing={missing:?}, extra={extra:?}",
                campaign.stage_id
            );
        }
    }

    let mut report_by_stage = BTreeMap::new();
    for admission in reports {
        validate_report_admission(admission).context("validate stage report admission")?;
        let campaign = campaigns.get(admission.report.stage_id.as_str()).ok_or_else(|| {
            anyhow::anyhow!(
                "benchmark report has no declared campaign: {}",
                admission.report.stage_id
            )
        })?;
        validate_stage_report_alignment(&admission.report, campaign, None)
            .context("align stage coverage report with campaign")?;
        if report_by_stage.insert(admission.report.stage_id.as_str(), admission).is_some() {
            bail!("benchmark coverage received duplicate admitted stage reports");
        }
    }

    let mut rows = Vec::with_capacity(runnable_stages.len());
    for (stage_id, tools) in runnable_stages {
        let campaign = campaigns.get(stage_id.as_str()).copied();
        let definition_status = campaign.map_or(BenchmarkStageDefinitionStatus::Missing, |value| {
            match value.evidence_tier {
                BenchmarkEvidenceTier::Qualification => {
                    BenchmarkStageDefinitionStatus::Qualification
                }
                BenchmarkEvidenceTier::Comparative => BenchmarkStageDefinitionStatus::Comparative,
            }
        });
        let admission = report_by_stage.get(stage_id.as_str()).copied();
        let execution_status = match (campaign, admission) {
            (None, None) => BenchmarkStageExecutionStatus::NotDefined,
            (None, Some(_)) => bail!("benchmark report has no declared campaign: {stage_id}"),
            (Some(_), None) => BenchmarkStageExecutionStatus::NotAdmitted,
            (Some(campaign), Some(admission)) => {
                if admission.report.run_id != campaign.run_id {
                    bail!("benchmark coverage report run identity drift for {stage_id}");
                }
                let report_commit = &admission.report.tools[0].repository_commit;
                if !current_report_sources.contains(report_commit) {
                    BenchmarkStageExecutionStatus::Stale
                } else {
                    match admission.report.verdict {
                        BenchmarkReportVerdict::Passed => BenchmarkStageExecutionStatus::Admitted,
                        BenchmarkReportVerdict::Failed => BenchmarkStageExecutionStatus::Failed,
                    }
                }
            }
        };
        rows.push(BenchmarkStageCoverageRow {
            stage_id,
            retained_tools: tools.into_iter().collect(),
            campaign_run_id: campaign.map(|value| value.run_id.clone()),
            report_repository_commit: admission
                .map(|value| value.report.tools[0].repository_commit.clone()),
            report_sha256: admission.map(|value| value.report_sha256.clone()),
            definition_status,
            execution_status,
        });
    }

    let summary = BenchmarkStageCoverageSummary {
        retained_stage_count: rows.len(),
        defined_stage_count: rows
            .iter()
            .filter(|row| row.definition_status != BenchmarkStageDefinitionStatus::Missing)
            .count(),
        qualification_stage_count: rows
            .iter()
            .filter(|row| row.definition_status == BenchmarkStageDefinitionStatus::Qualification)
            .count(),
        comparative_stage_count: rows
            .iter()
            .filter(|row| row.definition_status == BenchmarkStageDefinitionStatus::Comparative)
            .count(),
        admitted_stage_count: rows
            .iter()
            .filter(|row| row.execution_status == BenchmarkStageExecutionStatus::Admitted)
            .count(),
        stale_stage_count: rows
            .iter()
            .filter(|row| row.execution_status == BenchmarkStageExecutionStatus::Stale)
            .count(),
        failed_stage_count: rows
            .iter()
            .filter(|row| row.execution_status == BenchmarkStageExecutionStatus::Failed)
            .count(),
        missing_stage_count: rows
            .iter()
            .filter(|row| row.definition_status == BenchmarkStageDefinitionStatus::Missing)
            .count(),
    };
    let coverage = BenchmarkStageCoverage::v1(summary, rows);
    validate_stage_coverage(&coverage).context("validate derived benchmark stage coverage")?;
    Ok(coverage)
}

/// Derive and deterministically write or check the checked-in stage-coverage publication.
///
/// # Errors
/// Returns an error when repository contracts are invalid or generated files drift.
pub fn publish_checked_in_stage_coverage(
    repository_root: &Path,
    mode: BenchmarkCoverageWriteMode,
) -> Result<BenchmarkStageCoverage> {
    let catalog_validation = validate_container_catalog(repository_root)?;
    if !catalog_validation.is_valid() {
        bail!("container catalog validation failed: {:?}", catalog_validation.errors);
    }
    let registry = load_stage_campaign_registry(
        &repository_root.join("benchmarks/contracts/stage-campaigns.toml"),
    )?;
    let portfolio = load_benchmark_portfolio_from_repo(
        &repository_root.join("benchmarks/contracts/portfolio.toml"),
        repository_root,
    )?;
    validate_defined_campaign_portfolio_alignment(&registry, &portfolio)
        .context("align benchmark campaigns with portfolio")?;
    let interfaces = load_stage_interface_catalog_from_repo(
        &repository_root.join("benchmarks/contracts/stage-interfaces.toml"),
        repository_root,
    )?;
    let reports = load_stage_reports_from_dir(
        &repository_root.join("benchmarks/reports"),
        &registry,
        &interfaces,
    )?;
    let catalog = load_container_catalog(repository_root)?;
    let repository_commit = bijux_dna_containers::api::repository_head_commit(repository_root)?;
    let current_report_sources =
        current_report_sources(repository_root, &reports, &repository_commit)?;
    let coverage = derive_stage_coverage_with_current_sources(
        &registry,
        &catalog,
        &reports,
        &repository_commit,
        &current_report_sources,
    )?;

    let json = serde_json::to_string_pretty(&coverage)? + "\n";
    let markdown = render_stage_coverage_markdown(&coverage);
    let schema = serde_json::to_string_pretty(&benchmark_stage_coverage_schema())? + "\n";
    for (relative, content) in [
        (COVERAGE_JSON, json.as_bytes()),
        (COVERAGE_MARKDOWN, markdown.as_bytes()),
        (COVERAGE_SCHEMA, schema.as_bytes()),
    ] {
        let path = repository_root.join(relative);
        match mode {
            BenchmarkCoverageWriteMode::Write => {
                bijux_dna_infra::atomic_write_bytes(&path, content)
                    .with_context(|| format!("write generated benchmark coverage {relative}"))?;
            }
            BenchmarkCoverageWriteMode::Check => {
                let current = fs::read(&path)
                    .with_context(|| format!("read generated benchmark coverage {relative}"))?;
                if current != content {
                    bail!("generated benchmark coverage drifted: {relative}");
                }
            }
        }
    }
    Ok(coverage)
}

#[must_use]
pub fn render_stage_coverage_markdown(coverage: &BenchmarkStageCoverage) -> String {
    let summary = &coverage.summary;
    let mut lines = vec![
        "<!-- GENERATED FILE - DO NOT EDIT -->".to_string(),
        "<!-- source: bijux-dna-bench::workflow::stage_coverage -->".to_string(),
        "# Stage benchmark coverage".to_string(),
        String::new(),
        "A qualification campaign proves container execution and runnable-tool closure,"
            .to_string(),
        "and a reproducible measurement contract. It is not a comparative scientific".to_string(),
        "benchmark. Comparative readiness is governed separately by the corpus portfolio,"
            .to_string(),
        "truth strategy, cohort size, output contract, and downstream substitution probes."
            .to_string(),
        String::new(),
        format!("- Retained stages: **{}**", summary.retained_stage_count),
        format!("- Defined campaigns: **{}**", summary.defined_stage_count),
        format!("- Qualification-only campaigns: **{}**", summary.qualification_stage_count),
        format!("- Comparative campaigns: **{}**", summary.comparative_stage_count),
        format!("- Admitted campaigns: **{}**", summary.admitted_stage_count),
        format!("- Stale campaigns: **{}**", summary.stale_stage_count),
        format!("- Failed campaigns: **{}**", summary.failed_stage_count),
        format!("- Missing campaigns: **{}**", summary.missing_stage_count),
        String::new(),
        "| Stage | Retained tools | Campaign | Definition | Execution |".to_string(),
        "|---|---|---|---|---|".to_string(),
    ];
    for row in &coverage.stages {
        let tools = row
            .retained_tools
            .iter()
            .map(|tool| format!("`{tool}`"))
            .collect::<Vec<_>>()
            .join(", ");
        let run_id = row
            .campaign_run_id
            .as_deref()
            .map_or_else(|| "-".to_string(), |run_id| format!("`{run_id}`"));
        lines.push(format!(
            "| `{}` | {} | {} | {} | {} |",
            row.stage_id,
            tools,
            run_id,
            status_name(row.definition_status),
            execution_name(row.execution_status)
        ));
    }
    lines.join("\n") + "\n"
}

pub(super) fn runnable_stage_tools(
    catalog: &ContainerCatalog,
) -> Result<BTreeMap<String, BTreeSet<String>>> {
    let mut stages: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for tool in &catalog.tools {
        if !tool.is_product_supported() {
            continue;
        }
        for stage_id in &tool.stage_ids {
            if !stages.entry(stage_id.clone()).or_default().insert(tool.id.clone()) {
                bail!("container catalog duplicates tool {} for stage {stage_id}", tool.id);
            }
        }
    }
    if stages.is_empty() {
        bail!("container catalog has no runnable benchmark stages");
    }
    Ok(stages)
}

fn status_name(status: BenchmarkStageDefinitionStatus) -> &'static str {
    match status {
        BenchmarkStageDefinitionStatus::Missing => "missing",
        BenchmarkStageDefinitionStatus::Qualification => "qualification",
        BenchmarkStageDefinitionStatus::Comparative => "comparative",
    }
}

fn execution_name(status: BenchmarkStageExecutionStatus) -> &'static str {
    match status {
        BenchmarkStageExecutionStatus::NotDefined => "not_defined",
        BenchmarkStageExecutionStatus::NotAdmitted => "not_admitted",
        BenchmarkStageExecutionStatus::Stale => "stale",
        BenchmarkStageExecutionStatus::Failed => "failed",
        BenchmarkStageExecutionStatus::Admitted => "admitted",
    }
}
