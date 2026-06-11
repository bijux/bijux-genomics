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
fn bench_readiness_full_benchmark_report_writes_markdown_and_json_outputs() {
    let output = run_cli(&["bench", "readiness", "render-full-benchmark-report"]);
    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let rendered_path = String::from_utf8(output.stdout).expect("stdout utf8");
    assert_eq!(rendered_path.trim(), "benchmarks/readiness/FASTQ_BAM_VCF_BENCHMARK_REPORT.md");

    let repo_root = support::repo_root().expect("repo root");
    let markdown = std::fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read report markdown");
    let json_path = repo_root.join("benchmarks/readiness/FASTQ_BAM_VCF_BENCHMARK_REPORT.json");
    let json_payload = std::fs::read_to_string(&json_path).expect("read report json");
    let json_value: serde_json::Value =
        serde_json::from_str(&json_payload).expect("parse report json");

    assert!(markdown.contains("# FASTQ + BAM + VCF Benchmark Report"));
    assert!(markdown.contains("- Report rows: 128"));
    assert!(markdown.contains("- Expected-result rows: 127"));
    assert!(markdown.contains("- Explicit unsupported rows: 1"));
    assert!(markdown.contains("## Stage-Centric"));
    assert!(markdown.contains("## Tool-Centric"));
    assert!(markdown.contains("## Corpus-Centric"));
    assert!(markdown.contains("## Pipeline-Centric"));
    assert!(markdown.contains("## Runtime"));
    assert!(markdown.contains("## Memory"));
    assert!(markdown.contains("## Failures"));
    assert!(markdown.contains("## Missing Results"));
    assert!(markdown.contains("## Comparable Metrics"));
    assert!(markdown.contains("## Unsupported Pairs"));
    assert!(markdown.contains("fastq:corpus-02-edna-mini:fastq.screen_taxonomy:sample-set:kraken2"));
    assert!(markdown.contains("vcf.filter"));
    assert!(markdown.contains("samtools"));
    assert!(markdown.contains("missing_result"));
    assert!(markdown.contains("Unsupported Pairs"));

    assert_eq!(
        json_value.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.full_benchmark_report.v1")
    );
    assert_eq!(json_value.get("row_count").and_then(serde_json::Value::as_u64), Some(128));
}
