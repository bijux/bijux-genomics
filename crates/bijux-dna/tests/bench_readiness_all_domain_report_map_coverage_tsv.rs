#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_all_domain_report_map_coverage_writes_governed_tsv_file() {
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
        .args(["bench", "readiness", "render-all-domain-report-map-coverage"])
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
    assert_eq!(rendered_path.trim(), "benchmarks/readiness/all-domains/report-map-coverage.tsv");

    let payload =
        std::fs::read_to_string(repo_root.join(rendered_path.trim())).expect("read report map TSV");
    let mut lines = payload.lines();
    assert_eq!(
        lines.next(),
        Some(
            "result_id\tdomain\tstage_id\ttool_id\tcorpus_id\tasset_profile_id\tadapter_id\tparser_id\tschema_id\texpected_report_section\treport_section_id\tsummary_table_id\tproof_source\tcoverage_status\treason"
        )
    );

    let rows = lines.collect::<Vec<_>>();
    assert_eq!(rows.len(), 125);
    assert!(rows.iter().any(|row| {
        row == &"fastq:corpus-02-edna-mini:fastq.screen_taxonomy:sample-set:kraken2\tfastq\tfastq.screen_taxonomy\tkraken2\tcorpus-02-edna-mini\tdatabase_artifact_id+taxonomy_database_root\tfastq.adapter.screen_taxonomy\tfastq.parser.screen_taxonomy\tfastq_screen_taxonomy_v1\tcontamination_screening\tcontamination_screening\tscreening_contamination\tfastq_report_map\tcovered\tactive row `fastq` / `fastq.screen_taxonomy` / `kraken2` appears in governed report section `contamination_screening` with summary table `screening_contamination` through `fastq_report_map`"
    }));
    assert!(rows.iter().any(|row| {
        row == &"bam:corpus-01-kinship-mini:bam.kinship:sample-set:king\tbam\tbam.kinship\tking\tcorpus-01-kinship-mini\treference_fasta+reference_panel\tbam.adapter.kinship\tbam.parser.kinship\tbam_kinship_normalized_v1\tsample_identity\tsample_identity\tidentity_inference\tbam_report_map\tcovered\tactive row `bam` / `bam.kinship` / `king` appears in governed report section `sample_identity` with summary table `identity_inference` through `bam_report_map`"
    }));
    assert!(rows.iter().any(|row| {
        row == &"vcf:vcf_production_regression:vcf.postprocess:vcf_single_sample:bcftools\tvcf\tvcf.postprocess\tbcftools\tvcf_production_regression\tvcf_single_sample\tvcf.adapter.transform\tvcf.parser.vcf_output\tbijux.schemas.bench.vcf-normalized-metrics.postprocess.v1\tnormalization\tnormalization\tnormalization_metrics\tvcf_report_map\tcovered\tactive row `vcf` / `vcf.postprocess` / `bcftools` appears in governed report section `normalization` with summary table `normalization_metrics` through `vcf_report_map`"
    }));
    assert!(
        rows.iter().all(|row| row.contains("\tcovered\t")),
        "every active binding must retain complete governed report-map coverage"
    );
}
