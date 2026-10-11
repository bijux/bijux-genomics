use std::collections::BTreeMap;

use bijux_dna_bench::{
    appraise_matrix_report, hardening_queue_from_appraisal, load_appraisal_report,
    load_hardening_queue_report, publish_appraisal_report, publish_hardening_queue_report,
};
use bijux_dna_bench_model::{
    validate_appraisal_report, validate_hardening_queue_report, BenchmarkMatrixReport,
    BenchmarkMatrixRow, BenchmarkMatrixSummary, BenchmarkReadiness, BenchmarkSurfaceMatch,
    BENCHMARK_MATRIX_REPORT_SCHEMA_V1,
};

#[test]
fn appraisal_workflow_runs_every_registered_appraiser_and_groups_queue() -> anyhow::Result<()> {
    let report = appraise_matrix_report(matrix())?;
    validate_appraisal_report(&report)?;
    for appraiser in [
        "runtime-performance",
        "artifact-validity",
        "scientific-output",
        "reproducibility",
        "backend-equivalence",
        "failure-class",
        "corpus-suitability",
        "code-freeze",
    ] {
        assert!(report.summary.by_appraiser.contains_key(appraiser), "missing {appraiser}");
    }

    let queue = hardening_queue_from_appraisal(report)?;
    validate_hardening_queue_report(&queue)?;
    assert!(!queue.entries.is_empty());
    assert_eq!(queue.entries[0].queue_id, "hardening-0001");
    Ok(())
}

#[test]
fn appraisal_and_hardening_publications_validate_before_atomic_roundtrip() -> anyhow::Result<()> {
    let artifact_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark crate is outside the workspace"))?
        .join("artifacts/tests/bench-appraisal");
    bijux_dna_infra::ensure_dir(&artifact_root)?;
    let temp = tempfile::Builder::new().prefix("roundtrip-").tempdir_in(artifact_root)?;
    let appraisal = appraise_matrix_report(matrix())?;
    let queue = hardening_queue_from_appraisal(appraisal.clone())?;
    let appraisal_path = temp.path().join("appraisal.json");
    let queue_path = temp.path().join("hardening-queue.json");

    publish_appraisal_report(&appraisal_path, &appraisal)?;
    publish_hardening_queue_report(&queue_path, &queue)?;
    assert_eq!(load_appraisal_report(&appraisal_path)?, appraisal);
    assert_eq!(load_hardening_queue_report(&queue_path)?, queue);

    let mut invalid = appraisal;
    invalid.summary.total_findings = 0;
    let refused = temp.path().join("invalid.json");
    assert!(publish_appraisal_report(&refused, &invalid).is_err());
    assert!(!refused.exists());
    Ok(())
}

fn matrix() -> BenchmarkMatrixReport {
    let rows = vec![
        BenchmarkMatrixRow {
            row_id: "fastq.validate_reads::<unbound>".to_string(),
            matrix_domain: "fastq".to_string(),
            stage_id: "fastq.validate_reads".to_string(),
            tool_id: "<unbound>".to_string(),
            corpus_match: unavailable("general"),
            database_match: unavailable("tool-binding"),
            image_match: unavailable("tool-images"),
            readiness: BenchmarkReadiness {
                class: "refuse".to_string(),
                reasons: vec![
                    "corpus missing".to_string(),
                    "database missing".to_string(),
                    "image missing".to_string(),
                ],
            },
            repetitions: 0,
        },
        BenchmarkMatrixRow {
            row_id: "fastq.trim_reads::seqkit_v2".to_string(),
            matrix_domain: "fastq".to_string(),
            stage_id: "fastq.trim_reads".to_string(),
            tool_id: "seqkit_v2".to_string(),
            corpus_match: available("general"),
            database_match: available("not-required"),
            image_match: available("tool-images"),
            readiness: BenchmarkReadiness { class: "ready".to_string(), reasons: Vec::new() },
            repetitions: 2,
        },
    ];
    BenchmarkMatrixReport {
        schema_version: BENCHMARK_MATRIX_REPORT_SCHEMA_V1.to_string(),
        campaign_id: "campaign-fastq".to_string(),
        domain: "fastq".to_string(),
        domains: vec!["fastq".to_string()],
        generated_at: "1786400000".to_string(),
        summary: BenchmarkMatrixSummary {
            total_rows: rows.len(),
            readiness_counts: BTreeMap::from([("ready".to_string(), 1), ("refuse".to_string(), 1)]),
            domain_counts: BTreeMap::from([("fastq".to_string(), 2)]),
        },
        rows,
    }
}

fn available(profile: &str) -> BenchmarkSurfaceMatch {
    BenchmarkSurfaceMatch {
        required_profile: profile.to_string(),
        matched_profile: profile.to_string(),
        ready: true,
    }
}

fn unavailable(profile: &str) -> BenchmarkSurfaceMatch {
    BenchmarkSurfaceMatch {
        required_profile: profile.to_string(),
        matched_profile: "<missing>".to_string(),
        ready: false,
    }
}
