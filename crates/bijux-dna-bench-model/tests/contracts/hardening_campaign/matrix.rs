//! Owner: shared in-memory matrix inputs for hardening policy regression.
use super::*;
pub(super) fn row(row_id: &str, stage_id: &str, readiness: &str) -> BenchmarkMatrixRow {
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

pub(super) fn matrix_fixture() -> BenchmarkMatrixReport {
    BenchmarkMatrixReport {
        schema_version: "bijux.hpc.benchmark_matrix.v1".to_string(),
        campaign_id: "campaign-hardening".to_string(),
        domain: "all".to_string(),
        domains: vec![
            "fastq".to_string(),
            "bam".to_string(),
            "vcf".to_string(),
            "cross".to_string(),
        ],
        generated_at: "0".to_string(),
        summary: BenchmarkMatrixSummary {
            total_rows: 13,
            readiness_counts: std::collections::BTreeMap::new(),
            domain_counts: std::collections::BTreeMap::new(),
        },
        rows: vec![
            row("h1", "fastq.validate_reads", "ready"),
            row("h2", "fastq.trim_reads", "ready"),
            row("h3", "fastq.profile_reads", "degraded"),
            row("h4", "bam.validate", "ready"),
            row("h5", "bam.align", "degraded"),
            row("h6", "bam.mapping_summary", "ready"),
            row("h7", "bam.coverage", "degraded"),
            row("h8", "bam.contamination", "ready"),
            row("h9", "vcf.call", "ready"),
            row("h10", "vcf.filter", "refuse"),
            row("h11", "vcf.stats", "ready"),
            row("h12", "vcf.impute", "degraded"),
            row("h13", "vcf.postprocess", "ready"),
        ],
    }
}
