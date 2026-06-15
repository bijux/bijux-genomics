#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_corpus_incompatibility_writes_governed_tsv_columns() {
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
        .args(["bench", "readiness", "render-corpus-incompatibility"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let tsv_path = repo_root.join("benchmarks/readiness/corpus-incompatibility.tsv");
    assert!(tsv_path.is_file(), "corpus incompatibility TSV must exist");

    let tsv = std::fs::read_to_string(&tsv_path).expect("read corpus incompatibility TSV");
    let mut lines = tsv.lines();
    assert_eq!(
        lines.next(),
        Some("domain\tstage_id\ttool_id\tbenchmark_status\tsupport_status\tadapter_status\tparser_status\tincompatible_fixture_id\tincompatible_corpus_family_id\trequired_fixture_id\trequired_corpus_family_id\tincompatibility_kind\trequired_assets\trequired_contract\treason")
    );
    let rows = lines.collect::<Vec<_>>();
    assert_eq!(rows.len(), 324, "TSV must retain the governed incompatibility row count");
    assert!(
        rows.iter().any(|row| {
            row.contains("fastq\tfastq.infer_asvs\tdada2\tbenchmark_ready")
                && row.contains("\tcorpus-01-mini\tcorpus-01\tcorpus-03-amplicon-mini\tcorpus-03\tmissing_amplicon_asv_contract\t")
                && row.contains("expected_asvs_path=")
        }),
        "TSV must emit the explicit ASV-on-corpus-01 blocker row"
    );
    assert!(
        rows.iter().any(|row| {
            row.contains("fastq\tfastq.screen_taxonomy\tkraken2\tbenchmark_ready")
                && row.contains("\tmissing_taxonomy_database_bundle\t")
                && row.contains("database_artifact_id=taxonomy_db")
                && row.contains("taxonomy_database_root=taxonomy_reference")
        }),
        "TSV must emit the explicit taxonomy-without-corpus-02-db blocker row"
    );
    assert!(
        rows.iter().any(|row| {
            row.contains("bam\tbam.kinship\tking\tbenchmark_ready")
                && row.contains("\tcorpus-01-bam-mini\tcorpus-01-bam\tcorpus-01-kinship-mini\tcorpus-01-kinship\tmissing_kinship_pair_manifest\t")
                && row.contains("reference_panel=human_like_relatedness_panel")
                && row.contains("cases=")
        }),
        "TSV must emit the explicit kinship-without-pair-manifest blocker row"
    );
}
