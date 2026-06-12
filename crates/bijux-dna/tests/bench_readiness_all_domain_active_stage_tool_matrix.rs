#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn run_cli_json(args: &[&str]) -> serde_json::Value {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");

    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(args)
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    serde_json::from_slice(&output.stdout).expect("parse stdout as json")
}

#[test]
fn bench_readiness_all_domain_active_stage_tool_matrix_reports_governed_rows() {
    let payload = run_cli_json(&[
        "bench",
        "readiness",
        "render-all-domain-active-stage-tool-matrix",
        "--json",
    ]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.all_domain_active_stage_tool_matrix.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/all-domains/active-stage-tool-matrix.tsv")
    );
    let row_count = support::json_u64(&payload, "row_count").expect("row_count");
    assert_eq!(payload.get("stage_count").and_then(serde_json::Value::as_u64), Some(64));
    assert_eq!(payload.get("tool_count").and_then(serde_json::Value::as_u64), Some(69));

    let domain_counts = support::json_object(&payload, "domain_counts");
    assert_eq!(domain_counts.get("fastq").and_then(serde_json::Value::as_u64), Some(63));
    assert_eq!(domain_counts.get("bam").and_then(serde_json::Value::as_u64), Some(49));
    assert_eq!(domain_counts.get("vcf").and_then(serde_json::Value::as_u64), Some(20));
    assert_eq!(support::object_u64_sum(domain_counts), row_count);

    let status_counts = support::json_object(&payload, "status_counts");
    assert_eq!(
        status_counts.get("benchmark_ready").and_then(serde_json::Value::as_u64),
        Some(row_count)
    );
    assert_eq!(status_counts.len(), 1);

    let rows = support::json_array(&payload, "rows");
    assert_eq!(rows.len() as u64, row_count);

    assert!(rows.iter().any(|row| {
        row.get("domain").and_then(serde_json::Value::as_str) == Some("bam")
            && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.contamination")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("schmutzi")
            && row.get("corpus_id").and_then(serde_json::Value::as_str)
                == Some("corpus-01-adna-bam-mini")
            && row.get("asset_profile_id").and_then(serde_json::Value::as_str)
                == Some("reference_fasta+reference_panel")
            && row.get("adapter_id").and_then(serde_json::Value::as_str)
                == Some("bam.adapter.contamination")
            && row.get("parser_id").and_then(serde_json::Value::as_str)
                == Some("bam.parser.contamination")
            && row.get("schema_id").and_then(serde_json::Value::as_str)
                == Some("bam_contamination_normalized_v1")
            && row.get("status").and_then(serde_json::Value::as_str) == Some("benchmark_ready")
    }));

    assert!(rows.iter().any(|row| {
        row.get("domain").and_then(serde_json::Value::as_str) == Some("fastq")
            && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.trim_reads")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("trimmomatic")
            && row.get("corpus_id").and_then(serde_json::Value::as_str) == Some("corpus-01-mini")
            && row.get("asset_profile_id").and_then(serde_json::Value::as_str)
                == Some("corpus_only")
            && row.get("adapter_id").and_then(serde_json::Value::as_str)
                == Some("fastq.adapter.trim_reads")
            && row.get("parser_id").and_then(serde_json::Value::as_str)
                == Some("fastq.parser.trim_reads")
            && row.get("schema_id").and_then(serde_json::Value::as_str)
                == Some("fastq_trim_reads_v2")
            && row.get("status").and_then(serde_json::Value::as_str) == Some("benchmark_ready")
    }));

    assert!(rows.iter().any(|row| {
        row.get("domain").and_then(serde_json::Value::as_str) == Some("vcf")
            && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.stats")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bcftools")
            && row.get("corpus_id").and_then(serde_json::Value::as_str)
                == Some("vcf_production_regression")
            && row.get("asset_profile_id").and_then(serde_json::Value::as_str) == Some("vcf_cohort")
            && row.get("adapter_id").and_then(serde_json::Value::as_str)
                == Some("vcf.adapter.quality_control")
            && row.get("parser_id").and_then(serde_json::Value::as_str)
                == Some("vcf.parser.stats_report")
            && row.get("schema_id").and_then(serde_json::Value::as_str)
                == Some("bijux.schemas.bench.vcf-normalized-metrics.stats.v1")
            && row.get("status").and_then(serde_json::Value::as_str) == Some("benchmark_ready")
    }));
    assert!(rows.iter().any(|row| {
        row.get("domain").and_then(serde_json::Value::as_str) == Some("vcf")
            && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.postprocess")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bcftools")
            && row.get("corpus_id").and_then(serde_json::Value::as_str)
                == Some("vcf_production_regression")
            && row.get("asset_profile_id").and_then(serde_json::Value::as_str)
                == Some("vcf_single_sample")
            && row.get("adapter_id").and_then(serde_json::Value::as_str)
                == Some("vcf.adapter.transform")
            && row.get("parser_id").and_then(serde_json::Value::as_str)
                == Some("vcf.parser.vcf_output")
            && row.get("schema_id").and_then(serde_json::Value::as_str)
                == Some("bijux.schemas.bench.vcf-normalized-metrics.postprocess.v1")
            && row.get("status").and_then(serde_json::Value::as_str) == Some("benchmark_ready")
    }));
    assert!(rows.iter().any(|row| {
        row.get("domain").and_then(serde_json::Value::as_str) == Some("vcf")
            && row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("vcf.imputation_metrics")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("beagle")
            && row.get("asset_profile_id").and_then(serde_json::Value::as_str)
                == Some("vcf_cohort_with_panel")
            && row.get("status").and_then(serde_json::Value::as_str) == Some("benchmark_ready")
    }));
    assert!(rows.iter().any(|row| {
        row.get("domain").and_then(serde_json::Value::as_str) == Some("vcf")
            && row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("vcf.population_structure")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("plink2")
            && row.get("corpus_id").and_then(serde_json::Value::as_str)
                == Some("vcf_production_regression")
            && row.get("asset_profile_id").and_then(serde_json::Value::as_str) == Some("vcf_cohort")
            && row.get("adapter_id").and_then(serde_json::Value::as_str)
                == Some("vcf.adapter.population_structure")
            && row.get("parser_id").and_then(serde_json::Value::as_str)
                == Some("vcf.parser.report_json")
            && row.get("schema_id").and_then(serde_json::Value::as_str)
                == Some("bijux.schemas.bench.vcf-normalized-metrics.population-structure.v1")
            && row.get("status").and_then(serde_json::Value::as_str) == Some("benchmark_ready")
    }));

    assert!(
        rows.iter().all(|row| {
            row.get("status").and_then(serde_json::Value::as_str) == Some("benchmark_ready")
        }),
        "active scope must keep only benchmark-ready rows"
    );
    assert!(
        rows.iter().all(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) != Some("fastq.index_reference")
                && row.get("stage_id").and_then(serde_json::Value::as_str)
                    != Some("fastq.profile_overrepresented_sequences")
                && row.get("stage_id").and_then(serde_json::Value::as_str)
                    != Some("fastq.report_qc")
        }),
        "active scope must exclude not-benchmark-ready-only stages"
    );
}
