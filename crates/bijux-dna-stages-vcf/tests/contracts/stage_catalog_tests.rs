#[test]
fn stage_catalog_covers_every_domain_vcf_stage_id() {
    let catalog = catalog_stage_ids();
    let domain = domain_stage_ids();

    assert_eq!(
        catalog, domain,
        "stage_specs::vcf_stage_catalog must cover the VCF domain stage catalog exactly"
    );
}

#[test]
fn implemented_stages_match_domain_vcf_stage_catalog() {
    let implemented = bijux_dna_stages_vcf::implemented_stages()
        .into_iter()
        .map(|stage| stage.as_str().to_string())
        .collect::<std::collections::BTreeSet<_>>();

    assert_eq!(
        implemented,
        domain_stage_ids(),
        "implemented_stages must expose the full VCF domain stage surface implemented here"
    );
}

#[test]
fn vcf_domain_stage_completeness_accepts_every_catalog_stage() {
    for stage in bijux_dna_domain_vcf::VcfDomainStage::all() {
        assert!(
            bijux_dna_stages_vcf::stage_specs::vcf_domain_stage_completeness(*stage),
            "domain stage {} must be complete in stages-vcf catalog",
            stage.as_str()
        );
    }
}

#[test]
fn stage_catalog_entries_have_metric_schema_versions() {
    for spec in bijux_dna_stages_vcf::stage_specs::vcf_stage_catalog() {
        assert!(
            spec.metrics_schema.starts_with("bijux.vcf.") && spec.metrics_schema.ends_with(".v1"),
            "stage {} has invalid metrics schema {}",
            spec.stage_id,
            spec.metrics_schema
        );
    }
}

#[test]
fn supported_vcf_stage_helper_matches_supported_catalog_rows() {
    let supported = bijux_dna_stages_vcf::stage_specs::supported_vcf_stages()
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>();
    let catalog_supported = bijux_dna_stages_vcf::stage_specs::vcf_stage_catalog()
        .iter()
        .filter(|spec| spec.status == "supported")
        .map(|spec| spec.stage_id)
        .collect::<std::collections::BTreeSet<_>>();

    assert_eq!(
        supported, catalog_supported,
        "supported_vcf_stages must track the supported rows in the stage catalog"
    );
}

#[test]
fn stage_catalog_default_tools_are_present_in_runtime_surface() {
    for spec in bijux_dna_stages_vcf::stage_specs::vcf_stage_catalog() {
        let stage = bijux_dna_domain_vcf::VcfDomainStage::all()
            .iter()
            .copied()
            .find(|stage| stage.as_str() == spec.stage_id)
            .unwrap_or_else(|| panic!("catalog stage {} missing from domain enum", spec.stage_id));
        assert!(
            bijux_dna_stages_vcf::stage_specs::vcf_domain_stage_default_tool_id(stage)
                == Some(spec.default_tool_id),
            "stage {} default tool lookup drifted from the catalog",
            spec.stage_id
        );
    }
}

fn catalog_stage_ids() -> std::collections::BTreeSet<String> {
    bijux_dna_stages_vcf::stage_specs::vcf_stage_catalog()
        .iter()
        .map(|spec| spec.stage_id.to_string())
        .collect()
}

fn domain_stage_ids() -> std::collections::BTreeSet<String> {
    bijux_dna_domain_vcf::VCF_STAGE_ID_CATALOG
        .iter()
        .map(|stage| (*stage).to_string())
        .collect()
}
