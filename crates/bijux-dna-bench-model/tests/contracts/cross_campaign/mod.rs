//! Owner: pure cross-domain campaign regressions grouped by policy concern.
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

fn matrix_fixture() -> BenchmarkMatrixReport {
    BenchmarkMatrixReport {
        schema_version: "bijux.hpc.benchmark_matrix.v1".to_string(),
        campaign_id: "campaign-cross".to_string(),
        domain: "cross".to_string(),
        domains: vec!["cross".to_string()],
        generated_at: "0".to_string(),
        summary: BenchmarkMatrixSummary {
            total_rows: 10,
            readiness_counts: std::collections::BTreeMap::new(),
            domain_counts: std::collections::BTreeMap::new(),
        },
        rows: vec![
            row("c1", BRIDGE_FASTQ_TRIM_TO_BAM_ALIGN, "ready"),
            row("c2", BRIDGE_FASTQ_VALIDATE_TO_BAM_ALIGN, "ready"),
            row("c3", BRIDGE_FASTQ_ADNA_TO_BAM_DAMAGE, "degraded"),
            row("c4", BRIDGE_BAM_GENOTYPING_TO_VCF_CALL, "ready"),
            row("c5", BRIDGE_FASTQ_TRIM_TO_VCF_GL, "degraded"),
            row("c6", BRIDGE_BAM_SUMMARY_TO_VCF_STATS, "ready"),
            row("c7", BRIDGE_FASTQ_PROFILE_TO_BAM_SUMMARY, "ready"),
            row("c8", BRIDGE_FASTQ_VALIDATE_TO_BAM_CONTAMINATION, "degraded"),
            row("c9", BRIDGE_BAM_CONTAMINATION_TO_VCF_FILTER, "ready"),
            row("c10", BRIDGE_VCF_FILTER_TO_VCF_STATS, "refuse"),
        ],
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

mod admission;
mod assembly;
mod domain_handoffs;
mod execution_policy;
mod reviewer_evidence;
mod source_integrity;
