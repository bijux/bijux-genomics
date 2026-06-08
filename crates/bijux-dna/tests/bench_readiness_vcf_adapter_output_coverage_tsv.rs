#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_vcf_adapter_output_coverage_writes_governed_tsv_columns() {
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
        .args(["bench", "readiness", "render-vcf-adapter-output-coverage"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let tsv_path = repo_root.join("benchmarks/readiness/vcf-adapter-output-coverage.tsv");
    assert!(tsv_path.is_file(), "VCF adapter output coverage TSV must exist");

    let tsv = std::fs::read_to_string(&tsv_path).expect("read VCF adapter output coverage");
    let mut lines = tsv.lines();
    assert_eq!(
        lines.next(),
        Some(
            "stage_id\ttool_id\traw_outputs\tnormalized_metrics\tstdout\tstderr\tmanifest\tindex_outputs\tstatus\tbenchmark_status\tmissing_declarations\treason"
        )
    );

    let rows = lines.collect::<Vec<_>>();
    assert_eq!(rows.len(), 39, "TSV must retain the governed 39-row VCF adapter slice");

    let has_row = |stage_id: &str,
                   tool_id: &str,
                   status: &str,
                   benchmark_status: &str,
                   normalized_contains: &str,
                   index_contains: &str| {
        rows.iter().any(|row| {
            let columns = row.split('\t').collect::<Vec<_>>();
            columns.len() == 12
                && columns[0] == stage_id
                && columns[1] == tool_id
                && columns[3].contains(normalized_contains)
                && columns[7].contains(index_contains)
                && columns[8] == status
                && columns[9] == benchmark_status
        })
    };

    assert!(
        has_row("vcf.call", "bcftools", "complete", "benchmark_ready", "called_vcf=", ".tbi"),
        "TSV must retain the governed bcftools call output row"
    );
    assert!(
        has_row("vcf.stats", "bcftools", "complete", "benchmark_ready", "stats_json=", ""),
        "TSV must retain the governed bcftools stats output row"
    );
    assert!(
        has_row("vcf.qc", "bcftools", "complete", "benchmark_ready", "qc_report=", ""),
        "TSV must retain the governed bcftools QC output row"
    );
    assert!(
        has_row(
            "vcf.phasing",
            "shapeit5",
            "complete",
            "not_benchmark_ready",
            "phased_vcf=",
            ".tbi"
        ),
        "TSV must retain the governed shapeit5 phasing output row"
    );
    assert!(
        has_row(
            "vcf.demography",
            "ibdne",
            "complete",
            "not_benchmark_ready",
            "demography_report=",
            ""
        ),
        "TSV must retain the governed ibdne demography output row"
    );
    assert_eq!(
        rows.iter()
            .filter(|row| {
                let columns = row.split('\t').collect::<Vec<_>>();
                columns.len() == 12 && columns[0] == "vcf.roh" && columns[1] == "plink2"
            })
            .count(),
        1,
        "TSV must keep one canonical plink2 ROH output row"
    );
}
