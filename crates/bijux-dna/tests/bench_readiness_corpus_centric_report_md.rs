#![cfg(feature = "bam_downstream")]
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
fn bench_readiness_corpus_centric_report_writes_named_corpus_sections() {
    let output = run_cli(&["bench", "readiness", "render-corpus-centric-report"]);
    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let rendered_path = String::from_utf8(output.stdout).expect("stdout utf8");
    assert_eq!(rendered_path.trim(), "benchmarks/readiness/corpus-centric-report.md");

    let repo_root = support::repo_root().expect("repo root");
    let markdown = std::fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read corpus-centric markdown");

    assert!(markdown.contains("# Corpus-Centric Benchmark Report"));
    assert!(markdown.contains("- Corpus count: 8"));
    assert!(markdown.contains("- Assigned stages: 50"));
    assert!(markdown.contains("- Assigned stage-tool rows: 121"));
    assert!(markdown.contains("- Benchmark-ready rows: 118"));
    assert!(markdown.contains("- Blocked rows: 3"));
    assert!(markdown.contains("- Corpora with blocked stages: 1"));

    assert!(markdown.contains("## corpus-02"));
    assert!(markdown.contains("| fastq | fastq.screen_taxonomy | corpus-02-edna-mini | Contamination Screening | 4 | 4 | 0 | not_declared | none |"));

    assert!(markdown.contains("## corpus-03"));
    assert!(markdown.contains("| fastq | fastq.normalize_abundance | corpus-03-amplicon-mini | Amplicon Interpretation | 1 | 1 | 0 | not_applicable | none |"));
    assert!(markdown.contains("| fastq | fastq.cluster_otus | corpus-03-amplicon-mini | Amplicon Interpretation | 1 | 1 | 0 | not_applicable | none |"));
    assert!(markdown.contains("| fastq | fastq.infer_asvs | corpus-03-amplicon-mini | Amplicon Interpretation | 1 | 1 | 0 | not_applicable | none |"));
    assert!(markdown.contains("| fastq | fastq.remove_chimeras | corpus-03-amplicon-mini | Amplicon Interpretation | 1 | 1 | 0 | not_applicable | none |"));

    assert!(markdown.contains("## corpus-01-adna-bam"));
    assert!(markdown.contains("| bam | bam.damage | corpus-01-adna-damage-mini | Ancient Signal | 6 | 6 | 0 | terminal_c_to_t_5p, terminal_g_to_a_3p, damage_signal, runtime_s, memory_mb | none |"));
    assert!(markdown.contains("| bam | bam.contamination | corpus-01-adna-bam-mini | Ancient Signal | 3 | 3 | 0 | scope, prerequisites_passed, estimate, ci_low, ci_high | none |"));

    assert!(markdown.contains("## corpus-01-genotyping"));
    assert!(markdown.contains("| bam | bam.genotyping | corpus-01-genotyping-mini | Downstream Readiness | 1 | 1 | 0 | not_applicable | none |"));

    assert!(markdown.contains("## reference-index-assets"));
    assert!(markdown.contains("| fastq | fastq.index_reference | reference-index-assets | Reference Preparation | 2 | 2 | 0 | index_build_exit_code | none |"));

    assert!(markdown.contains("## corpus-01-kinship"));
    assert!(markdown.contains("| bam | bam.kinship | corpus-01-kinship-mini | Sample Identity | 2 | 2 | 0 | observed_max_overlap_snps, pair_count, status, pairwise_results | none |"));
}
