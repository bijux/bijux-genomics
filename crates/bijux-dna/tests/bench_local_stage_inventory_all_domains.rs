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
fn bench_local_stage_inventory_all_domains_json_reports_separate_governed_counts() {
    let payload =
        run_cli_json(&["bench", "local", "list-stages", "--domain", "fastq,bam,vcf", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_all_domain_stage_inventory.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("target/bench-readiness/all-domain-stage-list.json")
    );
    assert_eq!(
        payload
            .get("selected_domains")
            .and_then(serde_json::Value::as_array)
            .map(std::vec::Vec::len),
        Some(3)
    );

    let domain_counts = payload
        .get("domain_counts")
        .and_then(serde_json::Value::as_object)
        .expect("domain_counts object");
    assert_eq!(domain_counts.get("fastq").and_then(serde_json::Value::as_u64), Some(27));
    assert_eq!(domain_counts.get("bam").and_then(serde_json::Value::as_u64), Some(24));
    assert_eq!(domain_counts.get("vcf").and_then(serde_json::Value::as_u64), Some(20));
    assert_eq!(payload.get("total_stage_count").and_then(serde_json::Value::as_u64), Some(71));
    assert_eq!(
        payload.get("inventories").and_then(serde_json::Value::as_array).map(std::vec::Vec::len),
        Some(3)
    );
}
