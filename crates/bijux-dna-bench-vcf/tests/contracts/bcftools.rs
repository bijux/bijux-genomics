use std::path::PathBuf;

use bijux_dna_bench_vcf::adapters::bcftools::{
    render_vcf_bcftools_adapter, DEFAULT_VCF_BCFTOOLS_ADAPTER_PATH,
    VCF_BCFTOOLS_ADAPTER_SCHEMA_VERSION,
};

fn repo_root() -> PathBuf {
    crate::support::repo_root()
}

#[test]
fn vcf_bcftools_adapter_tracks_governed_rows() {
    let root = repo_root();
    let report = render_vcf_bcftools_adapter(
        &crate::support::inputs(&root),
        &root,
        &PathBuf::from(DEFAULT_VCF_BCFTOOLS_ADAPTER_PATH),
    )
    .expect("render VCF bcftools adapter");

    assert_eq!(report.schema_version, VCF_BCFTOOLS_ADAPTER_SCHEMA_VERSION);
    assert_eq!(report.domain, "vcf");
    assert_eq!(report.tool_id, "bcftools");
    assert_eq!(report.row_count, 11);
    assert_eq!(report.supported_row_count, 11);
    assert_eq!(report.planned_row_count, 0);
    assert_eq!(report.argv_valid_row_count, 11);
    assert_eq!(report.missing_input_test_passed_row_count, 11);
    assert_eq!(report.indexed_row_count, 9);
    assert!(report.rows.iter().any(|row| {
        row.stage_id == "vcf.call_diploid"
            && row.command_steps.iter().any(|step| {
                step.step_id == "call_diploid"
                    && step.argv
                        == vec!["bcftools", "call", "-mv", "-Oz", "-o",
                            "artifacts/benchmark-smoke/readiness/adapters/bcftools/vcf.call_diploid/diploid_vcf.vcf.gz"]
            })
    }));
    assert!(report.rows.iter().any(|row| {
        row.stage_id == "vcf.prepare_reference_panel"
            && row.parser_output_ids == vec!["chunks_json".to_string()]
            && row.command_steps.iter().any(|step| step.step_kind == "index")
    }));
    assert!(report.rows.iter().any(|row| {
        row.stage_id == "vcf.stats"
            && row.raw_output_ids == vec!["bcftools_stats_txt".to_string()]
            && row.parser_output_ids == vec!["stats_json".to_string()]
    }));
    assert!(report.rows.iter().any(|row| {
        row.stage_id == "vcf.qc"
            && row.raw_output_ids.len() == 6
            && row.parser_output_ids == vec!["qc_report".to_string()]
            && row.command_steps.iter().any(|step| step.step_id == "extract_genotypes")
            && row.command_steps.iter().any(|step| step.step_id == "normalize_qc")
    }));
}
