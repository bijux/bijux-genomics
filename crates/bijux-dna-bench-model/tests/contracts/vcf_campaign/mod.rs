//! Owner: pure VCF campaign regressions grouped by policy concern.
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

fn matrix_fixture() -> BenchmarkMatrixReport {
    BenchmarkMatrixReport {
        schema_version: "bijux.hpc.benchmark_matrix.v1".to_string(),
        campaign_id: "campaign-vcf".to_string(),
        domain: "vcf".to_string(),
        domains: vec!["vcf".to_string()],
        generated_at: "0".to_string(),
        summary: BenchmarkMatrixSummary {
            total_rows: 20,
            readiness_counts: std::collections::BTreeMap::new(),
            domain_counts: std::collections::BTreeMap::new(),
        },
        rows: vec![
            row("v1", "vcf.call", "ready"),
            row("v2", "vcf.filter", "degraded"),
            row("v3", "vcf.stats", "ready"),
            row("v4", "vcf.postprocess", "ready"),
            row("v5", "vcf.prepare_reference_panel", "degraded"),
            row("v6", "vcf.damage_filter", "ready"),
            row("v7", "vcf.call_diploid", "refuse"),
            row("v8", "vcf.call_pseudohaploid", "ready"),
            row("v9", "vcf.call_gl", "ready"),
            row("v10", "vcf.gl_propagation", "ready"),
            row("v11", "vcf.phasing", "degraded"),
            row("v12", "vcf.impute", "degraded"),
            row("v13", "vcf.imputation_metrics", "ready"),
            row("v14", "vcf.qc", "ready"),
            row("v15", "vcf.pca", "degraded"),
            row("v16", "vcf.admixture", "ready"),
            row("v17", "vcf.population_structure", "ready"),
            row("v18", "vcf.roh", "degraded"),
            row("v19", "vcf.ibd", "ready"),
            row("v20", "vcf.demography", "refuse"),
        ],
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

mod admission;
mod annotation;
mod assembly;
mod calling;
mod cohort_analysis;
mod evidence;
mod haplotypes;
mod variant_admission;
