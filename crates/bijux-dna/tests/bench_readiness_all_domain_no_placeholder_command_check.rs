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
fn bench_readiness_all_domain_no_placeholder_command_check_reports_clean_active_commands() {
    let payload = run_cli_json(&[
        "bench",
        "readiness",
        "render-all-domain-no-placeholder-command-check",
        "--json",
    ]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.all_domain_no_placeholder_command_check.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/all-domains/no-placeholder-command-check.json")
    );
    assert_eq!(
        payload.get("script_output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/rendered-commands-all-domains.sh")
    );
    assert_eq!(
        payload.get("argv_output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/rendered-commands-all-domains.argv.jsonl")
    );
    let row_count = support::json_u64(&payload, "row_count").expect("row_count");
    assert_eq!(payload.get("result_id_count").and_then(serde_json::Value::as_u64), Some(row_count));
    assert!(payload
        .get("stage_count")
        .and_then(serde_json::Value::as_u64)
        .is_some_and(|count| count >= 61));
    assert_eq!(payload.get("tool_count").and_then(serde_json::Value::as_u64), Some(68));
    assert_eq!(payload.get("command_step_count").and_then(serde_json::Value::as_u64), Some(143));
    assert_eq!(
        payload.get("shell_wrapped_step_count").and_then(serde_json::Value::as_u64),
        Some(84)
    );
    assert_eq!(payload.get("direct_step_count").and_then(serde_json::Value::as_u64), Some(59));
    assert_eq!(payload.get("valid_row_count").and_then(serde_json::Value::as_u64), Some(row_count));
    assert_eq!(payload.get("invalid_row_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("violation_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));

    let domain_counts = support::json_object(&payload, "domain_counts");
    assert_eq!(support::object_u64(domain_counts, "fastq"), Some(63));
    assert_eq!(support::object_u64(domain_counts, "bam"), Some(49));
    assert_eq!(support::object_u64_sum(domain_counts), row_count);

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
        command_source_counts
            .get("vcf_imputation_family_adapter")
            .and_then(serde_json::Value::as_u64),
        Some(2)
    );
    assert_eq!(
        command_source_counts
            .get("vcf_phasing_family_adapter")
            .and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(
        command_source_counts.get("vcf_plink_family_adapter").and_then(serde_json::Value::as_u64),
        Some(2)
    );

    let finding_type_counts = payload
        .get("finding_type_counts")
        .and_then(serde_json::Value::as_object)
        .expect("finding type counts");
    assert!(finding_type_counts.is_empty(), "active commands must not retain placeholder findings");

    let rows = support::json_array(&payload, "rows");
    assert_eq!(rows.len() as u64, row_count);
    assert!(rows.iter().all(|row| {
        row.get("has_real_invocation").and_then(serde_json::Value::as_bool) == Some(true)
            && row.get("finding_count").and_then(serde_json::Value::as_u64) == Some(0)
            && row.get("ok").and_then(serde_json::Value::as_bool) == Some(true)
    }));

    assert!(rows.iter().any(|row| {
        row.get("result_id").and_then(serde_json::Value::as_str)
            == Some("fastq:corpus-02-edna-mini:fastq.screen_taxonomy:sample-set:kraken2")
            && row.get("command_source").and_then(serde_json::Value::as_str)
                == Some("fastq_bam_command_adapter")
            && row.get("command_step_count").and_then(serde_json::Value::as_u64) == Some(1)
            && row.get("script_command_count").and_then(serde_json::Value::as_u64) == Some(1)
            && row.get("step_audits").and_then(serde_json::Value::as_array).is_some_and(|steps| {
                steps.len() == 1
                    && steps[0].get("executable").and_then(serde_json::Value::as_str) == Some("sh")
                    && steps[0].get("is_shell_wrapper").and_then(serde_json::Value::as_bool)
                        == Some(true)
                    && steps[0].get("has_real_invocation").and_then(serde_json::Value::as_bool)
                        == Some(true)
            })
    }));
    assert!(rows.iter().any(|row| {
        row.get("result_id").and_then(serde_json::Value::as_str)
            == Some("bam:corpus-01-kinship-mini:bam.kinship:sample-set:king")
            && row.get("command_source").and_then(serde_json::Value::as_str)
                == Some("fastq_bam_command_adapter")
            && row.get("step_audits").and_then(serde_json::Value::as_array).is_some_and(|steps| {
                steps.len() == 1
                    && steps[0].get("executable").and_then(serde_json::Value::as_str)
                        == Some("cargo")
                    && steps[0].get("is_shell_wrapper").and_then(serde_json::Value::as_bool)
                        == Some(false)
            })
    }));
    assert!(rows.iter().any(|row| {
        row.get("result_id").and_then(serde_json::Value::as_str)
            == Some("vcf:vcf_production_regression:vcf.postprocess:vcf_single_sample:bcftools")
            && row.get("command_source").and_then(serde_json::Value::as_str)
                == Some("vcf_bcftools_adapter")
            && row.get("command_step_count").and_then(serde_json::Value::as_u64) == Some(2)
            && row.get("script_command_count").and_then(serde_json::Value::as_u64) == Some(2)
            && row.get("step_audits").and_then(serde_json::Value::as_array).is_some_and(|steps| {
                steps.len() == 2
                    && steps.iter().all(|step| {
                        step.get("executable").and_then(serde_json::Value::as_str)
                            == Some("bcftools")
                            && step.get("is_shell_wrapper").and_then(serde_json::Value::as_bool)
                                == Some(false)
                            && step.get("has_real_invocation").and_then(serde_json::Value::as_bool)
                                == Some(true)
                    })
            })
    }));

    let violations =
        payload.get("violations").and_then(serde_json::Value::as_array).expect("violations");
    assert!(
        violations.is_empty(),
        "all active commands must stay free of todo, placeholder, echo-only, unconditional-success, empty executable, and missing-invocation drift"
    );
}
