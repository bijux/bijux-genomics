#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_undercovered_stages_writes_governed_tsv_columns() {
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
        .args(["bench", "readiness", "render-undercovered-stages"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let tsv_path = repo_root.join("target/bench-readiness/undercovered-stages.tsv");
    assert!(tsv_path.is_file(), "undercovered stages TSV must exist");

    let tsv = std::fs::read_to_string(&tsv_path).expect("read undercovered stages");
    let mut lines = tsv.lines();
    assert_eq!(
        lines.next(),
        Some("domain\tstage_id\tvalid_tool_count\tregistered_tool_count\tvalid_tool_ids\tregistered_tool_ids\tmissing_tool_ids\treason")
    );
    let rows = lines.collect::<Vec<_>>();
    assert_eq!(rows.len(), 1, "TSV must retain the governed undercovered-stage row count");
    assert!(
        rows.iter().any(|row| {
            row == &"bam\tbam.overlap_correction\t2\t1\tbamutil,samtools\tbamutil\tsamtools\tstage `bam.overlap_correction` admits 2 governed tool options (bamutil, samtools) but only registers bamutil; add samtools to avoid a single-backend benchmark slice"
        }),
        "TSV must retain the governed overlap-correction undercoverage gap"
    );
    assert!(
        !rows.iter().any(|row| row.starts_with("bam\tbam.filter\t")),
        "TSV must not retain an undercovered-stage row for bam.filter"
    );
}
