#![allow(clippy::expect_used, clippy::too_many_lines)]

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
        Some("artifacts/benchmark-smoke/readiness/all-domains/no-placeholder-command-check.json")
    );
    assert_eq!(
        payload.get("script_output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/rendered-commands-all-domains.sh")
    );
    assert_eq!(
        payload.get("argv_output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/rendered-commands-all-domains.argv.jsonl")
    );
    let rows = support::json_array(&payload, "rows");
    let row_count = u64::try_from(rows.len()).expect("row count fits u64");
    assert_eq!(support::json_u64(&payload, "row_count"), Some(row_count));
    let stage_count = u64::try_from(
        rows.iter()
            .filter_map(|row| row.get("stage_id").and_then(serde_json::Value::as_str))
            .collect::<BTreeSet<_>>()
            .len(),
    )
    .expect("stage count fits u64");
    let tool_count = u64::try_from(
        rows.iter()
            .filter_map(|row| row.get("tool_id").and_then(serde_json::Value::as_str))
            .collect::<BTreeSet<_>>()
            .len(),
    )
    .expect("tool count fits u64");
    assert_eq!(payload.get("result_id_count").and_then(serde_json::Value::as_u64), Some(row_count));
    assert_eq!(payload.get("stage_count").and_then(serde_json::Value::as_u64), Some(stage_count));
    assert_eq!(payload.get("tool_count").and_then(serde_json::Value::as_u64), Some(tool_count));
    let command_step_count =
        support::json_u64(&payload, "command_step_count").expect("command step count");
    let shell_wrapped_step_count =
        support::json_u64(&payload, "shell_wrapped_step_count").expect("shell-wrapped step count");
    let direct_step_count =
        support::json_u64(&payload, "direct_step_count").expect("direct step count");
    assert_eq!(shell_wrapped_step_count + direct_step_count, command_step_count);
    assert!(command_step_count >= row_count);
    assert_eq!(payload.get("valid_row_count").and_then(serde_json::Value::as_u64), Some(row_count));
    assert_eq!(payload.get("invalid_row_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("violation_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));

    let domain_counts = support::json_object(&payload, "domain_counts");
    let fastq_bam_count = support::object_u64(domain_counts, "fastq").expect("fastq count")
        + support::object_u64(domain_counts, "bam").expect("bam count");
    for domain in ["fastq", "bam", "vcf"] {
        let expected_count = u64::try_from(
            rows.iter()
                .filter(|row| row.get("domain").and_then(serde_json::Value::as_str) == Some(domain))
                .count(),
        )
        .expect("domain count fits u64");
        assert_eq!(support::object_u64(domain_counts, domain), Some(expected_count));
    }
    assert_eq!(support::object_u64_sum(domain_counts), row_count);

    let command_source_counts = payload
        .get("command_source_counts")
        .and_then(serde_json::Value::as_object)
        .expect("command source counts");
    assert_eq!(
        command_source_counts.get("fastq_bam_command_adapter").and_then(serde_json::Value::as_u64),
        Some(fastq_bam_count)
    );
    assert_eq!(
        command_source_counts.get("vcf_bcftools_adapter").and_then(serde_json::Value::as_u64),
        Some(11)
    );
    assert_eq!(
        command_source_counts.get("vcf_eigensoft_adapter").and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(
        command_source_counts
            .get("vcf_imputation_family_adapter")
            .and_then(serde_json::Value::as_u64),
        Some(2)
    );
    assert_eq!(
        command_source_counts.get("vcf_phasing_family_adapter").and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(
        command_source_counts.get("vcf_descent_family_adapter").and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(
        command_source_counts.get("vcf_plink_family_adapter").and_then(serde_json::Value::as_u64),
        Some(5)
    );

    let finding_type_counts = payload
        .get("finding_type_counts")
        .and_then(serde_json::Value::as_object)
        .expect("finding type counts");
    assert!(finding_type_counts.is_empty(), "active commands must not retain placeholder findings");

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
    assert!(rows.iter().any(|row| {
        row.get("result_id").and_then(serde_json::Value::as_str)
            == Some("vcf:vcf_production_regression:vcf.roh:vcf_cohort:plink2")
            && row.get("command_source").and_then(serde_json::Value::as_str)
                == Some("vcf_descent_family_adapter")
            && row.get("command_step_count").and_then(serde_json::Value::as_u64) == Some(1)
    }));

    let violations =
        payload.get("violations").and_then(serde_json::Value::as_array).expect("violations");
    assert!(
        violations.is_empty(),
        "all active commands must stay free of todo, placeholder, echo-only, unconditional-success, empty executable, and missing-invocation drift"
    );
}
