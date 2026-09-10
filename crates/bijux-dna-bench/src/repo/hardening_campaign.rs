//! Owner: admission and atomic publication of persisted hardening campaign reports.
use anyhow::{Context, Result};
use bijux_dna_bench_model::{
    validate_hardening_benchmark_campaign_report, HardeningBenchmarkCampaignReport,
};
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

/// Load and admit a persisted hardening campaign report.
///
/// # Errors
/// Returns an error when the report cannot be read, parsed, or admitted.
pub fn load_hardening_benchmark_campaign_report(
    path: &Path,
) -> Result<HardeningBenchmarkCampaignReport> {
    let raw = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
    let report: HardeningBenchmarkCampaignReport =
        serde_json::from_slice(&raw).with_context(|| format!("parse {}", path.display()))?;
    validate_hardening_benchmark_campaign_report(&report)
        .with_context(|| format!("validate {}", path.display()))?;
    Ok(report)
}

/// Validate and atomically publish a hardening campaign report.
///
/// # Errors
/// Returns an error when admission or atomic publication fails.
pub fn publish_hardening_benchmark_campaign_report(
    path: &Path,
    report: &HardeningBenchmarkCampaignReport,
) -> Result<()> {
    validate_hardening_benchmark_campaign_report(report)
        .context("validate hardening campaign report publication")?;
    publish_json(path, report)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]
    use super::*;
    use bijux_dna_bench_model::{derive_hardening_benchmark_campaign, HardeningQueueEntry};
    use bijux_dna_bench_model::{
        AppraisalReport, AppraisalSummary, BenchmarkMatrixReport, BenchmarkMatrixRow,
        BenchmarkMatrixSummary, BenchmarkReadiness, BenchmarkSurfaceMatch, HardeningQueueReport,
        APPRAISAL_REPORT_SCHEMA_V1, HARDENING_QUEUE_REPORT_SCHEMA_V1,
    };
    use std::collections::BTreeMap;
    fn row(row_id: &str, stage_id: &str, readiness: &str) -> BenchmarkMatrixRow {
        BenchmarkMatrixRow {
            row_id: row_id.to_string(),
            matrix_domain: if stage_id.contains("=>") {
                "cross".to_string()
            } else {
                stage_id.split('.').next().unwrap_or("fastq").to_string()
            },
            stage_id: stage_id.to_string(),
            tool_id: "tool_v1".to_string(),
            corpus_match: BenchmarkSurfaceMatch {
                required_profile: "general".to_string(),
                matched_profile: "general".to_string(),
                ready: true,
            },
            database_match: BenchmarkSurfaceMatch {
                required_profile: "general".to_string(),
                matched_profile: "general".to_string(),
                ready: true,
            },
            image_match: BenchmarkSurfaceMatch {
                required_profile: "tool-images".to_string(),
                matched_profile: "tool".to_string(),
                ready: true,
            },
            readiness: BenchmarkReadiness { class: readiness.to_string(), reasons: Vec::new() },
            repetitions: if readiness == "refuse" { 0 } else { 3 },
        }
    }

    fn valid_source_matrix() -> BenchmarkMatrixReport {
        BenchmarkMatrixReport {
            schema_version: "bijux.hpc.benchmark_matrix.v1".to_string(),
            campaign_id: "campaign-hardening".to_string(),
            domain: "all".to_string(),
            domains: vec!["fastq".to_string(), "vcf".to_string()],
            generated_at: "2026-08-11T00:00:00Z".to_string(),
            summary: BenchmarkMatrixSummary {
                total_rows: 2,
                readiness_counts: BTreeMap::from([("ready".to_string(), 2)]),
                domain_counts: BTreeMap::from([("fastq".to_string(), 1), ("vcf".to_string(), 1)]),
            },
            rows: vec![
                row("hardening-row-1", "fastq.validate_reads", "ready"),
                row("foreign-row-2", "vcf.stats", "ready"),
            ],
        }
    }

    fn valid_appraisal() -> AppraisalReport {
        AppraisalReport {
            schema_version: APPRAISAL_REPORT_SCHEMA_V1.to_string(),
            campaign_id: "campaign-hardening".to_string(),
            domain: "all".to_string(),
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
            campaign_id: "campaign-hardening".to_string(),
            domain: "all".to_string(),
            entries: vec![HardeningQueueEntry {
                queue_id: "hardening-0001".to_string(),
                severity: "warning".to_string(),
                failure_class: "readiness-degraded".to_string(),
                recommendation: "close the admitted hardening evidence gap".to_string(),
                affected_rows: vec!["foreign-row-2".to_string(), "hardening-row-1".to_string()],
                source_appraisers: vec!["failure-class".to_string()],
            }],
        }
    }

    #[test]
    fn derivation_rejects_misaligned_source_identity() {
        let mut queue = valid_queue();
        queue.campaign_id = "another-campaign".to_string();
        assert!(derive_hardening_benchmark_campaign(
            valid_source_matrix(),
            &valid_appraisal(),
            &queue,
            Some("G171"),
        )
        .is_err_and(|error| error.to_string().contains("source identities")));
    }

    #[test]
    fn derives_publishes_and_loads_admitted_hardening_campaign() -> anyhow::Result<()> {
        let report = derive_hardening_benchmark_campaign(
            valid_source_matrix(),
            &valid_appraisal(),
            &valid_queue(),
            Some("G171"),
        )?;
        assert_eq!(report.summary.total_goals, 1);
        assert_eq!(report.summary.total_rows, 1);
        assert_eq!(report.entries[0].status, "requires-hardening");
        assert_eq!(report.entries[0].hardening_entries[0].affected_rows, ["hardening-row-1"]);

        let root = tempfile::tempdir()?;
        let path = root.path().join("hardening-campaign.json");
        publish_hardening_benchmark_campaign_report(&path, &report)?;
        assert_eq!(load_hardening_benchmark_campaign_report(&path)?, report);
        Ok(())
    }
}
