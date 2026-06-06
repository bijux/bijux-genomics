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

#[test]
fn bench_local_vcf_smoke_root_writes_governed_json_file() {
    let output = run_cli(&["bench", "local", "render-vcf-smoke-root"]);
    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "target/local-smoke/vcf/SMOKE_ROOT.json"
    );

    let repo_root = support::repo_root().expect("repo root");
    let manifest_path = repo_root.join("target/local-smoke/vcf/SMOKE_ROOT.json");
    let raw = std::fs::read_to_string(&manifest_path).expect("read manifest");
    let parsed: serde_json::Value = serde_json::from_str(&raw).expect("parse manifest");

    assert_eq!(
        parsed.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_vcf_smoke_root.v1")
    );
    assert_eq!(parsed.get("stage_count").and_then(serde_json::Value::as_u64), Some(20));
    assert_eq!(parsed.get("tool_pair_count").and_then(serde_json::Value::as_u64), Some(20));

    let rows = parsed.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert!(rows.iter().all(|row| {
        row.get("stage_id")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|value| !value.trim().is_empty())
            && row
                .get("tool_id")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|value| !value.trim().is_empty())
            && row
                .get("pair_root")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|value| value.starts_with("target/local-smoke/vcf/"))
            && row
                .get("artifacts_root")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|value| value.starts_with("target/local-smoke/vcf/"))
            && row
                .get("result_manifest_path")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|value| value.ends_with("/stage-result.json"))
            && row
                .get("expected_outputs")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|values| !values.is_empty())
    }));

    let stats = rows
        .iter()
        .find(|row| row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.stats"))
        .expect("stats row");
    assert_eq!(stats.get("tool_id").and_then(serde_json::Value::as_str), Some("bcftools"));
    assert_eq!(stats.get("support_status").and_then(serde_json::Value::as_str), Some("supported"));
    assert_eq!(
        stats.get("local_smoke_mode").and_then(serde_json::Value::as_str),
        Some("vcf_cohort")
    );
    assert_eq!(
        stats.get("result_manifest_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/vcf/vcf.stats/bcftools/stage-result.json")
    );
}
