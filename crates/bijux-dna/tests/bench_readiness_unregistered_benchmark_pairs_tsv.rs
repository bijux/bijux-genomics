#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_unregistered_benchmark_pairs_writes_governed_tsv_columns() {
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
        .args(["bench", "readiness", "render-unregistered-benchmark-pairs"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let tsv_path = repo_root.join("target/bench-readiness/unregistered-benchmark-pairs.tsv");
    assert!(tsv_path.is_file(), "unregistered benchmark pairs TSV must exist");

    let tsv = std::fs::read_to_string(&tsv_path).expect("read unregistered benchmark pairs");
    let mut lines = tsv.lines();
    assert_eq!(
        lines.next(),
        Some(
            "domain\tstage_id\ttool_id\tsupport_status\tregistry_status\tregistered_stage_ids\treason"
        )
    );
    let rows = lines.collect::<Vec<_>>();
    assert_eq!(rows.len(), 15, "TSV must retain the governed unregistered-pair row count");
    assert!(
        rows.iter().any(|row| {
            row == &"bam\tbam.genotyping\tangsd\tplanned\ttool_registered_pair_missing\tbam.kinship,bam.sex\tbenchmark matrix references `bam.genotyping` / `angsd` but configs/ci/registry/tool_registry.toml does not register that pair; registry status: tool_registered_pair_missing; registered stages for `angsd`: bam.kinship, bam.sex"
        }),
        "TSV must retain the governed bam.genotyping / angsd registry drift row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"fastq\tfastq.detect_duplicates_premerge\tbijux_dna\tplanned_contract\ttool_missing\t\tbenchmark matrix references `fastq.detect_duplicates_premerge` / `bijux_dna` but configs/ci/registry/tool_registry.toml does not register that pair; registry status: tool_missing; registered stages for `bijux_dna`: <none>"
        }),
        "TSV must retain the governed fastq.detect_duplicates_premerge / bijux_dna registry drift row"
    );
    for tool_id in ["fastp", "prinseq", "seqfu"] {
        assert!(
            !rows.iter().any(|row| {
                row.starts_with(&format!("fastq\tfastq.profile_read_lengths\t{tool_id}\t"))
            }),
            "TSV must no longer retain a registry-drift row for fastq.profile_read_lengths / {tool_id}"
        );
    }
}
