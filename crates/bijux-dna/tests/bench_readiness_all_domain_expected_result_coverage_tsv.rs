#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_all_domain_expected_result_coverage_writes_governed_tsv_file() {
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
        .args(["bench", "readiness", "render-all-domain-expected-result-coverage"])
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
        "benchmarks/readiness/all-domains/expected-result-coverage.tsv"
    );

    let payload = std::fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read all-domain expected-result coverage");
    let mut lines = payload.lines();
    assert_eq!(
        lines.next(),
        Some(
            "result_id\tdomain\tstage_id\ttool_id\tcorpus_id\tasset_profile_id\tadapter_id\tparser_id\tschema_id\texpected_outputs\texpected_metrics\treport_section\tcoverage_status\treason"
        )
    );

    let rows = lines.collect::<Vec<_>>();
    assert!(rows.len() >= 128);
    assert!(rows.iter().any(|row| {
        row == &"fastq:corpus-02-edna-mini:fastq.screen_taxonomy:sample-set:kraken2\tfastq\tfastq.screen_taxonomy\tkraken2\tcorpus-02-edna-mini\tdatabase_artifact_id+taxonomy_database_root\tfastq.adapter.screen_taxonomy\tfastq.parser.screen_taxonomy\tfastq_screen_taxonomy_v1\tclassification_report_json,screen_report_tsv,unclassified_reads_r1,unclassified_reads_r2\tclassification_report_json,classified_fraction,classified_reads,taxonomy_database_id,top_taxa,unclassified_fraction,unclassified_reads\tcontamination_screening\tcovered\tactive row `fastq` / `fastq.screen_taxonomy` / `kraken2` keeps governed expected-result coverage with result_id `fastq:corpus-02-edna-mini:fastq.screen_taxonomy:sample-set:kraken2` in report section `contamination_screening`"
    }));
    assert!(rows.iter().any(|row| {
        row == &"bam:corpus-01-kinship-mini:bam.kinship:sample-set:king\tbam\tbam.kinship\tking\tcorpus-01-kinship-mini\treference_fasta+reference_panel\tbam.adapter.kinship\tbam.parser.kinship\tbam_kinship_normalized_v1\tkinship_report,summary,stage_metrics\tkinship_report,observed_max_overlap_snps,pair_count,pairwise_results,status\tsample_identity\tcovered\tactive row `bam` / `bam.kinship` / `king` keeps governed expected-result coverage with result_id `bam:corpus-01-kinship-mini:bam.kinship:sample-set:king` in report section `sample_identity`"
    }));
    assert!(rows.iter().any(|row| {
        row == &"vcf:vcf_production_regression:vcf.postprocess:vcf_single_sample:bcftools\tvcf\tvcf.postprocess\tbcftools\tvcf_production_regression\tvcf_single_sample\tvcf.adapter.transform\tvcf.parser.vcf_output\tbijux.schemas.bench.vcf-normalized-metrics.postprocess.v1\tpostprocess_vcf\treadable_vcf,tabix_present,contigs_consistent_with_species_context,left_align_applied,multiallelic_records_split,indels_normalized,variant_ids_normalized,invalid_records_removed,filter_standardized_to_pass\tnormalization\tcovered\tactive row `vcf` / `vcf.postprocess` / `bcftools` keeps governed expected-result coverage with result_id `vcf:vcf_production_regression:vcf.postprocess:vcf_single_sample:bcftools` in report section `normalization`"
    }));
    assert!(rows.iter().any(|row| {
        row == &"vcf:vcf_production_regression:vcf.imputation_metrics:vcf_cohort_with_panel:beagle\tvcf\tvcf.imputation_metrics\tbeagle\tvcf_production_regression\tvcf_cohort_with_panel\tvcf.adapter.panel_workflow\tvcf.parser.report_json\tbijux.schemas.bench.vcf-normalized-metrics.imputation-metrics.v1\timputation_metrics_json\tstatus,concordance,mean_info_score,r2_available,dosage_r2,low_confidence_sites,masked_truth_sites,missing_quality_fields\timputation\tcovered\tactive row `vcf` / `vcf.imputation_metrics` / `beagle` keeps governed expected-result coverage with result_id `vcf:vcf_production_regression:vcf.imputation_metrics:vcf_cohort_with_panel:beagle` in report section `imputation`"
    }));
    assert!(
        rows.iter().all(|row| row.contains("\tcovered\t")),
        "every active binding must retain complete governed expected-result coverage"
    );
}
