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
fn bench_readiness_all_domain_expected_benchmark_results_tracks_governed_rows() {
    let payload = run_cli_json(&[
        "bench",
        "readiness",
        "render-all-domain-expected-benchmark-results",
        "--json",
    ]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.all_domain_expected_benchmark_results.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/expected-benchmark-results-all-domains.tsv")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(121));
    assert_eq!(payload.get("result_id_count").and_then(serde_json::Value::as_u64), Some(121));
    assert_eq!(payload.get("stage_count").and_then(serde_json::Value::as_u64), Some(56));
    assert_eq!(payload.get("tool_count").and_then(serde_json::Value::as_u64), Some(64));
    assert_eq!(payload.get("corpus_count").and_then(serde_json::Value::as_u64), Some(9));
    assert_eq!(payload.get("asset_profile_count").and_then(serde_json::Value::as_u64), Some(11));

    let domain_counts =
        payload.get("domain_counts").and_then(serde_json::Value::as_object).expect("domain counts");
    assert_eq!(domain_counts.get("fastq").and_then(serde_json::Value::as_u64), Some(63));
    assert_eq!(domain_counts.get("bam").and_then(serde_json::Value::as_u64), Some(49));
    assert_eq!(domain_counts.get("vcf").and_then(serde_json::Value::as_u64), Some(9));

    let section_counts = payload
        .get("report_section_counts")
        .and_then(serde_json::Value::as_object)
        .expect("section counts");
    assert_eq!(section_counts.get("read_cleanup").and_then(serde_json::Value::as_u64), Some(37));
    assert_eq!(section_counts.get("variant_calling").and_then(serde_json::Value::as_u64), Some(4));

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 121);

    let result_ids = rows
        .iter()
        .filter_map(|row| row.get("result_id").and_then(serde_json::Value::as_str))
        .collect::<BTreeSet<_>>();
    assert_eq!(result_ids.len(), 121);

    let taxonomy = rows
        .iter()
        .find(|row| {
            row.get("result_id").and_then(serde_json::Value::as_str)
                == Some("fastq:corpus-02-edna-mini:fastq.screen_taxonomy:sample-set:kraken2")
        })
        .expect("taxonomy result row");
    assert_eq!(
        taxonomy.get("asset_profile_id").and_then(serde_json::Value::as_str),
        Some("database_artifact_id+taxonomy_database_root")
    );
    assert_eq!(
        taxonomy.get("report_section").and_then(serde_json::Value::as_str),
        Some("contamination_screening")
    );
    assert!(taxonomy.get("expected_outputs").and_then(serde_json::Value::as_array).is_some_and(
        |outputs| outputs.iter().any(|value| value.as_str() == Some("classification_report_json"))
    ));
    assert!(taxonomy.get("expected_metrics").and_then(serde_json::Value::as_array).is_some_and(
        |metrics| metrics.iter().any(|value| value.as_str() == Some("classified_read_fraction"))
    ));

    let kinship = rows
        .iter()
        .find(|row| {
            row.get("result_id").and_then(serde_json::Value::as_str)
                == Some("bam:corpus-01-kinship-mini:bam.kinship:sample-set:king")
        })
        .expect("kinship result row");
    assert_eq!(
        kinship.get("asset_profile_id").and_then(serde_json::Value::as_str),
        Some("reference_fasta+reference_panel")
    );
    assert_eq!(
        kinship.get("report_section").and_then(serde_json::Value::as_str),
        Some("sample_identity")
    );
    assert!(kinship.get("expected_outputs").and_then(serde_json::Value::as_array).is_some_and(
        |outputs| outputs.iter().any(|value| value.as_str() == Some("kinship_report"))
    ));
    assert!(kinship.get("expected_metrics").and_then(serde_json::Value::as_array).is_some_and(
        |metrics| metrics.iter().any(|value| value.as_str() == Some("observed_max_overlap_snps"))
    ));

    let vcf_call = rows
        .iter()
        .find(|row| {
            row.get("result_id").and_then(serde_json::Value::as_str)
                == Some("vcf:vcf_production_regression:vcf.call:bam_bundle:bcftools")
        })
        .expect("VCF call result row");
    assert_eq!(
        vcf_call.get("report_section").and_then(serde_json::Value::as_str),
        Some("variant_calling")
    );
    assert!(vcf_call
        .get("expected_outputs")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|outputs| outputs.iter().any(|value| value.as_str() == Some("called_vcf"))));
    assert!(vcf_call.get("expected_metrics").and_then(serde_json::Value::as_array).is_some_and(
        |metrics| metrics.iter().any(|value| value.as_str() == Some("variant_count"))
    ));

    let vcf_postprocess = rows
        .iter()
        .find(|row| {
            row.get("result_id").and_then(serde_json::Value::as_str)
                == Some("vcf:vcf_production_regression:vcf.postprocess:vcf_single_sample:bcftools")
        })
        .expect("VCF postprocess result row");
    assert_eq!(
        vcf_postprocess.get("report_section").and_then(serde_json::Value::as_str),
        Some("normalization")
    );
    assert!(vcf_postprocess
        .get("expected_outputs")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|outputs| outputs
            .iter()
            .any(|value| value.as_str() == Some("postprocess_vcf"))));
    assert!(vcf_postprocess
        .get("expected_metrics")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|metrics| metrics.iter().any(|value| value.as_str() == Some("readable_vcf"))));
}
