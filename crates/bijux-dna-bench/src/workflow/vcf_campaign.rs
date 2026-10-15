//! Owner: admission and atomic publication of persisted VCF campaign reports.
use anyhow::{Context, Result};
pub use bijux_dna_bench_model::derive_vcf_benchmark_campaign;
use bijux_dna_bench_model::{validate_vcf_benchmark_campaign_report, VcfBenchmarkCampaignReport};
use serde::Serialize;
use std::path::Path;

fn publish_json(path: &Path, value: &impl Serialize) -> Result<()> {
    if let Some(parent) = path.parent().filter(|parent| !parent.as_os_str().is_empty()) {
        bijux_dna_infra::ensure_dir(parent)?;
    }
    let payload = serde_json::to_vec_pretty(value)?;
    bijux_dna_infra::atomic_write_bytes(path, &payload)
        .with_context(|| format!("publish {}", path.display()))
}

/// Load and admit a persisted VCF campaign report.
///
/// # Errors
/// Returns an error when the report cannot be read, parsed, or admitted.
pub fn load_vcf_benchmark_campaign_report(path: &Path) -> Result<VcfBenchmarkCampaignReport> {
    let raw = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
    let report: VcfBenchmarkCampaignReport =
        serde_json::from_slice(&raw).with_context(|| format!("parse {}", path.display()))?;
    validate_vcf_benchmark_campaign_report(&report)
        .with_context(|| format!("validate {}", path.display()))?;
    Ok(report)
}

/// Validate and atomically publish a VCF campaign report.
///
/// # Errors
/// Returns an error when admission or atomic publication fails.
pub fn publish_vcf_benchmark_campaign_report(
    path: &Path,
    report: &VcfBenchmarkCampaignReport,
) -> Result<()> {
    validate_vcf_benchmark_campaign_report(report)
        .context("validate VCF campaign report publication")?;
    publish_json(path, report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AppraisalReport, AppraisalSummary, BenchmarkMatrixReport, BenchmarkMatrixRow,
        BenchmarkMatrixSummary, BenchmarkReadiness, BenchmarkSurfaceMatch, HardeningQueueReport,
        APPRAISAL_REPORT_SCHEMA_V1, HARDENING_QUEUE_REPORT_SCHEMA_V1,
    };
    use std::collections::BTreeMap;

    fn row(row_id: &str, stage_id: &str, readiness: &str) -> BenchmarkMatrixRow {
        BenchmarkMatrixRow {
            row_id: row_id.to_string(),
            matrix_domain: "vcf".to_string(),
            stage_id: stage_id.to_string(),
            tool_id: "bcftools_v1".to_string(),
            corpus_match: BenchmarkSurfaceMatch {
                required_profile: "wgs".to_string(),
                matched_profile: "wgs".to_string(),
                ready: true,
            },
            database_match: BenchmarkSurfaceMatch {
                required_profile: "vcf".to_string(),
                matched_profile: "vcf".to_string(),
                ready: true,
            },
            image_match: BenchmarkSurfaceMatch {
                required_profile: "tool-images".to_string(),
                matched_profile: "bcftools".to_string(),
                ready: true,
            },
            readiness: BenchmarkReadiness { class: readiness.to_string(), reasons: Vec::new() },
            repetitions: if readiness == "refuse" { 0 } else { 5 },
        }
    }

    fn valid_source_matrix() -> BenchmarkMatrixReport {
        BenchmarkMatrixReport {
            schema_version: "bijux.hpc.benchmark_matrix.v1".to_string(),
            campaign_id: "campaign-vcf".to_string(),
            domain: "vcf".to_string(),
            domains: vec!["vcf".to_string()],
            generated_at: "2026-08-11T00:00:00Z".to_string(),
            summary: BenchmarkMatrixSummary {
                total_rows: 1,
                readiness_counts: BTreeMap::from([("ready".to_string(), 1)]),
                domain_counts: BTreeMap::from([("vcf".to_string(), 1)]),
            },
            rows: vec![row("vcf-row-1", "vcf.call", "ready")],
        }
    }

    fn valid_appraisal() -> AppraisalReport {
        AppraisalReport {
            schema_version: APPRAISAL_REPORT_SCHEMA_V1.to_string(),
            campaign_id: "campaign-vcf".to_string(),
            domain: "vcf".to_string(),
            findings: Vec::new(),
            summary: AppraisalSummary {
                total_findings: 0,
                by_appraiser: BTreeMap::new(),
                by_severity: BTreeMap::new(),
            },
        }
    }

    fn valid_queue() -> HardeningQueueReport {
        HardeningQueueReport {
            schema_version: HARDENING_QUEUE_REPORT_SCHEMA_V1.to_string(),
            campaign_id: "campaign-vcf".to_string(),
            domain: "vcf".to_string(),
            entries: Vec::new(),
        }
    }

    #[test]
    fn derives_publishes_and_loads_admitted_vcf_campaign() -> anyhow::Result<()> {
        let report = derive_vcf_benchmark_campaign(
            valid_source_matrix(),
            &valid_appraisal(),
            &valid_queue(),
            Some("G141"),
        )?;
        assert_eq!(report.summary.total_goals, 1);
        assert_eq!(report.summary.total_rows, 1);
        assert_eq!(report.entries[0].status, "ready-for-benchmark-run");

        let root = tempfile::tempdir()?;
        let path = root.path().join("vcf-campaign.json");
        publish_vcf_benchmark_campaign_report(&path, &report)?;
        assert_eq!(load_vcf_benchmark_campaign_report(&path)?, report);
        Ok(())
    }
}
