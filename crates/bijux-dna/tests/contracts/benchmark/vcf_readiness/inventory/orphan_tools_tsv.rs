#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::process::Command;

use crate::support;

#[test]
fn bench_readiness_vcf_orphan_tools_writes_governed_tsv_columns() {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");

    let output = Command::new(support::cli_binary_path())
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(["bench", "readiness", "render-vcf-orphan-tools"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let tsv_path = repo_root.join("artifacts/benchmark-smoke/readiness/vcf/vcf-orphan-tools.tsv");
    assert!(tsv_path.is_file(), "VCF orphan tools TSV must exist");

    let tsv = std::fs::read_to_string(&tsv_path).expect("read VCF orphan tools");
    let mut lines = tsv.lines();
    assert_eq!(lines.next(), Some("tool_id\tregistered_binary\tserved_stage_count\tdecision"));
    let rows = lines.collect::<Vec<_>>();
    assert!(rows.is_empty(), "every governed VCF tool must serve a supported stage");
}
