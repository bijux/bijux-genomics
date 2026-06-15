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
fn bench_readiness_stage_centric_report_writes_named_stage_sections() {
    let output = run_cli(&["bench", "readiness", "render-stage-centric-report"]);
    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let rendered_path = String::from_utf8(output.stdout).expect("stdout utf8");
    assert_eq!(rendered_path.trim(), "benchmarks/readiness/stage-centric-report.md");

    let repo_root = support::repo_root().expect("repo root");
    let markdown = std::fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read stage-centric markdown");

    assert!(markdown.contains("# Stage-Centric Benchmark Report"));
    assert!(markdown.contains("- Stage count: 51"));
    assert!(markdown.contains("- Multi-tool stages: 29"));
    assert!(markdown.contains("| fastq | fastq.trim_reads | Read Cleanup | 14 | 13 | 1 | not_declared | seqpurge (support) |"));
    assert!(markdown.contains("| fastq | fastq.index_reference | Reference Preparation | 2 | 2 | 0 | index_build_exit_code | none |"));
    assert!(markdown.contains("| fastq | fastq.normalize_abundance | Amplicon Interpretation | 1 | 1 | 0 | not_applicable | none |"));

    assert!(markdown.contains("## fastq.profile_overrepresented_sequences"));
    assert!(markdown.contains("- Shared metric contract: declared"));
    assert!(markdown.contains("- Shared metrics: sequence_count, flagged_sequences, top_fraction"));
    assert!(markdown.contains("| fastq_scan | benchmark_ready | none | observer_specialized_benchmark | runnable | comparable | fixture:corpus-01-mini | not_required |"));

    assert!(markdown.contains("## bam.damage"));
    assert!(markdown.contains("- Shared metrics: terminal_c_to_t_5p, terminal_g_to_a_3p, damage_signal, runtime_s, memory_mb"));
    assert!(markdown.contains("| damageprofiler | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-adna-damage-mini | not_required |"));

    assert!(markdown.contains("## bam.contamination"));
    assert!(markdown.contains("| verifybamid2 | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-adna-bam-mini | assigned |"));
}
