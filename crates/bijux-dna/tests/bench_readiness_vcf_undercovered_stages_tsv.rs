#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_vcf_undercovered_stages_writes_governed_tsv_columns() {
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
        .args(["bench", "readiness", "render-vcf-undercovered-stages"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let tsv_path = repo_root.join("target/bench-readiness/vcf-undercovered-stages.tsv");
    assert!(tsv_path.is_file(), "VCF undercovered stages TSV must exist");

    let tsv = std::fs::read_to_string(&tsv_path).expect("read VCF undercovered stages");
    let mut lines = tsv.lines();
    assert_eq!(
        lines.next(),
        Some("stage_id\tvalid_tool_classes\tregistered_tools\tmissing_tools\tdecision")
    );
    let rows = lines.collect::<Vec<_>>();
    assert_eq!(rows.len(), 12, "TSV must retain the governed VCF undercovered row count");
    for row in [
        "vcf.admixture\tcohort_analysis,variant_processing\tplink2\tbcftools,plink\tlimit_to_specialized_tool",
        "vcf.call_gl\tgenotype_likelihood_calling,variant_processing\tbcftools\tangsd\tfuture_not_benchmark_ready",
        "vcf.call_pseudohaploid\tgenotype_likelihood_calling,variant_processing\tbcftools\tangsd\tfuture_not_benchmark_ready",
        "vcf.damage_filter\tgenotype_likelihood_calling,variant_processing\tbcftools\tangsd\tfuture_not_benchmark_ready",
        "vcf.gl_propagation\tgenotype_likelihood_calling,variant_processing\tbcftools\tangsd\tfuture_not_benchmark_ready",
        "vcf.ibd\tdemography,relatedness\tgermline\tibdhap,ibdne,ibdseq\tfuture_not_benchmark_ready",
        "vcf.imputation\timputation,phasing,variant_processing\tbeagle\tbcftools,beagle-imputation,glimpse,impute5,minimac4\tlimit_to_specialized_tool",
        "vcf.impute\timputation,phasing\tbeagle\tbeagle-imputation,glimpse,impute5,minimac4\tfuture_not_benchmark_ready",
        "vcf.pca\tcohort_analysis,population_structure,variant_processing\tplink2\tbcftools,eigensoft\tlimit_to_specialized_tool",
        "vcf.phasing\tphasing,variant_processing\tshapeit5\tbcftools,beagle,eagle,shapeit\tlimit_to_specialized_tool",
        "vcf.population_structure\tcohort_analysis,population_structure\tplink2\teigensoft,plink\tfuture_not_benchmark_ready",
        "vcf.qc\tcohort_analysis,variant_processing\tplink2\tbcftools,plink\tlimit_to_specialized_tool",
    ] {
        assert!(
            rows.iter().any(|candidate| candidate == &row),
            "TSV must retain governed VCF undercovered row: {row}"
        );
    }
}
