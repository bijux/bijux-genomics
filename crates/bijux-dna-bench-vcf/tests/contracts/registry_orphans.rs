use std::path::PathBuf;

use bijux_dna_bench_vcf::evidence::registry::orphans::{
    render_vcf_orphan_tools, DEFAULT_VCF_ORPHAN_TOOLS_PATH, VCF_ORPHAN_TOOLS_SCHEMA_VERSION,
};

fn repo_root() -> PathBuf {
    crate::support::repo_root()
}

#[test]
fn vcf_orphan_tools_report_tracks_governed_decisions() {
    let root = repo_root();
    let report = render_vcf_orphan_tools(
        &crate::support::inputs(&root),
        &root,
        &PathBuf::from(DEFAULT_VCF_ORPHAN_TOOLS_PATH),
    )
    .expect("render VCF orphan tools");

    assert_eq!(report.schema_version, VCF_ORPHAN_TOOLS_SCHEMA_VERSION);
    assert_eq!(report.domain, "vcf");
    assert_eq!(report.required_tool_count, 16);
    assert_eq!(report.registered_tool_count, 16);
    assert_eq!(report.served_tool_count, report.required_tool_count);
    assert_eq!(report.orphan_count, 0);
    assert!(report.rows.is_empty());
}
