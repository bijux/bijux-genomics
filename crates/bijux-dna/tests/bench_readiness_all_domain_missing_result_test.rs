#![allow(clippy::expect_used)]

use std::collections::BTreeSet;
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
fn bench_readiness_all_domain_missing_result_test_tracks_three_governed_missing_rows() {
    let payload = run_cli_json(&[
        "bench",
        "readiness",
        "render-all-domain-missing-result-test",
        "--json",
    ]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.all_domain_missing_result_test.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("target/bench-readiness/missing-result-test-all-domains.json")
    );
    assert_eq!(
        payload.get("fake_result_root").and_then(serde_json::Value::as_str),
        Some("target/bench-readiness/missing-result-test-all-domains-fixture")
    );
    assert_eq!(payload.get("expected_row_count").and_then(serde_json::Value::as_u64), Some(120));
    assert_eq!(
        payload.get("present_result_row_count").and_then(serde_json::Value::as_u64),
        Some(117)
    );
    assert_eq!(
        payload.get("missing_result_row_count").and_then(serde_json::Value::as_u64),
        Some(3)
    );
    assert_eq!(payload.get("passes_behavior_test"), Some(&serde_json::Value::Bool(true)));

    let domain_counts =
        payload.get("domain_counts").and_then(serde_json::Value::as_object).expect("domain counts");
    assert_eq!(domain_counts.get("fastq").and_then(serde_json::Value::as_u64), Some(63));
    assert_eq!(domain_counts.get("bam").and_then(serde_json::Value::as_u64), Some(49));
    assert_eq!(domain_counts.get("vcf").and_then(serde_json::Value::as_u64), Some(8));

    let removed_result_ids = payload
        .get("removed_result_ids")
        .and_then(serde_json::Value::as_array)
        .expect("removed result ids")
        .iter()
        .map(|value| value.as_str().expect("removed result id"))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        removed_result_ids,
        [
            "fastq:corpus-02-edna-mini:fastq.screen_taxonomy:sample-set:kraken2",
            "bam:corpus-01-bam-mini:bam.coverage:sample-set:samtools",
            "vcf:vcf_production_regression:vcf.stats:vcf_cohort:bcftools",
        ]
        .into_iter()
        .collect()
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 120);

    let missing_rows = rows
        .iter()
        .filter(|row| row.get("result_status").and_then(serde_json::Value::as_str) == Some("missing_result"))
        .collect::<Vec<_>>();
    assert_eq!(missing_rows.len(), 3);
    assert_eq!(
        missing_rows
            .iter()
            .map(|row| row.get("domain").and_then(serde_json::Value::as_str).expect("domain"))
            .collect::<BTreeSet<_>>(),
        ["fastq", "bam", "vcf"].into_iter().collect()
    );
    assert!(missing_rows.iter().all(|row| {
        row.get("observed_output_artifact_ids")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|values| values.is_empty())
    }));

    let fastq_missing = rows
        .iter()
        .find(|row| {
            row.get("result_id").and_then(serde_json::Value::as_str)
                == Some("fastq:corpus-02-edna-mini:fastq.screen_taxonomy:sample-set:kraken2")
        })
        .expect("fastq missing row");
    assert_eq!(
        fastq_missing.get("report_section").and_then(serde_json::Value::as_str),
        Some("contamination_screening")
    );

    let bam_missing = rows
        .iter()
        .find(|row| {
            row.get("result_id").and_then(serde_json::Value::as_str)
                == Some("bam:corpus-01-bam-mini:bam.coverage:sample-set:samtools")
        })
        .expect("bam missing row");
    assert_eq!(
        bam_missing.get("stage_id").and_then(serde_json::Value::as_str),
        Some("bam.coverage")
    );

    let vcf_missing = rows
        .iter()
        .find(|row| {
            row.get("result_id").and_then(serde_json::Value::as_str)
                == Some("vcf:vcf_production_regression:vcf.stats:vcf_cohort:bcftools")
        })
        .expect("vcf missing row");
    assert_eq!(
        vcf_missing.get("report_section").and_then(serde_json::Value::as_str),
        Some("quality_control")
    );

    let present_row = rows
        .iter()
        .find(|row| {
            row.get("result_id").and_then(serde_json::Value::as_str)
                == Some("bam:corpus-01-bam-mini:bam.qc_pre:sample-set:multiqc")
        })
        .expect("present row");
    assert_eq!(
        present_row.get("result_status").and_then(serde_json::Value::as_str),
        Some("present")
    );
    assert!(present_row
        .get("observed_output_artifact_ids")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|values| !values.is_empty()));
}
