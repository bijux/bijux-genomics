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
fn bench_readiness_full_benchmark_dashboard_writes_markdown_and_json_outputs() {
    let output = run_cli(&["bench", "readiness", "render-full-benchmark-dashboard"]);
    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let rendered_path = String::from_utf8(output.stdout).expect("stdout utf8");
    assert_eq!(rendered_path.trim(), "benchmarks/readiness/FASTQ_BAM_VCF_BENCHMARK_DASHBOARD.md");

    let repo_root = support::repo_root().expect("repo root");
    let markdown = std::fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read dashboard markdown");
    let json_path = repo_root.join("benchmarks/readiness/FASTQ_BAM_VCF_BENCHMARK_DASHBOARD.json");
    let json_payload = std::fs::read_to_string(&json_path).expect("read dashboard json");
    let json_value: serde_json::Value =
        serde_json::from_str(&json_payload).expect("parse dashboard json");

    assert!(markdown.contains("# Full Benchmark Dashboard"));
    assert!(markdown.contains("| metric | count | source path | source field | detail |"));
    assert!(markdown.contains("| total_stages | 71 |"));
    assert!(markdown.contains("| total_tools | 64 |"));
    assert!(markdown.contains("| total_expected_jobs | 121 |"));
    assert!(markdown.contains("| ready_jobs | 118 |"));
    assert!(markdown.contains("| blocked_jobs | 3 |"));
    assert!(markdown.contains("| missing_parsers | 0 |"));
    assert!(markdown.contains("| missing_adapters | 0 |"));
    assert!(markdown.contains("| missing_assets | 0 |"));
    assert!(markdown.contains("| failed_real_smokes | 0 |"));
    assert!(markdown.contains("Unsupported pairs tracked outside the expected-job slice: 1."));

    assert_eq!(
        json_value.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.full_benchmark_dashboard.v1")
    );
    assert_eq!(
        json_value.get("total_expected_jobs").and_then(serde_json::Value::as_u64),
        Some(121)
    );
}
