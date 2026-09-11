use std::path::PathBuf;

use bijux_dna_bench_vcf::evidence::registry::undercoverage::{
    render_vcf_undercovered_stages, DEFAULT_VCF_UNDERCOVERED_STAGES_PATH,
    VCF_UNDERCOVERED_STAGES_SCHEMA_VERSION,
};

fn repo_root() -> PathBuf {
    crate::support::repo_root()
}

#[test]
fn vcf_undercovered_stages_report_tracks_governed_stage_slice() {
    let root = repo_root();
    let report = render_vcf_undercovered_stages(
        &crate::support::inputs(&root),
        &root,
        &PathBuf::from(DEFAULT_VCF_UNDERCOVERED_STAGES_PATH),
    )
    .expect("render VCF undercovered stages");

    assert_eq!(report.schema_version, VCF_UNDERCOVERED_STAGES_SCHEMA_VERSION);
    assert_eq!(report.domain, "vcf");
    assert_eq!(report.stage_count, 20);
    assert_eq!(report.undercovered_stage_count, 0);
    assert!(report.decision_counts.is_empty());
    assert!(report.rows.is_empty());
}
