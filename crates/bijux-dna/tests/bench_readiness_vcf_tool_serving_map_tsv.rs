#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_vcf_tool_serving_map_writes_governed_tsv_columns() {
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
        .args(["bench", "readiness", "render-vcf-tool-serving-map"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let tsv_path = repo_root.join("benchmarks/readiness/vcf-tool-serving-map.tsv");
    assert!(tsv_path.is_file(), "VCF tool serving map TSV must exist");

    let tsv = std::fs::read_to_string(&tsv_path).expect("read VCF tool serving map");
    let mut lines = tsv.lines();
    assert_eq!(
        lines.next(),
        Some(
            "tool_id\tstage_id\tsupport_status\tadapter_status\tparser_status\tcorpus_status\tasset_status\tbenchmark_status"
        )
    );
    let rows = lines.collect::<Vec<_>>();
    assert_eq!(rows.len(), 23, "TSV must retain the governed VCF row count");
    for row in [
        "bcftools\tvcf.qc\tsupported\trunnable\tparse_normalized\tfixture:vcf_production_regression\tassigned\tbenchmark_ready",
        "bcftools\tvcf.call\tsupported\trunnable\tparse_normalized\tfixture:vcf_production_regression\tassigned\tbenchmark_ready",
        "bcftools\tvcf.call_gl\tsupported\trunnable\tparse_normalized\tfixture:vcf_production_regression\tassigned\tbenchmark_ready",
        "plink2\tvcf.admixture\tsupported\trunnable\tparse_normalized\tfixture:vcf_production_regression\tassigned\tbenchmark_ready",
        "plink2\tvcf.pca\tsupported\trunnable\tparse_normalized\tfixture:vcf_production_regression\tassigned\tbenchmark_ready",
        "eigensoft\tvcf.pca\tsupported\trunnable\tparse_normalized\tfixture:vcf_production_regression\tassigned\tbenchmark_ready",
        "plink\tvcf.qc\tsupported\trunnable\tparse_normalized\tfixture:vcf_production_regression\tassigned\tbenchmark_ready",
        "plink2\tvcf.qc\tsupported\trunnable\tparse_normalized\tfixture:vcf_production_regression\tassigned\tbenchmark_ready",
        "bcftools\tvcf.prepare_reference_panel\tsupported\trunnable\tparse_normalized\tfixture:vcf_production_regression\tassigned\tbenchmark_ready",
        "shapeit5\tvcf.phasing\tsupported\trunnable\tparse_normalized\tfixture:vcf_production_regression\tassigned\tbenchmark_ready",
        "beagle\tvcf.imputation_metrics\tsupported\trunnable\tparse_normalized\tfixture:vcf_production_regression\tassigned\tbenchmark_ready",
        "beagle\tvcf.impute\tsupported\trunnable\tparse_normalized\tfixture:vcf_production_regression\tassigned\tbenchmark_ready",
        "plink2\tvcf.population_structure\tsupported\trunnable\tparse_normalized\tfixture:vcf_production_regression\tassigned\tbenchmark_ready",
        "ibdne\tvcf.demography\tplanned\tdeclared_only\tparse_normalized\tfixture:vcf_production_regression\tnot_required\tnot_benchmark_ready",
    ] {
        assert!(
            rows.iter().any(|candidate| candidate == &row),
            "TSV must retain governed VCF readiness row: {row}"
        );
    }
}
