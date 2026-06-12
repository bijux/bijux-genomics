#![allow(clippy::expect_used)]

#[cfg(feature = "bam_downstream")]
use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[cfg(feature = "bam_downstream")]
#[test]
fn bench_local_render_tool_comparison_template_writes_governed_tsv_columns() {
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
        .args(["bench", "local", "render-tool-comparison-template"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let tsv_path = repo_root.join("benchmarks/readiness/local-ready/tool-comparison-template.tsv");
    assert!(tsv_path.is_file(), "tool comparison template must exist");

    let tsv = std::fs::read_to_string(&tsv_path).expect("read tool comparison template");
    let mut lines = tsv.lines();
    assert_eq!(
        lines.next(),
        Some(
            "stage_id\ttool_id\truntime_seconds\tmemory_mb\toutput_metric\tstatus\tfailure_reason"
        )
    );
    let rows = lines.collect::<Vec<_>>();
    assert_eq!(rows.len(), 51);
    assert!(rows.iter().any(|row| row.starts_with("fastq.report_qc\tmultiqc\t")));
}
