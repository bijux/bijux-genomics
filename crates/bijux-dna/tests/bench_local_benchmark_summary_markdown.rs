#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[cfg(feature = "bam_downstream")]
#[test]
fn bench_local_render_benchmark_summary_writes_readable_markdown() {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");

    let output = Command::new("cargo")
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(["run", "-q", "-p", "bijux-dna", "--features", "bam_downstream", "--"])
        .args(["bench", "local", "render-benchmark-summary"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let markdown_path = repo_root.join("benchmarks/readiness/local-ready/benchmark-summary.md");
    assert!(markdown_path.is_file(), "benchmark summary markdown must exist");

    let markdown = std::fs::read_to_string(&markdown_path).expect("read benchmark summary");
    assert!(markdown.contains("# Local Benchmark Summary"));
    assert!(markdown.contains("- Stage count: `51`"));
    assert!(markdown.contains("- Ready stages: `51`"));
    assert!(markdown.contains("## Stage Readiness"));
    assert!(markdown.contains(
        "| Stage | Tool | Readiness Kind | Readiness Status | Runtime (s) | Memory (MB) | Failure Reason |"
    ));
    assert!(markdown.contains(
        "| `fastq.report_qc` | `multiqc` | `smoke` | `ready` | `1.0` | `4096.0` | `not_applicable` |"
    ));
}
