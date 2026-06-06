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
fn bench_readiness_vcf_orphan_tools_reports_governed_decisions() {
    let payload = run_cli_json(&["bench", "readiness", "render-vcf-orphan-tools", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.vcf_orphan_tools.v1")
    );
    assert_eq!(payload.get("domain").and_then(serde_json::Value::as_str), Some("vcf"));
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("target/bench-readiness/vcf-orphan-tools.tsv")
    );
    assert_eq!(payload.get("orphan_count").and_then(serde_json::Value::as_u64), Some(10));
    assert_eq!(payload.get("required_tool_count").and_then(serde_json::Value::as_u64), Some(16));
    assert_eq!(payload.get("registered_tool_count").and_then(serde_json::Value::as_u64), Some(16));
    assert_eq!(payload.get("served_tool_count").and_then(serde_json::Value::as_u64), Some(6));

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 10);
    assert!(rows.iter().all(|row| {
        row.get("served_stage_count").and_then(serde_json::Value::as_u64) == Some(0)
            && row.get("decision").and_then(serde_json::Value::as_str)
                == Some("future_not_benchmark_ready")
    }));

    let has_row =
        |tool_id: &str, registered_binary: &str, served_stage_count: u64, decision: &str| {
            rows.iter().any(|row| {
                row.get("tool_id").and_then(serde_json::Value::as_str) == Some(tool_id)
                    && row.get("registered_binary").and_then(serde_json::Value::as_str)
                        == Some(registered_binary)
                    && row.get("served_stage_count").and_then(serde_json::Value::as_u64)
                        == Some(served_stage_count)
                    && row.get("decision").and_then(serde_json::Value::as_str) == Some(decision)
            })
        };

    for (tool_id, registered_binary) in [
        ("angsd", "angsd"),
        ("eagle", "eagle"),
        ("eigensoft", "smartpca"),
        ("glimpse", "glimpse"),
        ("ibdhap", "ibdhap"),
        ("ibdseq", "ibdseq"),
        ("impute5", "impute5"),
        ("minimac4", "minimac4"),
        ("plink", "plink"),
        ("shapeit", "shapeit"),
    ] {
        assert!(
            has_row(tool_id, registered_binary, 0, "future_not_benchmark_ready"),
            "VCF orphan tool report must retain the governed orphan row for {tool_id}"
        );
    }
}
