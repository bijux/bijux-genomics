//! Owner: bijux-dna-bench
//! Repository-backed admission of benchmark stage reports.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};
use bijux_dna_bench_model::contract::{validate_report_admission, validate_stage_report_alignment};
use bijux_dna_bench_model::{
    AdmittedBenchmarkStageReport, BenchmarkStageCampaignRegistry, BenchmarkStageInterfaceCatalog,
    BenchmarkStageReport, BENCHMARK_REPORT_ADMISSION_SCHEMA_V1,
};

use crate::repo::bench_reports_dir;

/// Load and structurally validate one benchmark stage report.
///
/// # Errors
/// Returns an error if the report cannot be read, parsed, or admitted.
pub fn load_stage_report(path: &Path) -> Result<AdmittedBenchmarkStageReport> {
    let raw = fs::read(path)
        .with_context(|| format!("read benchmark stage report {}", path.display()))?;
    let report: BenchmarkStageReport = serde_json::from_slice(&raw)
        .with_context(|| format!("parse benchmark stage report {}", path.display()))?;
    let report_file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| anyhow::anyhow!("benchmark report path has no UTF-8 file name"))?
        .to_string();
    let admission = AdmittedBenchmarkStageReport {
        schema_version: BENCHMARK_REPORT_ADMISSION_SCHEMA_V1.to_string(),
        report_file_name,
        source_sha256: bijux_dna_infra::hash_bytes_sha256(&raw),
        report_sha256: bijux_dna_infra::hash_bytes_sha256(&serde_json::to_vec(&report)?),
        report,
    };
    validate_report_admission(&admission)
        .with_context(|| format!("validate benchmark stage report {}", path.display()))?;
    Ok(admission)
}

/// Load, align, and admit all checked-in benchmark stage reports.
///
/// # Errors
/// Returns an error if a report is duplicated or drifts from its campaign/interface authority.
pub fn load_checked_in_stage_reports() -> Result<Vec<AdmittedBenchmarkStageReport>> {
    let (registry, _, interfaces) = super::load_checked_in_benchmark_planning_contracts()?;
    load_stage_reports_from_dir(&bench_reports_dir(), &registry, &interfaces)
}

/// Load and align benchmark stage reports from a declared publication directory.
///
/// # Errors
/// Returns an error if a report is duplicated or drifts from campaign/interface authority.
pub fn load_stage_reports_from_dir(
    reports_dir: &Path,
    registry: &BenchmarkStageCampaignRegistry,
    interfaces: &BenchmarkStageInterfaceCatalog,
) -> Result<Vec<AdmittedBenchmarkStageReport>> {
    let repository_root = reports_dir
        .ancestors()
        .find(|root| root.join(".git").exists())
        .ok_or_else(|| anyhow::anyhow!("benchmark report directory is outside a Git repository"))?;
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
    let mut catalog_identities: BTreeMap<String, String> = BTreeMap::new();
    for path in paths {
        let admission = load_stage_report(&path)?;
        let report = &admission.report;
        let repository_commit = &report.tools[0].repository_commit;
        let expected_catalog_sha256 =
            if let Some(identity) = catalog_identities.get(repository_commit) {
                identity.clone()
            } else {
                let identity = catalog_identity_at_commit(repository_root, repository_commit)?;
                catalog_identities.insert(repository_commit.clone(), identity.clone());
                identity
            };
        if report.catalog_sha256 != expected_catalog_sha256 {
            anyhow::bail!(
                "benchmark report catalog identity is stale for {}: expected {}, observed {}",
                report.run_id,
                expected_catalog_sha256,
                report.catalog_sha256
            );
        }
        let campaign = campaigns.get(report.run_id.as_str()).ok_or_else(|| {
            anyhow::anyhow!("benchmark report has no declared campaign: {}", report.run_id)
        })?;
        validate_stage_report_alignment(
            report,
            campaign,
            interface_by_stage.get(report.stage_id.as_str()).copied(),
        )
        .with_context(|| format!("align benchmark stage report {}", path.display()))?;
        if !seen_stages.insert(report.stage_id.clone()) {
            anyhow::bail!("benchmark report stage is duplicated: {}", report.stage_id);
        }
        reports.push(admission);
    }
    Ok(reports)
}

fn catalog_identity_at_commit(repository_root: &Path, commit: &str) -> Result<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repository_root)
        .args(["show", &format!("{commit}:containers/contracts/catalog.toml")])
        .output()
        .with_context(|| format!("read container catalog at benchmark commit {commit}"))?;
    if !output.status.success() {
        anyhow::bail!(
            "cannot read container catalog at benchmark commit {commit}: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(bijux_dna_infra::hash_bytes_sha256(&output.stdout))
}
