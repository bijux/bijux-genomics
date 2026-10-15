//! Owner: pure FASTQ campaign regressions grouped by policy concern.
#![cfg(test)]
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
        matrix_domain: "fastq".to_string(),
        stage_id: stage_id.to_string(),
        tool_id: "seqkit_v2".to_string(),
        corpus_match: BenchmarkSurfaceMatch {
            required_profile: "general".to_string(),
            matched_profile: "general".to_string(),
            ready: true,
        },
        database_match: BenchmarkSurfaceMatch {
            required_profile: "not-required".to_string(),
            matched_profile: "not-required".to_string(),
            ready: true,
        },
        image_match: BenchmarkSurfaceMatch {
            required_profile: "tool-images".to_string(),
            matched_profile: "seqkit".to_string(),
            ready: true,
        },
        readiness: BenchmarkReadiness { class: readiness.to_string(), reasons: Vec::new() },
        repetitions: if readiness == "refuse" { 0 } else { 3 },
    }
}

fn matrix_fixture() -> BenchmarkMatrixReport {
    BenchmarkMatrixReport {
        schema_version: "bijux.hpc.benchmark_matrix.v1".to_string(),
        campaign_id: "campaign-fastq".to_string(),
        domain: "fastq".to_string(),
        domains: vec!["fastq".to_string()],
        generated_at: "0".to_string(),
        summary: BenchmarkMatrixSummary {
            total_rows: 15,
            readiness_counts: std::collections::BTreeMap::new(),
            domain_counts: std::collections::BTreeMap::new(),
        },
        rows: vec![
            row("r1", "fastq.validate_reads", "ready"),
            row("r2", "fastq.normalize_read_names", "refuse"),
            row("r3", "fastq.trim_reads", "ready"),
            row("r4", "fastq.extract_umis", "ready"),
            row("r5", "fastq.detect_duplicates_premerge", "ready"),
            row("r6", "fastq.deplete_host", "degraded"),
            row("r7", "fastq.deplete_rrna", "ready"),
            row("r8", "fastq.deplete_reference_contaminants", "ready"),
            row("r9", "fastq.screen_taxonomy", "degraded"),
            row("r10", "fastq.normalize_primers", "ready"),
            row("r11", "fastq.filter_reads", "ready"),
            row("r15", "fastq.profile_reads", "ready"),
            row("r12", "fastq.materialize_qc_manifest", "ready"),
            row("r13", "fastq.capture_provenance_snapshot", "ready"),
            row("r14", "fastq.trim_terminal_damage", "ready"),
        ],
    }
}

fn valid_source_matrix() -> BenchmarkMatrixReport {
    BenchmarkMatrixReport {
        schema_version: "bijux.hpc.benchmark_matrix.v1".to_string(),
        campaign_id: "campaign-fastq".to_string(),
        domain: "fastq".to_string(),
        domains: vec!["fastq".to_string()],
        generated_at: "2026-08-11T00:00:00Z".to_string(),
        summary: BenchmarkMatrixSummary {
            total_rows: 1,
            readiness_counts: BTreeMap::from([("ready".to_string(), 1)]),
            domain_counts: BTreeMap::from([("fastq".to_string(), 1)]),
        },
        rows: vec![row("fastq-row-1", "fastq.validate_reads", "ready")],
    }
}

fn valid_appraisal() -> AppraisalReport {
    AppraisalReport {
        schema_version: APPRAISAL_REPORT_SCHEMA_V1.to_string(),
        campaign_id: "campaign-fastq".to_string(),
        domain: "fastq".to_string(),
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
        campaign_id: "campaign-fastq".to_string(),
        domain: "fastq".to_string(),
        entries: Vec::new(),
    }
}

mod admission;
mod assembly;
mod depletion;
mod evidence;
mod preprocessing;
mod read_layout;
mod sampling;
mod validation;
