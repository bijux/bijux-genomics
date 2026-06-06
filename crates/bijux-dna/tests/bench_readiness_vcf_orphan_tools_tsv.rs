#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_vcf_orphan_tools_writes_governed_tsv_columns() {
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

    let tsv_path = repo_root.join("target/bench-readiness/vcf-orphan-tools.tsv");
    assert!(tsv_path.is_file(), "VCF orphan tools TSV must exist");

    let tsv = std::fs::read_to_string(&tsv_path).expect("read VCF orphan tools");
    let mut lines = tsv.lines();
    assert_eq!(lines.next(), Some("tool_id\tregistered_binary\tserved_stage_count\tdecision"));
    let rows = lines.collect::<Vec<_>>();
    assert_eq!(rows.len(), 11, "TSV must retain the governed VCF orphan row count");
    for row in [
        "angsd\tangsd\t0\tfuture_not_benchmark_ready",
        "beagle-imputation\tbeagle\t0\tfuture_not_benchmark_ready",
        "eagle\teagle\t0\tfuture_not_benchmark_ready",
        "eigensoft\tsmartpca\t0\tfuture_not_benchmark_ready",
        "glimpse\tglimpse\t0\tfuture_not_benchmark_ready",
        "ibdhap\tibdhap\t0\tfuture_not_benchmark_ready",
        "ibdseq\tibdseq\t0\tfuture_not_benchmark_ready",
        "impute5\timpute5\t0\tfuture_not_benchmark_ready",
        "minimac4\tminimac4\t0\tfuture_not_benchmark_ready",
        "plink\tplink\t0\tfuture_not_benchmark_ready",
        "shapeit\tshapeit\t0\tfuture_not_benchmark_ready",
    ] {
        assert!(
            rows.iter().any(|candidate| candidate == &row),
            "TSV must retain governed VCF orphan row: {row}"
        );
    }
}
