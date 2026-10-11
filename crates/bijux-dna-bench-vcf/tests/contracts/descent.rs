use bijux_dna_bench_vcf::adapters::descent::{
    render_vcf_descent_family_adapter, DEFAULT_VCF_DESCENT_FAMILY_ADAPTER_PATH,
};
use std::path::PathBuf;

#[test]
fn vcf_descent_family_adapter_tracks_governed_rows() {
    let root = crate::support::repo_root();
    let report = render_vcf_descent_family_adapter(
        &crate::support::inputs(&root),
        &root,
        &PathBuf::from(DEFAULT_VCF_DESCENT_FAMILY_ADAPTER_PATH),
    )
    .expect("render VCF descent-family adapter");

    assert_eq!(report.row_count, 5);
    assert_eq!(report.tool_count, 5);
    assert_eq!(report.benchmark_ready_row_count, 5);
    assert_eq!(report.parser_output_row_count, 5);
    assert_eq!(report.missing_input_test_passed_row_count, 5);

    let roh = report
        .rows
        .iter()
        .find(|row| row.tool_id == "plink2" && row.stage_id == "vcf.roh")
        .expect("plink2 roh row");
    assert_eq!(roh.normalized_output_artifact_id, "roh_report");
    assert!(
        roh.command_steps[0].argv.iter().any(|part| part == "--homozyg"),
        "plink2 roh row must retain homozyg command flags"
    );

    let ibdhap = report
        .rows
        .iter()
        .find(|row| row.tool_id == "ibdhap" && row.stage_id == "vcf.ibd")
        .expect("ibdhap IBD row");
    assert_eq!(ibdhap.benchmark_status, "benchmark_ready");
    assert_eq!(ibdhap.normalized_output_artifact_id, "ibd_segments");

    let demography = report
        .rows
        .iter()
        .find(|row| row.tool_id == "ibdne" && row.stage_id == "vcf.demography")
        .expect("ibdne demography row");
    assert_eq!(demography.benchmark_status, "benchmark_ready");
    assert!(
        demography
            .input_ibd_segments_path
            .as_deref()
            .is_some_and(|path| path.ends_with("artifacts/input/ibd_segments.tsv")),
        "demography row must materialize a governed IBD segment input"
    );
}
