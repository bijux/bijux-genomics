//! Owner: admission and atomic publication of persisted Cross-domain campaign reports.
use anyhow::{Context, Result};
pub use bijux_dna_bench_model::derive_cross_benchmark_campaign;
use bijux_dna_bench_model::{
    validate_cross_benchmark_campaign_report, CrossBenchmarkCampaignReport,
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

/// Load and admit a persisted cross-domain campaign report.
///
/// # Errors
/// Returns an error when the report cannot be read, parsed, or admitted.
pub fn load_cross_benchmark_campaign_report(path: &Path) -> Result<CrossBenchmarkCampaignReport> {
    let raw = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
    let report: CrossBenchmarkCampaignReport =
        serde_json::from_slice(&raw).with_context(|| format!("parse {}", path.display()))?;
    validate_cross_benchmark_campaign_report(&report)
        .with_context(|| format!("validate {}", path.display()))?;
    Ok(report)
}

/// Validate and atomically publish a cross-domain campaign report.
///
/// # Errors
/// Returns an error when admission or atomic publication fails.
pub fn publish_cross_benchmark_campaign_report(
    path: &Path,
    report: &CrossBenchmarkCampaignReport,
) -> Result<()> {
    validate_cross_benchmark_campaign_report(report)
        .context("validate cross-campaign report publication")?;
    publish_json(path, report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    const BRIDGE_FASTQ_TRIM_TO_BAM_ALIGN: &str = "fastq.trim_reads=>bam.align";
    use crate::{
        AppraisalReport, AppraisalSummary, BenchmarkMatrixReport, BenchmarkMatrixRow,
        BenchmarkMatrixSummary, BenchmarkReadiness, BenchmarkSurfaceMatch, HardeningQueueReport,
        APPRAISAL_REPORT_SCHEMA_V1, HARDENING_QUEUE_REPORT_SCHEMA_V1,
    };

    fn row(row_id: &str, stage_id: &str, readiness: &str) -> BenchmarkMatrixRow {
        BenchmarkMatrixRow {
            row_id: row_id.to_string(),
            matrix_domain: "cross".to_string(),
            stage_id: stage_id.to_string(),
            tool_id: "seqkit_v2=>bwa_v0".to_string(),
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
                matched_profile: "apptainer".to_string(),
                ready: true,
            },
            readiness: BenchmarkReadiness { class: readiness.to_string(), reasons: Vec::new() },
            repetitions: if readiness == "refuse" { 0 } else { 5 },
        }
    }

    fn valid_source_matrix() -> BenchmarkMatrixReport {
        BenchmarkMatrixReport {
            schema_version: "bijux.hpc.benchmark_matrix.v1".to_string(),
            campaign_id: "campaign-cross".to_string(),
            domain: "cross".to_string(),
            domains: vec!["cross".to_string()],
            generated_at: "2026-08-11T00:00:00Z".to_string(),
            summary: BenchmarkMatrixSummary {
                total_rows: 1,
                readiness_counts: BTreeMap::from([("ready".to_string(), 1)]),
                domain_counts: BTreeMap::from([("cross".to_string(), 1)]),
            },
            rows: vec![row("cross-row-1", BRIDGE_FASTQ_TRIM_TO_BAM_ALIGN, "ready")],
        }
    }

    #[test]
    fn derives_publishes_and_loads_admitted_cross_campaign() -> anyhow::Result<()> {
        let matrix = valid_source_matrix();
        let appraisal = AppraisalReport {
            schema_version: APPRAISAL_REPORT_SCHEMA_V1.to_string(),
            campaign_id: "campaign-cross".to_string(),
            domain: "cross".to_string(),
            findings: Vec::new(),
            summary: AppraisalSummary {
                total_findings: 0,
                by_appraiser: BTreeMap::new(),
                by_severity: BTreeMap::new(),
            },
        };
        let queue = HardeningQueueReport {
            schema_version: HARDENING_QUEUE_REPORT_SCHEMA_V1.to_string(),
            campaign_id: "campaign-cross".to_string(),
            domain: "cross".to_string(),
            entries: Vec::new(),
        };

        let report = derive_cross_benchmark_campaign(matrix, &appraisal, &queue, Some("G161"))?;
        assert_eq!(report.summary.total_goals, 1);
        assert_eq!(report.summary.total_rows, 1);
        assert_eq!(report.entries[0].status, "ready-for-benchmark-run");

        let root = tempfile::tempdir()?;
        let path = root.path().join("cross-campaign.json");
        publish_cross_benchmark_campaign_report(&path, &report)?;
        assert_eq!(load_cross_benchmark_campaign_report(&path)?, report);
        Ok(())
    }
}
