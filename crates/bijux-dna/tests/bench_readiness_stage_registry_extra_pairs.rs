#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn run_cli(args: &[&str]) -> std::process::Output {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");

    Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(args)
        .output()
        .expect("run cli")
}

fn run_cli_json(args: &[&str]) -> serde_json::Value {
    let output = run_cli(args);
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
fn bench_readiness_stage_registry_extra_pairs_reports_registry_domain_drift() {
    let payload =
        run_cli_json(&["bench", "readiness", "render-stage-registry-extra-pairs", "--json"]);
    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.stage_registry_extra_pairs.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("target/bench-readiness/stage-registry-extra-pairs.tsv")
    );
    assert_eq!(payload.get("extra_pair_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(false));

    let domain_counts = payload
        .get("domain_counts")
        .and_then(serde_json::Value::as_object)
        .expect("domain_counts object");
    assert_eq!(domain_counts.get("bam").and_then(serde_json::Value::as_u64), Some(1));
    assert!(domain_counts.get("fastq").is_none());

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 1, "governed stage-registry drift slice must retain one row");
    assert!(
        rows.iter().any(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.haplogroups")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("samtools")
                && row.get("contract_status").and_then(serde_json::Value::as_str)
                    == Some("pair_missing_from_contract")
                && row.get("intentional_override_status").and_then(serde_json::Value::as_str)
                    == Some("none")
        }),
        "bam.haplogroups / samtools must remain visible as a registry-only pair"
    );
    assert!(
        !rows.iter().any(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.qc_pre")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("multiqc")
        }),
        "bam.qc_pre / multiqc must no longer remain visible as a registry-only tool row"
    );
}
