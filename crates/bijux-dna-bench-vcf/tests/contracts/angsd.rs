use std::path::PathBuf;

use bijux_dna_bench_vcf::adapters::angsd::{
    render_vcf_angsd_adapter, DEFAULT_VCF_ANGSD_ADAPTER_PATH, VCF_ANGSD_ADAPTER_SCHEMA_VERSION,
};

fn repo_root() -> PathBuf {
    crate::support::repo_root()
}

#[test]
fn vcf_angsd_adapter_tracks_governed_rows() {
    let repo_root = repo_root();
    let report = render_vcf_angsd_adapter(
        &crate::support::inputs(&repo_root),
        &repo_root,
        &PathBuf::from(DEFAULT_VCF_ANGSD_ADAPTER_PATH),
    )
    .expect("render VCF angsd adapter");

    assert_eq!(report.schema_version, VCF_ANGSD_ADAPTER_SCHEMA_VERSION);
    assert_eq!(report.domain, "vcf");
    assert_eq!(report.tool_id, "angsd");
    assert_eq!(report.tool_status, "external_artifact");
    assert_eq!(report.row_count, 2);
    assert_eq!(report.supported_stage_row_count, 2);
    assert_eq!(report.benchmark_ready_row_count, 2);
    assert_eq!(report.argv_valid_row_count, 2);
    assert_eq!(report.missing_input_test_passed_row_count, 2);
    assert_eq!(report.bam_list_row_count, 2);
    assert_eq!(report.parser_output_row_count, 2);

    let call_gl =
        report.rows.iter().find(|row| row.stage_id == "vcf.call_gl").expect("call_gl row");
    assert_eq!(call_gl.command_steps.len(), 3);
    assert_eq!(call_gl.command_steps[0].argv[0], "angsd");
    assert!(
        call_gl.command_steps[0].argv.iter().any(|part| part == "-GL"),
        "call_gl row must keep ANGSD genotype-likelihood flags"
    );
    assert_eq!(call_gl.command_steps[1].step_kind, "result_normalizer");
    assert_eq!(call_gl.command_steps[1].argv[0], "bcftools");
    assert_eq!(call_gl.command_steps[2].step_kind, "index");
    assert!(call_gl.declared_outputs.iter().any(|output| output.artifact_id == "gl_sites_vcf_tbi"));

    assert!(report
        .rows
        .iter()
        .all(|row| { matches!(row.stage_id.as_str(), "vcf.call_gl" | "vcf.call_pseudohaploid") }));
}
