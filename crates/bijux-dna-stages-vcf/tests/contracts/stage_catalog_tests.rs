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

#[test]
fn supported_stage_defaults_match_domain_index_defaults() {
    let defaults = parse_domain_index_active_defaults();
    for spec in bijux_dna_stages_vcf::stage_specs::vcf_stage_catalog() {
        if spec.status != "supported" {
            continue;
        }
        assert_eq!(
            defaults.get(spec.stage_id).map(String::as_str),
            Some(spec.default_tool_id),
            "supported stage {} default drifted from domain/vcf/index.yaml",
            spec.stage_id
        );
    }
}

fn parse_domain_index_active_defaults() -> std::collections::BTreeMap<String, String> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../");
    let raw = std::fs::read_to_string(root.join("domain/vcf/index.yaml"))
        .unwrap_or_else(|err| panic!("read domain/vcf/index.yaml: {err}"));
    let mut defaults = std::collections::BTreeMap::new();
    let mut in_block = false;
    for line in raw.lines() {
        if line == "active_defaults:" {
            in_block = true;
            continue;
        }
        if in_block && !line.starts_with(' ') {
            break;
        }
        if !in_block {
            continue;
        }
        if let Some((stage_id, tool_id)) = line.trim().split_once(':') {
            defaults.insert(
                stage_id.trim().to_string(),
                tool_id.trim().trim_matches('"').to_string(),
            );
        }
    }
    defaults
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
