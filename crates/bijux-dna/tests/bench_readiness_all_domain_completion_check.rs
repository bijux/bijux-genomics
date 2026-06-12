#![allow(clippy::expect_used)]

use std::collections::BTreeMap;
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
fn bench_readiness_all_domain_completion_check_reports_governed_completion_rules() {
    let payload =
        run_cli_json(&["bench", "readiness", "render-all-domain-completion-check", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.all_domain_completion_check.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/completion-check-all-domains.json")
    );
    assert_eq!(
        payload.get("fixture_root").and_then(serde_json::Value::as_str),
        Some("runs/bench/readiness-probes/all-domains/completion-check")
    );
    let row_count = support::json_u64(&payload, "row_count").expect("row_count");
    let complete_row_count =
        support::json_u64(&payload, "complete_row_count").expect("complete_row_count");
    let incomplete_row_count =
        support::json_u64(&payload, "incomplete_row_count").expect("incomplete_row_count");
    assert_eq!(complete_row_count + incomplete_row_count, row_count);
    assert_eq!(incomplete_row_count, 5);
    assert_eq!(payload.get("passes_behavior_test"), Some(&serde_json::Value::Bool(true)));

    let domain_counts = support::json_object(&payload, "domain_counts");
    assert_eq!(domain_counts.get("fastq").and_then(serde_json::Value::as_u64), Some(66));
    assert_eq!(domain_counts.get("bam").and_then(serde_json::Value::as_u64), Some(49));
    assert_eq!(domain_counts.get("vcf").and_then(serde_json::Value::as_u64), Some(20));

    let failure_reason_counts = payload
        .get("failure_reason_counts")
        .and_then(serde_json::Value::as_object)
        .expect("failure reason counts");
    assert_eq!(
        failure_reason_counts.get("missing_declared_outputs").and_then(serde_json::Value::as_u64),
        Some(2)
    );
    for reason in [
        "missing_normalized_metrics",
        "missing_manifest",
        "required_files_incomplete",
        "execution_not_successful",
    ] {
        assert_eq!(
            failure_reason_counts.get(reason).and_then(serde_json::Value::as_u64),
            Some(1),
            "expected one seeded `{reason}` row"
        );
    }

    let mutations = payload
        .get("seeded_mutations")
        .and_then(serde_json::Value::as_array)
        .expect("seeded mutations");
    assert_eq!(mutations.len(), 5);
    let mutation_result_ids = mutations
        .iter()
        .map(|mutation| {
            (
                mutation
                    .get("mutation_id")
                    .and_then(serde_json::Value::as_str)
                    .expect("mutation id"),
                mutation.get("result_id").and_then(serde_json::Value::as_str).expect("result id"),
            )
        })
        .collect::<BTreeMap<_, _>>();
    assert_eq!(
        mutation_result_ids.get("missing_declared_output").copied(),
        Some("fastq:corpus-01-mini:fastq.profile_reads:sample-set:seqkit_stats")
    );
    assert_eq!(
        mutation_result_ids.get("missing_normalized_metrics").copied(),
        Some("fastq:corpus-02-edna-mini:fastq.screen_taxonomy:sample-set:kraken2")
    );
    assert_eq!(
        mutation_result_ids.get("missing_manifest").copied(),
        Some("vcf:vcf_production_regression:vcf.call:bam_bundle:bcftools")
    );
    assert_eq!(
        mutation_result_ids.get("required_file_empty").copied(),
        Some("bam:corpus-01-bam-mini:bam.coverage:sample-set:samtools")
    );
    assert_eq!(
        mutation_result_ids.get("execution_not_successful").copied(),
        Some("bam:corpus-01-bam-mini:bam.qc_pre:sample-set:multiqc")
    );

    let rows = support::json_array(&payload, "rows");
    assert_eq!(rows.len() as u64, row_count);

    let missing_declared = rows
        .iter()
        .find(|row| {
            row.get("result_id").and_then(serde_json::Value::as_str)
                == Some("fastq:corpus-01-mini:fastq.profile_reads:sample-set:seqkit_stats")
        })
        .expect("missing declared row");
    assert_eq!(
        missing_declared.get("completion_status").and_then(serde_json::Value::as_str),
        Some("incomplete")
    );
    assert!(missing_declared
        .get("missing_declared_output_ids")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|values| values.iter().any(|value| value.as_str() == Some("qc_tsv"))));

    let missing_normalized = rows
        .iter()
        .find(|row| {
            row.get("result_id").and_then(serde_json::Value::as_str)
                == Some("fastq:corpus-02-edna-mini:fastq.screen_taxonomy:sample-set:kraken2")
        })
        .expect("missing normalized row");
    assert!(missing_normalized
        .get("failure_reasons")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|values| values
            .iter()
            .any(|value| value.as_str() == Some("missing_normalized_metrics"))));
    assert_eq!(
        missing_normalized
            .get("present_normalized_metrics_count")
            .and_then(serde_json::Value::as_u64),
        Some(0)
    );

    let missing_manifest = rows
        .iter()
        .find(|row| {
            row.get("result_id").and_then(serde_json::Value::as_str)
                == Some("vcf:vcf_production_regression:vcf.call:bam_bundle:bcftools")
        })
        .expect("missing manifest row");
    assert_eq!(
        missing_manifest.get("manifest_exists").and_then(serde_json::Value::as_bool),
        Some(false)
    );

    let empty_required = rows
        .iter()
        .find(|row| {
            row.get("result_id").and_then(serde_json::Value::as_str)
                == Some("bam:corpus-01-bam-mini:bam.coverage:sample-set:samtools")
        })
        .expect("required file row");
    assert!(empty_required
        .get("required_files")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|values| {
            values.iter().any(|value| {
                value.get("file_id").and_then(serde_json::Value::as_str) == Some("command_script")
                    && value.get("exists").and_then(serde_json::Value::as_bool) == Some(true)
                    && value.get("non_empty").and_then(serde_json::Value::as_bool) == Some(false)
            })
        }));

    let execution_failed = rows
        .iter()
        .find(|row| {
            row.get("result_id").and_then(serde_json::Value::as_str)
                == Some("bam:corpus-01-bam-mini:bam.qc_pre:sample-set:multiqc")
        })
        .expect("execution row");
    assert_eq!(
        execution_failed.get("exit_code_zero").and_then(serde_json::Value::as_bool),
        Some(false)
    );
    assert_eq!(execution_failed.get("exit_code").and_then(serde_json::Value::as_i64), Some(23));
    assert_eq!(
        execution_failed.get("runtime_status").and_then(serde_json::Value::as_str),
        Some("failed")
    );
}
