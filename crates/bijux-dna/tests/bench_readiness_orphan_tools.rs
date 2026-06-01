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
fn bench_readiness_orphan_tools_reports_governed_decisions() {
    let payload = run_cli_json(&["bench", "readiness", "render-orphan-tools", "--json"]);
    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.orphan_tools.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("target/bench-readiness/orphan-tools.tsv")
    );
    assert_eq!(payload.get("orphan_count").and_then(serde_json::Value::as_u64), Some(3));

    let domain_counts = payload
        .get("domain_counts")
        .and_then(serde_json::Value::as_object)
        .expect("domain_counts object");
    assert_eq!(
        domain_counts.get("bam").and_then(serde_json::Value::as_u64),
        Some(3),
        "the current orphan-tool slice must be entirely BAM-owned"
    );
    assert!(
        domain_counts.get("fastq").is_none(),
        "FASTQ currently has no orphan governed tools in the benchmark scope"
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 3, "the governed orphan slice must retain three BAM rows");
    assert!(
        rows.iter().any(|row| {
            row.get("domain").and_then(serde_json::Value::as_str) == Some("bam")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("addeam")
                && row.get("decision").and_then(serde_json::Value::as_str)
                    == Some("register_to_stage")
        }),
        "addeam must remain visible as a benchmark registration gap"
    );
    assert!(
        rows.iter().any(|row| {
            row.get("domain").and_then(serde_json::Value::as_str) == Some("bam")
                && row.get("tool_id").and_then(serde_json::Value::as_str)
                    == Some("damageprofiler")
                && row.get("benchmark_stage_ids").and_then(serde_json::Value::as_array)
                    == Some(
                        &vec![
                            serde_json::Value::String("bam.authenticity".to_string()),
                            serde_json::Value::String("bam.damage".to_string()),
                        ]
                    )
        }),
        "damageprofiler must retain both benchmarked BAM stages in the orphan report"
    );
}
