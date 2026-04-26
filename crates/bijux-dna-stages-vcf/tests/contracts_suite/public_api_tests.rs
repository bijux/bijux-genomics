#[test]
fn public_api_docs_list_public_modules_and_root_exports() {
    let public_api = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/PUBLIC_API.md"),
    )
    .unwrap_or_else(|err| panic!("read docs/PUBLIC_API.md: {err}"));

    for public_module in [
        "engine",
        "invariants",
        "metrics",
        "path_contract",
        "pipeline",
        "stage_specs",
        "vcf_io",
        "wrappers",
    ] {
        assert!(
            public_api.contains(&format!("`{public_module}`")),
            "docs/PUBLIC_API.md must list public module {public_module}"
        );
    }

    assert!(
        public_api.contains("`implemented_stages`"),
        "docs/PUBLIC_API.md must list implemented_stages"
    );
}

#[test]
fn public_api_exports_remain_usable_from_crate_root() {
    let _ = bijux_dna_stages_vcf::implemented_stages();

    let _ = std::any::type_name::<bijux_dna_stages_vcf::engine::VcfPipelineRequest>();
    let _ = std::any::type_name::<bijux_dna_stages_vcf::invariants::InvariantConfig>();
    let _ = std::any::type_name::<bijux_dna_stages_vcf::path_contract::VcfPathContract>();
    let _ = std::any::type_name::<bijux_dna_stages_vcf::stage_specs::VcfStageSpec>();
    let _ = std::any::type_name::<bijux_dna_stages_vcf::vcf_io::VcfFieldRequirement>();
    let _ = std::any::type_name::<bijux_dna_stages_vcf::wrappers::ToolVersionCheck>();

    let _ = bijux_dna_stages_vcf::metrics::parse_depth_from_info("DP=3");
    let _ = std::any::type_name::<bijux_dna_stages_vcf::pipeline::QcStageParams>();
}
