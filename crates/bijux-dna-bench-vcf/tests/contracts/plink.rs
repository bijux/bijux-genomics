use std::path::PathBuf;

use bijux_dna_bench_vcf::adapters::plink::{
    render_vcf_plink_family_adapter, DEFAULT_VCF_PLINK2_ADAPTER_PATH,
    DEFAULT_VCF_PLINK_ADAPTER_PATH,
};

fn repo_root() -> PathBuf {
    crate::support::repo_root()
}

#[test]
fn vcf_plink_adapter_tracks_governed_rows() {
    let repo_root = repo_root();
    let report = render_vcf_plink_family_adapter(
        &crate::support::inputs(&repo_root),
        &repo_root,
        "plink",
        &PathBuf::from(DEFAULT_VCF_PLINK_ADAPTER_PATH),
    )
    .expect("render VCF plink adapter");

    assert_eq!(report.schema_version, "bijux.bench.readiness.vcf_plink_adapter.v1");
    assert_eq!(report.tool_id, "plink");
    assert_eq!(report.tool_status, "production");
    assert_eq!(report.row_count, 3);
    assert_eq!(report.benchmark_ready_row_count, 3);
    assert_eq!(report.parser_output_row_count, 3);
    assert_eq!(report.normalized_metrics_row_count, 3);
    assert_eq!(report.raw_output_declared_row_count, 3);

    let qc = report.rows.iter().find(|row| row.stage_id == "vcf.qc").expect("plink qc row");
    assert_eq!(qc.normalized_metrics_artifact_id, "qc_report");
    assert!(
        qc.command_steps[0].argv.iter().any(|part| part == "--missing"),
        "plink qc row must retain missingness output flags"
    );
    let admixture = report
        .rows
        .iter()
        .find(|row| row.stage_id == "vcf.admixture")
        .expect("plink admixture row");
    assert!(
        admixture.command_steps[0].argv.iter().any(|part| part == "--make-bed"),
        "plink admixture row must retain cohort-preparation outputs"
    );
    let population_structure = report
        .rows
        .iter()
        .find(|row| row.stage_id == "vcf.population_structure")
        .expect("plink population_structure row");
    assert!(population_structure.command_steps[0].argv.iter().any(|part| part == "--mds-plot"));
}

#[test]
fn vcf_plink2_adapter_tracks_governed_rows() {
    let repo_root = repo_root();
    let report = render_vcf_plink_family_adapter(
        &crate::support::inputs(&repo_root),
        &repo_root,
        "plink2",
        &PathBuf::from(DEFAULT_VCF_PLINK2_ADAPTER_PATH),
    )
    .expect("render VCF plink2 adapter");

    assert_eq!(report.schema_version, "bijux.bench.readiness.vcf_plink2_adapter.v1");
    assert_eq!(report.tool_id, "plink2");
    assert_eq!(report.tool_status, "production");
    assert_eq!(report.row_count, 5);
    assert_eq!(report.benchmark_ready_row_count, 5);
    assert_eq!(report.parser_output_row_count, 5);
    assert_eq!(report.normalized_metrics_row_count, 5);
    assert_eq!(report.raw_output_declared_row_count, 5);

    let pca = report.rows.iter().find(|row| row.stage_id == "vcf.pca").expect("plink2 pca row");
    assert_eq!(pca.normalized_metrics_artifact_id, "pca_report");
    assert!(
        pca.command_steps[0].argv.iter().any(|part| part == "--pca"),
        "plink2 pca row must keep eigenvector/eigenvalue output flags"
    );
    let roh = report.rows.iter().find(|row| row.stage_id == "vcf.roh").expect("plink2 roh row");
    assert!(
        roh.command_steps[0].argv.iter().any(|part| part == "--homozyg"),
        "plink2 roh row must retain ROH HOM output flags"
    );
    let population_structure = report
        .rows
        .iter()
        .find(|row| row.stage_id == "vcf.population_structure")
        .expect("plink2 population_structure row");
    assert_eq!(population_structure.command_steps.len(), 2);
}
