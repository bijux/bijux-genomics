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
fn bench_readiness_all_domain_adapter_coverage_reports_complete_active_rows() {
    let payload =
        run_cli_json(&["bench", "readiness", "render-all-domain-adapter-coverage", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.all_domain_adapter_coverage.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/all-domains/adapter-coverage.tsv")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(125));
    assert_eq!(payload.get("result_id_count").and_then(serde_json::Value::as_u64), Some(125));
    assert_eq!(payload.get("stage_count").and_then(serde_json::Value::as_u64), Some(58));
    assert_eq!(payload.get("tool_count").and_then(serde_json::Value::as_u64), Some(66));
    assert_eq!(
        payload.get("rendered_command_binding_count").and_then(serde_json::Value::as_u64),
        Some(125)
    );
    assert_eq!(payload.get("covered_row_count").and_then(serde_json::Value::as_u64), Some(125));
    assert_eq!(payload.get("missing_row_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("coverage_percent").and_then(serde_json::Value::as_f64), Some(100.0));
    assert_eq!(payload.get("violation_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));

    let domain_counts =
        payload.get("domain_counts").and_then(serde_json::Value::as_object).expect("domain counts");
    assert_eq!(domain_counts.get("fastq").and_then(serde_json::Value::as_u64), Some(63));
    assert_eq!(domain_counts.get("bam").and_then(serde_json::Value::as_u64), Some(49));
    assert_eq!(domain_counts.get("vcf").and_then(serde_json::Value::as_u64), Some(13));

    let command_source_counts = payload
        .get("command_source_counts")
        .and_then(serde_json::Value::as_object)
        .expect("command source counts");
    assert_eq!(
        command_source_counts.get("fastq_bam_command_adapter").and_then(serde_json::Value::as_u64),
        Some(112)
    );
    assert_eq!(
        command_source_counts.get("vcf_bcftools_adapter").and_then(serde_json::Value::as_u64),
        Some(11)
    );
    assert_eq!(
        command_source_counts.get("vcf_plink_family_adapter").and_then(serde_json::Value::as_u64),
        Some(2)
    );

    let coverage_status_counts = payload
        .get("coverage_status_counts")
        .and_then(serde_json::Value::as_object)
        .expect("coverage status counts");
    assert_eq!(
        coverage_status_counts.get("covered").and_then(serde_json::Value::as_u64),
        Some(125)
    );
    assert_eq!(coverage_status_counts.len(), 1);

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 125);
    assert!(rows.iter().all(|row| {
        row.get("coverage_status").and_then(serde_json::Value::as_str) == Some("covered")
    }));

    assert!(rows.iter().any(|row| {
        row.get("result_id").and_then(serde_json::Value::as_str)
            == Some("fastq:corpus-01-mini:fastq.trim_reads:sample-set:trimmomatic")
            && row.get("command_source").and_then(serde_json::Value::as_str)
                == Some("fastq_bam_command_adapter")
            && row.get("command_step_count").and_then(serde_json::Value::as_u64) == Some(1)
            && row.get("script_command_count").and_then(serde_json::Value::as_u64) == Some(1)
            && row
                .get("primary_executables")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|items| items.iter().any(|value| value.as_str() == Some("sh")))
    }));
    assert!(rows.iter().any(|row| {
        row.get("result_id").and_then(serde_json::Value::as_str)
            == Some("bam:corpus-01-adna-bam-mini:bam.contamination:sample-set:schmutzi")
            && row.get("command_source").and_then(serde_json::Value::as_str)
                == Some("fastq_bam_command_adapter")
            && row.get("command_step_count").and_then(serde_json::Value::as_u64) == Some(1)
            && row.get("script_command_count").and_then(serde_json::Value::as_u64) == Some(1)
            && row
                .get("primary_executables")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|items| items.iter().any(|value| value.as_str() == Some("/bin/sh")))
    }));
    assert!(rows.iter().any(|row| {
        row.get("result_id").and_then(serde_json::Value::as_str)
            == Some("vcf:vcf_production_regression:vcf.postprocess:vcf_single_sample:bcftools")
            && row.get("command_source").and_then(serde_json::Value::as_str)
                == Some("vcf_bcftools_adapter")
            && row
                .get("command_step_count")
                .and_then(serde_json::Value::as_u64)
                .is_some_and(|count| count >= 1)
            && row
                .get("script_command_count")
                .and_then(serde_json::Value::as_u64)
                .is_some_and(|count| count >= 1)
            && row
                .get("primary_executables")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|items| items.iter().any(|value| value.as_str() == Some("bcftools")))
    }));

    let violations =
        payload.get("violations").and_then(serde_json::Value::as_array).expect("violations");
    assert!(violations.is_empty(), "all active rows must retain executable command rendering");
}
