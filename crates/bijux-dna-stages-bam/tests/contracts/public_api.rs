use std::path::Path;

#[test]
fn public_api_docs_list_stable_root_exports() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let public_api =
        std::fs::read_to_string(root.join("docs/PUBLIC_API.md")).expect("read docs/PUBLIC_API.md");
    let readme = std::fs::read_to_string(root.join("README.md")).expect("read README.md");
    let stable_exports = ["BamStagePlugin", "StagePlanJson", "implemented_stages"];

    for export in stable_exports {
        assert!(public_api.contains(export), "docs/PUBLIC_API.md must list `{export}`");
        assert!(readme.contains(export), "README.md must list `{export}`");
    }
}

#[test]
fn public_api_exports_remain_usable_from_crate_root() {
    let _plugin = bijux_dna_stages_bam::BamStagePlugin;
    let _plan_json: Option<bijux_dna_stages_bam::StagePlanJson> = None;
    let stages = bijux_dna_stages_bam::implemented_stages();

    assert!(!stages.is_empty(), "implemented_stages must expose BAM stages");
}

#[test]
fn public_modules_remain_available() {
    let _metrics_fn = bijux_dna_stages_bam::metrics::bam_metrics_from_dir;
    let _flagstat_parser = bijux_dna_stages_bam::observer::parse_samtools_flagstat;
    let _stage_count = bijux_dna_stages_bam::stage_specs::BamStage::all().len();
}
