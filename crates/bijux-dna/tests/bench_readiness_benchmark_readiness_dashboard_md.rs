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
fn bench_readiness_benchmark_readiness_dashboard_writes_markdown_and_json_outputs() {
    let output = run_cli(&["bench", "readiness", "render-benchmark-readiness-dashboard"]);
    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let rendered_path = String::from_utf8(output.stdout).expect("stdout utf8");
    assert_eq!(rendered_path.trim(), "benchmarks/readiness/FASTQ_BAM_BENCHMARK_READINESS.md");

    let repo_root = support::repo_root().expect("repo root");
    let markdown = std::fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read dashboard markdown");
    let json_path = repo_root.join("benchmarks/readiness/FASTQ_BAM_BENCHMARK_READINESS.json");
    let json_payload = std::fs::read_to_string(&json_path).expect("read dashboard json");
    let json_value: serde_json::Value =
        serde_json::from_str(&json_payload).expect("parse dashboard json");

    assert!(markdown.contains("# FASTQ + BAM Benchmark Readiness Dashboard"));
    assert!(markdown.contains("- Expected pairs: 123"));
    assert!(markdown.contains("- Ready pairs: 112"));
    assert!(markdown.contains("- Blocked pairs: 11"));
    assert!(markdown.contains("| Matrix | attention_required | all governed fastq and bam stage-tool pairs | 123 | 112 | 11 |"));
    assert!(markdown.contains("| Reports | complete | governed local report surfaces | 5 | 5 | 0 | expected_results=112, stage_sections=51, tool_sections=67, corpus_sections=7 |"));
    assert!(markdown.contains(
        "| pair_readiness | benchmarks/readiness/pair-readiness.tsv | 123 stage_tool_pairs |"
    ));
    assert!(markdown.contains("| stage_centric_report | benchmarks/readiness/stage-centric-report.md | 51 stage_sections |"));
    assert!(markdown.contains("| fastq | fastq.index_reference | bowtie2_build | corpus | observer_specialized_benchmark | runnable | comparable | planner_only | assigned |"));
    assert!(markdown.contains("| fastq | fastq.report_qc | multiqc | corpus | observer_specialized_benchmark | runnable | comparable | planner_only | not_required |"));
    assert!(markdown.contains("| fastq | fastq.trim_reads | seqpurge | support | planned_contract | declared_only | not_normalized | fixture:corpus-01-mini | not_required |"));

    assert_eq!(
        json_value.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.benchmark_readiness_dashboard.v1")
    );
    assert_eq!(
        json_value.get("markdown_output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/FASTQ_BAM_BENCHMARK_READINESS.md")
    );
    assert_eq!(
        json_value.get("json_output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/FASTQ_BAM_BENCHMARK_READINESS.json")
    );
}
