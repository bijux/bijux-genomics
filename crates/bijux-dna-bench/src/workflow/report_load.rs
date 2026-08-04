//! Owner: bijux-dna-bench
//! Repository-backed admission of benchmark stage reports.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use bijux_dna_bench_model::contract::{validate_stage_report, validate_stage_report_alignment};
use bijux_dna_bench_model::{
    BenchmarkStageCampaignRegistry, BenchmarkStageInterfaceCatalog, BenchmarkStageReport,
};

use crate::repo::bench_reports_dir;

/// Load and structurally validate one benchmark stage report.
///
/// # Errors
/// Returns an error if the report cannot be read, parsed, or admitted.
pub fn load_stage_report(path: &Path) -> Result<BenchmarkStageReport> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("read benchmark stage report {}", path.display()))?;
    let report: BenchmarkStageReport = serde_json::from_str(&raw)
        .with_context(|| format!("parse benchmark stage report {}", path.display()))?;
    validate_stage_report(&report)
        .with_context(|| format!("validate benchmark stage report {}", path.display()))?;
    Ok(report)
}

/// Load, align, and admit all checked-in benchmark stage reports.
///
/// # Errors
/// Returns an error if a report is duplicated or drifts from its campaign/interface authority.
pub fn load_checked_in_stage_reports() -> Result<Vec<BenchmarkStageReport>> {
    let (registry, _, interfaces) = super::load_checked_in_benchmark_planning_contracts()?;
    load_stage_reports_from_dir(&bench_reports_dir(), &registry, &interfaces)
}

fn load_stage_reports_from_dir(
    reports_dir: &Path,
    registry: &BenchmarkStageCampaignRegistry,
    interfaces: &BenchmarkStageInterfaceCatalog,
) -> Result<Vec<BenchmarkStageReport>> {
    let campaigns = registry
        .campaigns
        .iter()
        .map(|campaign| (campaign.run_id.as_str(), campaign))
        .collect::<BTreeMap<_, _>>();
    let interface_by_stage = interfaces
        .interfaces
        .iter()
        .map(|interface| (interface.stage_id.as_str(), interface))
        .collect::<BTreeMap<_, _>>();
    let mut paths = fs::read_dir(reports_dir)
        .with_context(|| format!("read benchmark report directory {}", reports_dir.display()))?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| {
            path.extension().and_then(|extension| extension.to_str()) == Some("json")
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.starts_with("run-"))
        })
        .collect::<Vec<_>>();
    paths.sort();

    let mut reports = Vec::new();
    let mut seen_stages = std::collections::BTreeSet::new();
    for path in paths {
        let report = load_stage_report(&path)?;
        let campaign = campaigns.get(report.run_id.as_str()).ok_or_else(|| {
            anyhow::anyhow!("benchmark report has no declared campaign: {}", report.run_id)
        })?;
        validate_stage_report_alignment(
            &report,
            campaign,
            interface_by_stage.get(report.stage_id.as_str()).copied(),
        )
        .with_context(|| format!("align benchmark stage report {}", path.display()))?;
        if !seen_stages.insert(report.stage_id.clone()) {
            anyhow::bail!("benchmark report stage is duplicated: {}", report.stage_id);
        }
        reports.push(report);
    }
    Ok(reports)
}
