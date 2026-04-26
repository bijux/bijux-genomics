#[test]
fn stage_contract_docs_list_every_domain_stage_id() {
    let stage_contracts = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/STAGE_CONTRACTS.md"),
    )
    .unwrap_or_else(|err| panic!("read docs/STAGE_CONTRACTS.md: {err}"));

    for stage_id in bijux_dna_domain_vcf::VCF_STAGE_ID_CATALOG {
        assert!(
            stage_contracts.contains(&format!("`{stage_id}`")),
            "docs/STAGE_CONTRACTS.md must list VCF domain stage {stage_id}"
        );
    }
}

#[test]
fn stage_contract_docs_describe_artifact_and_refusal_duties() {
    let stage_contracts = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/STAGE_CONTRACTS.md"),
    )
    .unwrap_or_else(|err| panic!("read docs/STAGE_CONTRACTS.md: {err}"));

    for required_phrase in [
        "caller-provided output directories",
        "typed metrics",
        "Refusal paths",
        "External-tool fallback",
    ] {
        assert!(
            stage_contracts.contains(required_phrase),
            "docs/STAGE_CONTRACTS.md must describe {required_phrase}"
        );
    }
}
