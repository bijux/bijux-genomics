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
fn bench_readiness_all_domain_output_declarations_tracks_governed_rows() {
    let payload =
        run_cli_json(&["bench", "readiness", "render-all-domain-output-declarations", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.all_domain_output_declarations.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("target/bench-readiness/output-declarations-all-domains.tsv")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(120));
    assert_eq!(payload.get("result_id_count").and_then(serde_json::Value::as_u64), Some(120));
    assert_eq!(payload.get("complete_row_count").and_then(serde_json::Value::as_u64), Some(120));
    assert_eq!(payload.get("incomplete_row_count").and_then(serde_json::Value::as_u64), Some(0));

    let domain_counts =
        payload.get("domain_counts").and_then(serde_json::Value::as_object).expect("domain counts");
    assert_eq!(domain_counts.get("fastq").and_then(serde_json::Value::as_u64), Some(63));
    assert_eq!(domain_counts.get("bam").and_then(serde_json::Value::as_u64), Some(49));
    assert_eq!(domain_counts.get("vcf").and_then(serde_json::Value::as_u64), Some(8));

    let status_counts =
        payload.get("status_counts").and_then(serde_json::Value::as_object).expect("status counts");
    assert_eq!(status_counts.get("complete").and_then(serde_json::Value::as_u64), Some(120));

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 120);
    let result_ids = rows
        .iter()
        .filter_map(|row| row.get("result_id").and_then(serde_json::Value::as_str))
        .collect::<BTreeSet<_>>();
    assert_eq!(result_ids.len(), 120);

    let taxonomy = rows
        .iter()
        .find(|row| {
            row.get("result_id").and_then(serde_json::Value::as_str)
                == Some("fastq:corpus-02-edna-mini:fastq.screen_taxonomy:sample-set:kraken2")
        })
        .expect("taxonomy row");
    assert_eq!(taxonomy.get("status").and_then(serde_json::Value::as_str), Some("complete"));
    assert!(taxonomy.get("raw_outputs").and_then(serde_json::Value::as_array).is_some_and(
        |outputs| outputs.iter().any(|value| value.as_str() == Some("screen_report_tsv"))
    ));
    assert_eq!(
        taxonomy
            .get("normalized_metrics")
            .and_then(serde_json::Value::as_array)
            .and_then(|items| items.first())
            .and_then(serde_json::Value::as_str),
        Some("classification_report_json")
    );
    assert_eq!(
        taxonomy.get("manifest").and_then(serde_json::Value::as_str),
        Some(
            "target/slurm-dry-run/runs/local-benchmark-dry-run/corpus-02-edna-mini/fastq.screen_taxonomy/sample-set/kraken2/stage-result.json"
        )
    );
    assert_eq!(
        taxonomy.get("index_outputs").and_then(serde_json::Value::as_array).map(Vec::len),
        Some(0)
    );

    let kinship = rows
        .iter()
        .find(|row| {
            row.get("result_id").and_then(serde_json::Value::as_str)
                == Some("bam:corpus-01-kinship-mini:bam.kinship:sample-set:king")
        })
        .expect("kinship row");
    assert_eq!(kinship.get("status").and_then(serde_json::Value::as_str), Some("complete"));
    assert!(kinship
        .get("raw_outputs")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|outputs| outputs.iter().any(|value| value.as_str() == Some("summary"))));
    assert_eq!(
        kinship
            .get("normalized_metrics")
            .and_then(serde_json::Value::as_array)
            .and_then(|items| items.first())
            .and_then(serde_json::Value::as_str),
        Some("kinship_report")
    );

    let vcf_call = rows
        .iter()
        .find(|row| {
            row.get("result_id").and_then(serde_json::Value::as_str)
                == Some("vcf:vcf_production_regression:vcf.call:bam_bundle:bcftools")
        })
        .expect("VCF call row");
    assert_eq!(vcf_call.get("status").and_then(serde_json::Value::as_str), Some("complete"));
    assert!(vcf_call
        .get("raw_outputs")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|outputs| outputs.iter().any(|value| value.as_str() == Some("called_vcf"))));
    assert!(vcf_call
        .get("normalized_metrics")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|outputs| outputs.iter().any(|value| value.as_str() == Some("called_vcf"))));
    assert!(vcf_call.get("index_outputs").and_then(serde_json::Value::as_array).is_some_and(
        |outputs| outputs.iter().any(|value| value.as_str() == Some("called_vcf_tbi"))
    ));
    assert_eq!(
        vcf_call.get("manifest").and_then(serde_json::Value::as_str),
        Some(
            "target/slurm-dry-run/runs/local-benchmark-dry-run/vcf_production_regression/vcf.call/bam_bundle/bcftools/stage-result.json"
        )
    );
}
