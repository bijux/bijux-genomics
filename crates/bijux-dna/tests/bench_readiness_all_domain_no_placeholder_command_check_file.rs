#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_all_domain_no_placeholder_command_check_writes_governed_json_file() {
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
        .args(["bench", "readiness", "render-all-domain-no-placeholder-command-check"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let rendered_path = String::from_utf8(output.stdout).expect("stdout utf8");
    assert_eq!(
        rendered_path.trim(),
        "benchmarks/readiness/all-domains/no-placeholder-command-check.json"
    );

    let payload = std::fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read no-placeholder command check report");
    let payload: serde_json::Value = serde_json::from_str(&payload).expect("parse report json");

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.all_domain_no_placeholder_command_check.v1")
    );
    let row_count =
        payload.get("row_count").and_then(serde_json::Value::as_u64).expect("row_count");
    let command_step_count = payload
        .get("command_step_count")
        .and_then(serde_json::Value::as_u64)
        .expect("command_step_count");
    let shell_wrapped_step_count = payload
        .get("shell_wrapped_step_count")
        .and_then(serde_json::Value::as_u64)
        .expect("shell_wrapped_step_count");
    let direct_step_count = payload
        .get("direct_step_count")
        .and_then(serde_json::Value::as_u64)
        .expect("direct_step_count");
    assert!(row_count >= 128);
    assert_eq!(command_step_count, shell_wrapped_step_count + direct_step_count);
    assert_eq!(payload.get("invalid_row_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("violation_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));

    let finding_type_counts = payload
        .get("finding_type_counts")
        .and_then(serde_json::Value::as_object)
        .expect("finding type counts");
    assert!(finding_type_counts.is_empty());

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows");
    assert!(rows.iter().any(|row| {
        row.get("result_id").and_then(serde_json::Value::as_str)
            == Some("fastq:corpus-02-edna-mini:fastq.screen_taxonomy:sample-set:kraken2")
            && row.get("step_audits").and_then(serde_json::Value::as_array).is_some_and(|steps| {
                steps.len() == 1
                    && steps[0]
                        .get("command_heads")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|heads| {
                            heads.iter().filter_map(serde_json::Value::as_str).collect::<Vec<_>>()
                                == vec!["set", "mkdir", "kraken2", "awk", "printf"]
                        })
            })
    }));
    assert!(rows.iter().any(|row| {
        row.get("result_id").and_then(serde_json::Value::as_str)
            == Some("vcf:vcf_production_regression:vcf.postprocess:vcf_single_sample:bcftools")
            && row.get("step_audits").and_then(serde_json::Value::as_array).is_some_and(|steps| {
                steps
                    .iter()
                    .filter_map(|step| step.get("step_id").and_then(serde_json::Value::as_str))
                    .collect::<Vec<_>>()
                    == vec!["fill_tags", "index_postprocess_vcf"]
            })
    }));
    assert!(rows.iter().any(|row| {
        row.get("result_id").and_then(serde_json::Value::as_str)
            == Some(
                "vcf:vcf_production_regression:vcf.imputation_metrics:vcf_cohort_with_panel:beagle",
            )
            && row.get("step_audits").and_then(serde_json::Value::as_array).is_some_and(|steps| {
                steps.len() == 3
                    && steps
                        .iter()
                        .map(|step| step.get("step_id").and_then(serde_json::Value::as_str))
                        .collect::<Vec<_>>()
                        == vec![
                            Some("impute"),
                            Some("index_imputed_vcf"),
                            Some("derive_imputation_metrics"),
                        ]
                    && steps[0].get("executable").and_then(serde_json::Value::as_str) == Some("sh")
                    && steps[1].get("executable").and_then(serde_json::Value::as_str)
                        == Some("bcftools")
                    && steps[2].get("executable").and_then(serde_json::Value::as_str) == Some("sh")
            })
    }));

    let violations =
        payload.get("violations").and_then(serde_json::Value::as_array).expect("violations");
    assert!(violations.is_empty());
}
