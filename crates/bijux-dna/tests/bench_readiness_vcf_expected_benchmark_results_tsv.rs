#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_vcf_expected_benchmark_results_writes_governed_tsv_columns() {
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
        .args(["bench", "readiness", "render-vcf-expected-benchmark-results"])
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
    assert_eq!(rendered_path.trim(), "benchmarks/readiness/vcf-expected-benchmark-results.tsv");

    let tsv = std::fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read VCF expected benchmark results TSV");
    let mut lines = tsv.lines();
    assert_eq!(
        lines.next(),
        Some(
            "domain\tstage_id\ttool_id\tcorpus_id\tasset_profile_id\texpected_outputs\texpected_metrics\treport_section"
        )
    );
    let rows = lines.collect::<Vec<_>>();
    assert_eq!(rows.len(), 13);
    assert!(
        rows.iter().any(|row| {
            row == &"vcf\tvcf.call\tbcftools\tvcf_production_regression\tbam_bundle\tcalled_vcf\tvariant_count,snp_count,indel_count,sample_count\tvariant_calling"
        }),
        "TSV must retain the governed VCF call expected-result row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"vcf\tvcf.qc\tbcftools\tvcf_production_regression\tvcf_cohort\tqc_report\tvariant_count,sample_missingness,variant_missingness,maf_summary,heterozygosity,hwe_summary,excluded_samples,excluded_variants,sample_missingness_exclusion_threshold,variant_missingness_exclusion_threshold\tquality_control"
        }),
        "TSV must retain the governed bcftools QC expected-result row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"vcf\tvcf.gl_propagation\tbcftools\tvcf_production_regression\tvcf_single_sample\tgl_propagated_vcf\tinput_likelihood_fields,output_likelihood_fields,lost_fields,site_count_before,site_count_after,sample_count\tlikelihood_postprocess"
        }),
        "TSV must retain the governed VCF GL propagation expected-result row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"vcf\tvcf.postprocess\tbcftools\tvcf_production_regression\tvcf_single_sample\tpostprocess_vcf\treadable_vcf,tabix_present,contigs_consistent_with_species_context,left_align_applied,multiallelic_records_split,indels_normalized,variant_ids_normalized,invalid_records_removed,filter_standardized_to_pass\tnormalization"
        }),
        "TSV must retain the governed VCF postprocess expected-result row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"vcf\tvcf.prepare_reference_panel\tbcftools\tvcf_production_regression\tvcf_reference_panel\tprepared_panel,chunks_json\tinput_variants,output_variants,sample_count,sample_ids,sample_consistent,duplicate_sites_removed,normalization_status,parseable\treference_panel_preparation"
        }),
        "TSV must retain the governed VCF prepare-reference-panel expected-result row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"vcf\tvcf.stats\tbcftools\tvcf_production_regression\tvcf_cohort\tstats_json\tvariant_count,snp_count,indel_count,transition_count,transversion_count,ti_tv,sample_count\tquality_control"
        }),
        "TSV must retain the governed VCF stats expected-result row"
    );
}
