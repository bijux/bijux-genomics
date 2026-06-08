#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_all_domain_expected_benchmark_results_writes_governed_tsv_file() {
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
        .args(["bench", "readiness", "render-all-domain-expected-benchmark-results"])
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
        "benchmarks/readiness/expected-benchmark-results-all-domains.tsv"
    );

    let payload = std::fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read all-domain expected benchmark results");
    let mut lines = payload.lines();
    assert_eq!(
        lines.next(),
        Some(
            "result_id\tdomain\tstage_id\ttool_id\tcorpus_id\tasset_profile_id\texpected_outputs\texpected_metrics\treport_section"
        )
    );

    let rows = lines.collect::<Vec<_>>();
    assert_eq!(rows.len(), 124);
    assert!(rows.iter().any(|row| {
        row == &"fastq:corpus-02-edna-mini:fastq.screen_taxonomy:sample-set:kraken2\tfastq\tfastq.screen_taxonomy\tkraken2\tcorpus-02-edna-mini\tdatabase_artifact_id+taxonomy_database_root\tclassification_report_json,screen_report_tsv,unclassified_reads_r1,unclassified_reads_r2\tclassification_report_json,classified_read_fraction\tcontamination_screening"
    }));
    assert!(rows.iter().any(|row| {
        row == &"bam:corpus-01-kinship-mini:bam.kinship:sample-set:king\tbam\tbam.kinship\tking\tcorpus-01-kinship-mini\treference_fasta+reference_panel\tkinship_report,summary,stage_metrics\tkinship_report,observed_max_overlap_snps,pair_count,pairwise_results,status\tsample_identity"
    }));
    assert!(rows.iter().any(|row| {
        row == &"vcf:vcf_production_regression:vcf.call:bam_bundle:bcftools\tvcf\tvcf.call\tbcftools\tvcf_production_regression\tbam_bundle\tcalled_vcf\tvariant_count,snp_count,indel_count,sample_count\tvariant_calling"
    }));
    assert!(rows.iter().any(|row| {
        row == &"vcf:vcf_production_regression:vcf.postprocess:vcf_single_sample:bcftools\tvcf\tvcf.postprocess\tbcftools\tvcf_production_regression\tvcf_single_sample\tpostprocess_vcf\treadable_vcf,tabix_present,contigs_consistent_with_species_context,left_align_applied,multiallelic_records_split,indels_normalized,variant_ids_normalized,invalid_records_removed,filter_standardized_to_pass\tnormalization"
    }));
}
