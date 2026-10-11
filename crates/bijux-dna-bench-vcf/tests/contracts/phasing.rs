use std::path::PathBuf;

use bijux_dna_bench_vcf::adapters::phasing::{
    render_vcf_phasing_family_adapter, DEFAULT_VCF_BEAGLE_ADAPTER_PATH,
    DEFAULT_VCF_EAGLE_ADAPTER_PATH, DEFAULT_VCF_SHAPEIT5_ADAPTER_PATH,
    DEFAULT_VCF_SHAPEIT_ADAPTER_PATH,
};

fn repo_root() -> PathBuf {
    crate::support::repo_root()
}

#[test]
fn vcf_shapeit5_adapter_tracks_governed_rows() {
    let report = render_vcf_phasing_family_adapter(
        &crate::support::inputs(&repo_root()),
        &repo_root(),
        "shapeit5",
        &PathBuf::from(DEFAULT_VCF_SHAPEIT5_ADAPTER_PATH),
    )
    .expect("render shapeit5 adapter");
    assert_eq!(report.schema_version, "bijux.bench.readiness.vcf_shapeit5_adapter.v1");
    assert_eq!(report.tool_id, "shapeit5");
    assert_eq!(report.row_count, 1);
    assert_eq!(report.benchmark_ready_row_count, 1);
    assert_eq!(report.indexed_row_count, 1);
}

#[test]
fn vcf_eagle_adapter_tracks_governed_rows() {
    let report = render_vcf_phasing_family_adapter(
        &crate::support::inputs(&repo_root()),
        &repo_root(),
        "eagle",
        &PathBuf::from(DEFAULT_VCF_EAGLE_ADAPTER_PATH),
    )
    .expect("render eagle adapter");
    assert_eq!(report.schema_version, "bijux.bench.readiness.vcf_eagle_adapter.v1");
    assert_eq!(report.tool_id, "eagle");
    assert_eq!(report.row_count, 1);
    assert_eq!(report.benchmark_ready_row_count, 1);
    assert_eq!(report.indexed_row_count, 1);
}

#[test]
fn vcf_shapeit_adapter_tracks_governed_rows() {
    let report = render_vcf_phasing_family_adapter(
        &crate::support::inputs(&repo_root()),
        &repo_root(),
        "shapeit",
        &PathBuf::from(DEFAULT_VCF_SHAPEIT_ADAPTER_PATH),
    )
    .expect("render shapeit adapter");
    assert_eq!(report.schema_version, "bijux.bench.readiness.vcf_shapeit_adapter.v1");
    assert_eq!(report.tool_id, "shapeit");
    assert_eq!(report.row_count, 1);
    assert_eq!(report.benchmark_ready_row_count, 1);
    assert_eq!(report.indexed_row_count, 1);
}

#[test]
fn vcf_beagle_adapter_tracks_governed_rows() {
    let report = render_vcf_phasing_family_adapter(
        &crate::support::inputs(&repo_root()),
        &repo_root(),
        "beagle",
        &PathBuf::from(DEFAULT_VCF_BEAGLE_ADAPTER_PATH),
    )
    .expect("render beagle adapter");
    assert_eq!(report.schema_version, "bijux.bench.readiness.vcf_beagle_adapter.v1");
    assert_eq!(report.tool_id, "beagle");
    assert_eq!(report.row_count, 3);
    assert_eq!(report.benchmark_ready_row_count, 3);
    assert_eq!(report.indexed_row_count, 3);
}
