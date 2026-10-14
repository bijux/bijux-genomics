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
    assert_declared_phasing_target(&report.rows, "--input");
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
    assert_declared_phasing_target(&report.rows, "--vcfTarget");
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
    assert_declared_phasing_target(&report.rows, "--input");
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
    assert_declared_phasing_target(&report.rows, "gt=");
}

fn assert_declared_phasing_target(
    rows: &[bijux_dna_bench_vcf::adapters::phasing::VcfPhasingFamilyAdapterRow],
    input_flag: &str,
) {
    for row in rows {
        let declared = row
            .required_inputs
            .iter()
            .find(|input| input.artifact_id == "vcf")
            .expect("declared target VCF");
        assert_eq!(declared.path, row.input_vcf_path);
        let target = if input_flag == "gt=" {
            format!("gt='{}'", declared.path)
        } else {
            format!("{input_flag} '{}'", declared.path)
        };
        assert!(
            row.command_steps
                .iter()
                .filter(|step| step.step_id == "phase")
                .flat_map(|step| &step.argv)
                .any(|argument| argument.contains(&target)),
            "{} / {} command must use admitted target {}",
            row.stage_id,
            row.tool_id,
            declared.path
        );
    }
}
