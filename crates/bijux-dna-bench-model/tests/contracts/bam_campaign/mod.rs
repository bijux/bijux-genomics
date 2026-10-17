//! Owner: direct BAM campaign policy regressions grouped by responsibility.
#![allow(clippy::expect_used)]
use super::*;
use crate::{
    AppraisalReport, AppraisalSummary, BenchmarkMatrixReport, BenchmarkMatrixRow,
    BenchmarkMatrixSummary, BenchmarkReadiness, BenchmarkSurfaceMatch, HardeningQueueReport,
    APPRAISAL_REPORT_SCHEMA_V1, HARDENING_QUEUE_REPORT_SCHEMA_V1,
};

fn row(row_id: &str, stage_id: &str, readiness: &str) -> BenchmarkMatrixRow {
    BenchmarkMatrixRow {
        row_id: row_id.to_string(),
        matrix_domain: "bam".to_string(),
        stage_id: stage_id.to_string(),
        tool_id: "samtools_v1".to_string(),
        corpus_match: BenchmarkSurfaceMatch {
            required_profile: "wgs".to_string(),
            matched_profile: "wgs".to_string(),
            ready: true,
        },
        database_match: BenchmarkSurfaceMatch {
            required_profile: "align".to_string(),
            matched_profile: "align".to_string(),
            ready: true,
        },
        image_match: BenchmarkSurfaceMatch {
            required_profile: "tool-images".to_string(),
            matched_profile: "samtools".to_string(),
            ready: true,
        },
        readiness: BenchmarkReadiness { class: readiness.to_string(), reasons: Vec::new() },
        repetitions: if readiness == "refuse" { 0 } else { 3 },
    }
}

fn matrix_fixture() -> BenchmarkMatrixReport {
    BenchmarkMatrixReport {
        schema_version: "bijux.hpc.benchmark_matrix.v1".to_string(),
        campaign_id: "campaign-bam".to_string(),
        domain: "bam".to_string(),
        domains: vec!["bam".to_string()],
        generated_at: "0".to_string(),
        summary: BenchmarkMatrixSummary {
            total_rows: 16,
            readiness_counts: std::collections::BTreeMap::new(),
            domain_counts: std::collections::BTreeMap::new(),
        },
        rows: vec![
            row("b1", "bam.validate", "ready"),
            row("b2", "bam.align", "ready"),
            row("b3", "bam.qc_pre", "degraded"),
            row("b4", "bam.markdup", "ready"),
            row("b5", "bam.duplication_metrics", "ready"),
            row("b6", "bam.mapq_filter", "ready"),
            row("b7", "bam.mapping_summary", "ready"),
            row("b8", "bam.coverage", "degraded"),
            row("b9", "bam.validate", "refuse"),
            row("b10", "bam.damage", "ready"),
            row("b11", "bam.authenticity", "degraded"),
            row("b12", "bam.contamination", "ready"),
            row("b13", "bam.endogenous_content", "degraded"),
            row("b14", "bam.sex", "ready"),
            row("b15", "bam.haplogroups", "degraded"),
            row("b16", "bam.kinship", "ready"),
        ],
    }
}

fn valid_source_matrix() -> BenchmarkMatrixReport {
    BenchmarkMatrixReport {
        schema_version: "bijux.hpc.benchmark_matrix.v1".to_string(),
        campaign_id: "campaign-bam".to_string(),
        domain: "bam".to_string(),
        domains: vec!["bam".to_string()],
        generated_at: "2026-08-11T00:00:00Z".to_string(),
        summary: BenchmarkMatrixSummary {
            total_rows: 1,
            readiness_counts: BTreeMap::from([("ready".to_string(), 1)]),
            domain_counts: BTreeMap::from([("bam".to_string(), 1)]),
        },
        rows: vec![row("bam-row-1", "bam.validate", "ready")],
    }
}

fn valid_appraisal() -> AppraisalReport {
    AppraisalReport {
        schema_version: APPRAISAL_REPORT_SCHEMA_V1.to_string(),
        campaign_id: "campaign-bam".to_string(),
        domain: "bam".to_string(),
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
        campaign_id: "campaign-bam".to_string(),
        domain: "bam".to_string(),
        entries: Vec::new(),
    }
}

mod admission;
mod alignment;
mod ancient_dna;
mod assembly;
mod contamination;
mod filtering;
mod identity;
mod metrics;
mod resources;
mod validation;
