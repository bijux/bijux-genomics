#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_all_domain_parser_fixture_coverage_writes_governed_tsv_file() {
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
        .args(["bench", "readiness", "render-all-domain-parser-fixture-coverage"])
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
    assert_eq!(
        rendered_path.trim(),
        "benchmarks/readiness/all-domains/parser-fixture-coverage.tsv"
    );

    let payload = std::fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read all-domain parser fixture coverage");
    let mut lines = payload.lines();
    assert_eq!(
        lines.next(),
        Some(
            "domain\tstage_id\ttool_id\tcorpus_id\tasset_profile_id\tadapter_id\tparser_id\tschema_id\tparser_fixture_parser_id\tparser_fixture_schema_id\tparser_fixture_reference_kind\tparser_fixture_reference\tproof_source\tcoverage_status\treason"
        )
    );

    let rows = lines.collect::<Vec<_>>();
    assert!(rows.len() >= 128);
    assert!(rows.iter().any(|row| {
        row == &"fastq\tfastq.trim_reads\ttrimmomatic\tcorpus-01-mini\tcorpus_only\tfastq.adapter.trim_reads\tfastq.parser.trim_reads\tfastq_trim_reads_v2\tnone\tnone\tfixture_corpus\tfixture:corpus-01-mini\tfastq_parser_coverage\tcovered\trow `fastq.trim_reads` / `trimmomatic` has governed support, adapter-backed command rendering, fixture-backed corpus coverage, and normalized parser output"
    }));
    assert!(rows.iter().any(|row| {
        row == &"bam\tbam.contamination\tschmutzi\tcorpus-01-adna-bam-mini\treference_fasta+reference_panel\tbam.adapter.contamination\tbam.parser.contamination\tbam_contamination_normalized_v1\tnone\tnone\tfixture_corpus\tfixture:corpus-01-adna-bam-mini\tbam_parser_coverage\tcovered\trow `bam.contamination` / `schmutzi` is benchmark_ready with governed support, adapter-backed command rendering, fixture-backed corpus coverage, and parser-fixture-validated output"
    }));
    assert!(rows.iter().any(|row| {
        row.starts_with(
            "vcf\tvcf.postprocess\tbcftools\tvcf_production_regression\tvcf_single_sample\tvcf.adapter.transform\tvcf.parser.vcf_output\tbijux.schemas.bench.vcf-normalized-metrics.postprocess.v1\tparse_bcftools_postprocess_metrics\tbijux.vcf.postprocess.v1\tfixture_inventory_path\tbenchmarks/tests/fixtures/bench/parsers/vcf/bcftools/vcf.postprocess\tvcf_parser_coverage\tcovered\trow `vcf.postprocess` / `bcftools` is benchmark_ready with parser fixture inventory `benchmarks/tests/fixtures/bench/parsers/vcf/bcftools/vcf.postprocess` and schema `bijux.vcf.postprocess.v1`"
        )
    }));
    assert!(rows.iter().any(|row| {
        row.starts_with(
            "vcf\tvcf.imputation_metrics\tbeagle\tvcf_production_regression\tvcf_cohort_with_panel\tvcf.adapter.panel_workflow\tvcf.parser.report_json\tbijux.schemas.bench.vcf-normalized-metrics.imputation-metrics.v1\tparse_vcf_report_json\tbijux.vcf.report_json.v1\tfixture_inventory_path\tbenchmarks/tests/fixtures/bench/parsers/vcf/beagle/vcf.imputation_metrics\tvcf_parser_coverage\tcovered\t"
        )
    }));
    assert!(
        rows.iter().all(|row| row.contains("\tcovered\t")),
        "all active rows must retain covered parser-fixture proof"
    );
}
