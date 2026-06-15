#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_vcf_parser_fixture_coverage_writes_governed_tsv_columns() {
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
        .args(["bench", "readiness", "render-vcf-parser-fixture-coverage"])
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
    assert_eq!(rendered_path.trim(), "benchmarks/readiness/vcf/vcf-parser-fixture-coverage.tsv");

    let tsv = std::fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read VCF parser fixture coverage TSV");
    let mut lines = tsv.lines();
    assert_eq!(
        lines.next(),
        Some("stage_id\ttool_id\tcorpus_id\tasset_profile_id\tadapter_id\tparser_id\tschema_id\tparser_fixture_parser_id\tparser_fixture_schema_id\tparser_fixture_root_path\texpected_normalized_path\traw_fixture_count\traw_fixture_paths\tcoverage_status\treason")
    );
    let rows = lines.collect::<Vec<_>>();
    assert_eq!(rows.len(), 20);
    assert!(
        rows.iter().any(|row| {
            row.starts_with(
                "vcf.admixture\tplink2\tvcf_production_regression\tvcf_cohort\tvcf.adapter.population_structure\tvcf.parser.report_json\tbijux.schemas.bench.vcf-normalized-metrics.admixture.v1\tparse_plink2_admixture_metrics\tbijux.vcf.admixture.v1\tbenchmarks/tests/fixtures/bench/parsers/vcf/plink2/vcf.admixture\tbenchmarks/tests/fixtures/bench/parsers/vcf/plink2/vcf.admixture/expected.normalized.json\t3\t"
            )
        }),
        "TSV must retain the governed admixture parser fixture coverage row"
    );
    assert!(
        rows.iter().any(|row| {
            row.starts_with(
                "vcf.qc\tbcftools\tvcf_production_regression\tvcf_cohort\tvcf.adapter.quality_control\tvcf.parser.qc_report\tbijux.schemas.bench.vcf-normalized-metrics.qc.v1\tparse_bcftools_qc_metrics\tbijux.vcf.qc.v1\tbenchmarks/tests/fixtures/bench/parsers/vcf/bcftools/vcf.qc\tbenchmarks/tests/fixtures/bench/parsers/vcf/bcftools/vcf.qc/expected.normalized.json\t6\t"
            )
        }),
        "TSV must retain the governed VCF QC parser fixture coverage row"
    );
    assert!(
        rows.iter().any(|row| {
            row.starts_with(
                "vcf.call\tbcftools\tvcf_production_regression\tbam_bundle\tvcf.adapter.calling\tvcf.parser.call_summary\tbijux.schemas.bench.vcf-normalized-metrics.call.v1\tparse_bcftools_call_metrics\tbijux.vcf.call.v1\tbenchmarks/tests/fixtures/bench/parsers/vcf/bcftools/vcf.call\tbenchmarks/tests/fixtures/bench/parsers/vcf/bcftools/vcf.call/expected.normalized.json\t1\t"
            )
        }),
        "TSV must retain the governed VCF call parser fixture coverage row"
    );
    assert!(
        rows.iter().any(|row| {
            row.starts_with(
                "vcf.call_gl\tbcftools\tvcf_production_regression\tbam_bundle\tvcf.adapter.calling\tvcf.parser.call_summary\tbijux.schemas.bench.vcf-normalized-metrics.call-gl.v1\tparse_bcftools_call_gl_metrics\tbijux.vcf.call_gl.v1\tbenchmarks/tests/fixtures/bench/parsers/vcf/bcftools/vcf.call_gl\tbenchmarks/tests/fixtures/bench/parsers/vcf/bcftools/vcf.call_gl/expected.normalized.json\t1\t"
            )
        }),
        "TSV must retain the governed VCF GL parser fixture coverage row"
    );
    assert!(
        rows.iter().any(|row| {
            row.starts_with(
                "vcf.pca\tplink2\tvcf_production_regression\tvcf_cohort\tvcf.adapter.population_structure\tvcf.parser.report_json\tbijux.schemas.bench.vcf-normalized-metrics.pca.v1\tparse_plink2_pca_metrics\tbijux.vcf.pca.v1\tbenchmarks/tests/fixtures/bench/parsers/vcf/plink2/vcf.pca\tbenchmarks/tests/fixtures/bench/parsers/vcf/plink2/vcf.pca/expected.normalized.json\t4\t"
            )
        }),
        "TSV must retain the governed plink2 PCA parser fixture coverage row"
    );
    assert!(
        rows.iter().any(|row| {
            row.starts_with(
                "vcf.population_structure\tplink2\tvcf_production_regression\tvcf_cohort\tvcf.adapter.population_structure\tvcf.parser.report_json\tbijux.schemas.bench.vcf-normalized-metrics.population-structure.v1\tparse_plink2_population_structure_metrics\tbijux.vcf.population_structure.v1\tbenchmarks/tests/fixtures/bench/parsers/vcf/plink2/vcf.population_structure\tbenchmarks/tests/fixtures/bench/parsers/vcf/plink2/vcf.population_structure/expected.normalized.json\t9\t"
            )
        }),
        "TSV must retain the governed population-structure parser fixture coverage row"
    );
    assert!(
        rows.iter().any(|row| {
            row.starts_with(
                "vcf.pca\teigensoft\tvcf_production_regression\tvcf_cohort\tvcf.adapter.population_structure\tvcf.parser.report_json\tbijux.schemas.bench.vcf-normalized-metrics.pca.v1\tparse_eigensoft_pca_metrics\tbijux.vcf.pca.v1\tbenchmarks/tests/fixtures/bench/parsers/vcf/eigensoft/pca\tbenchmarks/tests/fixtures/bench/parsers/vcf/eigensoft/pca/expected.normalized.json\t8\t"
            )
        }),
        "TSV must retain the governed eigensoft PCA parser fixture coverage row"
    );
    assert!(
        rows.iter().any(|row| {
            row.starts_with(
                "vcf.stats\tbcftools\tvcf_production_regression\tvcf_cohort\tvcf.adapter.quality_control\tvcf.parser.stats_report\tbijux.schemas.bench.vcf-normalized-metrics.stats.v1\tparse_bcftools_stats_metrics\tbijux.vcf.stats.v1\tbenchmarks/tests/fixtures/bench/parsers/vcf/bcftools/vcf.stats\tbenchmarks/tests/fixtures/bench/parsers/vcf/bcftools/vcf.stats/expected.normalized.json\t1\t"
            )
        }),
        "TSV must retain the governed VCF stats parser fixture coverage row"
    );
    assert!(
        rows.iter().any(|row| {
            row.starts_with(
                "vcf.imputation_metrics\tbeagle\tvcf_production_regression\tvcf_cohort_with_panel\tvcf.adapter.panel_workflow\tvcf.parser.report_json\tbijux.schemas.bench.vcf-normalized-metrics.imputation-metrics.v1\tparse_beagle_imputation_metrics\tbijux.vcf.imputation_metrics.v1\tbenchmarks/tests/fixtures/bench/parsers/vcf/imputation/beagle/vcf.imputation_metrics\tbenchmarks/tests/fixtures/bench/parsers/vcf/imputation/beagle/vcf.imputation_metrics/expected.normalized.json\t7\t"
            )
        }),
        "TSV must retain the governed VCF imputation-metrics parser fixture coverage row"
    );
}
