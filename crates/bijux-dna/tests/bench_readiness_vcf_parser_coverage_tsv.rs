#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_vcf_parser_coverage_writes_governed_tsv_columns() {
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
        .args(["bench", "readiness", "render-vcf-parser-coverage"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let rendered_path = String::from_utf8(output.stdout).expect("stdout utf8");
    assert_eq!(rendered_path.trim(), "target/bench-readiness/vcf-parser-coverage.tsv");

    let tsv = std::fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read VCF parser coverage TSV");
    let mut lines = tsv.lines();
    assert_eq!(
        lines.next(),
        Some("stage_id\ttool_id\tparser_id\tfixture_path\tschema_id\tcoverage_status")
    );
    let rows = lines.collect::<Vec<_>>();
    assert_eq!(rows.len(), 8);
    assert!(
        rows.iter().any(|row| {
            row == &"vcf.call\tbcftools\tparse_bcftools_call_metrics\tbenchmarks/tests/fixtures/bench/parsers/vcf/bcftools/vcf.call\tbijux.vcf.call.v1\tcovered"
        }),
        "TSV must retain the governed VCF call parser coverage row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"vcf.call_gl\tbcftools\tparse_bcftools_call_gl_metrics\tbenchmarks/tests/fixtures/bench/parsers/vcf/bcftools/vcf.call_gl\tbijux.vcf.call_gl.v1\tcovered"
        }),
        "TSV must retain the governed VCF GL parser coverage row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"vcf.stats\tbcftools\tparse_bcftools_stats_metrics\tbenchmarks/tests/fixtures/bench/parsers/vcf/bcftools/vcf.stats\tbijux.vcf.stats.v1\tcovered"
        }),
        "TSV must retain the governed VCF stats parser coverage row"
    );
}
