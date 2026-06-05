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
fn bench_readiness_tool_centric_report_writes_named_tool_sections() {
    let output = run_cli(&["bench", "readiness", "render-tool-centric-report"]);
    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let rendered_path = String::from_utf8(output.stdout).expect("stdout utf8");
    assert_eq!(rendered_path.trim(), "target/bench-readiness/tool-centric-report.md");

    let repo_root = support::repo_root().expect("repo root");
    let markdown = std::fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read tool-centric markdown");

    assert!(markdown.contains("# Tool-Centric Benchmark Report"));
    assert!(markdown.contains("- Tool count: 67"));
    assert!(markdown.contains("- Stage-tool rows: 123"));
    assert!(markdown.contains("| samtools | bam | 10 | 10 | 0 | none |"));
    assert!(
        markdown.contains("| fastp | fastq | 5 | 4 | 1 | fastq.filter_low_complexity (support) |")
    );

    assert!(markdown.contains("## samtools"));
    assert!(markdown.contains("| bam | bam.coverage | Coverage and Quality | Coverage, Bias, and QC | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-bam-mini | not_required |"));
    assert!(markdown.contains("| bam | bam.validate | Alignment Intake | Alignment Baseline | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-bam-mini | not_required |"));

    assert!(markdown.contains("## bowtie2"));
    assert!(markdown.contains("| bam | bam.align | Alignment Intake | Alignment Baseline | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-mini | not_required |"));
    assert!(markdown.contains("| fastq | fastq.deplete_host | Contamination Screening | Screening and Contamination | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | assigned |"));
    assert!(markdown.contains("| fastq | fastq.deplete_reference_contaminants | Contamination Screening | Screening and Contamination | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | assigned |"));

    assert!(markdown.contains("## fastp"));
    assert!(markdown.contains("| fastq | fastq.filter_low_complexity | Read Cleanup | Cleanup and Retention | not_benchmark_ready | support | planned_contract | declared_only | not_normalized | fixture:corpus-01-mini | not_required |"));

    assert!(markdown.contains("## kraken2"));
    assert!(markdown.contains("| fastq | fastq.screen_taxonomy | Contamination Screening | Screening and Contamination | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-02-edna-mini | assigned |"));

    assert!(markdown.contains("## gatk"));
    assert!(markdown.contains("| bam | bam.recalibration | Downstream Readiness | Variant and Bias Readiness | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-bam-mini | assigned |"));
}
