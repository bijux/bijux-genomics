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
fn implemented_stages_match_supported_stage_catalog() {
    let implemented = bijux_dna_stages_vcf::implemented_stages()
        .into_iter()
        .map(|stage| stage.as_str().to_string())
        .collect::<std::collections::BTreeSet<_>>();
    let supported = bijux_dna_stages_vcf::stage_specs::supported_vcf_stages()
        .into_iter()
        .map(str::to_string)
        .collect::<std::collections::BTreeSet<_>>();

    assert_eq!(
        implemented, supported,
        "implemented_stages must expose the supported VCF stage surface implemented here"
    );
}

#[test]
fn vcf_domain_stage_completeness_matches_supported_catalog_rows() {
    for spec in bijux_dna_stages_vcf::stage_specs::vcf_stage_catalog() {
        let stage = bijux_dna_domain_vcf::VcfDomainStage::all()
            .iter()
            .copied()
            .find(|stage| stage.as_str() == spec.stage_id)
            .unwrap_or_else(|| panic!("catalog stage {} missing from domain enum", spec.stage_id));
        assert_eq!(
            bijux_dna_stages_vcf::stage_specs::vcf_domain_stage_completeness(stage),
            spec.status == "supported",
            "domain stage {} completeness drifted from stage status",
            spec.stage_id
        );
    }
}

#[test]
fn stage_catalog_entries_have_metric_schema_versions() {
    for spec in bijux_dna_stages_vcf::stage_specs::vcf_stage_catalog() {
        assert!(
            spec.metrics_schema.starts_with("bijux.vcf.")
                && std::path::Path::new(spec.metrics_schema)
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("v1")),
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
fn stage_catalog_rows_keep_benchmark_adapter_and_parser_contract_ids() {
    for spec in bijux_dna_stages_vcf::stage_specs::vcf_stage_catalog() {
        let stage = bijux_dna_domain_vcf::VcfDomainStage::all()
            .iter()
            .copied()
            .find(|stage| stage.as_str() == spec.stage_id)
            .unwrap_or_else(|| panic!("catalog stage {} missing from domain enum", spec.stage_id));
        assert!(
            bijux_dna_stages_vcf::stage_specs::vcf_domain_stage_adapter_id(stage)
                .is_some_and(|adapter_id| !adapter_id.trim().is_empty()),
            "stage {} must declare a benchmark adapter contract id",
            spec.stage_id
        );
        assert!(
            bijux_dna_stages_vcf::stage_specs::vcf_domain_stage_parser_id(stage)
                .is_some_and(|parser_id| !parser_id.trim().is_empty()),
            "stage {} must declare a benchmark parser contract id",
            spec.stage_id
        );
    }
}

#[test]
fn stage_catalog_rows_keep_expected_output_contract_ids() {
    for spec in bijux_dna_stages_vcf::stage_specs::vcf_stage_catalog() {
        let stage = bijux_dna_domain_vcf::VcfDomainStage::all()
            .iter()
            .copied()
            .find(|stage| stage.as_str() == spec.stage_id)
            .unwrap_or_else(|| panic!("catalog stage {} missing from domain enum", spec.stage_id));
        assert!(
            bijux_dna_stages_vcf::stage_specs::vcf_domain_stage_expected_output_ids(stage)
                .is_some_and(|output_ids| !output_ids.is_empty()),
            "stage {} must declare benchmark output ids",
            spec.stage_id
        );
    }

    let prepare_reference_panel_outputs =
        bijux_dna_stages_vcf::stage_specs::vcf_domain_stage_expected_output_ids(
            bijux_dna_domain_vcf::VcfDomainStage::PrepareReferencePanel,
        )
        .unwrap_or_else(|| panic!("prepare reference panel outputs"));
    assert_eq!(prepare_reference_panel_outputs, ["prepared_panel", "chunks_json"]);

    let stats_outputs = bijux_dna_stages_vcf::stage_specs::vcf_domain_stage_expected_output_ids(
        bijux_dna_domain_vcf::VcfDomainStage::Stats,
    )
    .unwrap_or_else(|| panic!("stats outputs"));
    assert_eq!(stats_outputs, ["stats_json"]);
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

#[test]
fn default_settings_doc_defaults_match_stage_catalog() {
    let defaults = parse_default_settings_doc_defaults();
    for spec in bijux_dna_stages_vcf::stage_specs::vcf_stage_catalog() {
        assert_eq!(
            defaults.get(spec.stage_id).map(String::as_str),
            Some(spec.default_tool_id),
            "default settings doc drifted for {}",
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
            defaults
                .insert(stage_id.trim().to_string(), tool_id.trim().trim_matches('"').to_string());
        }
    }
    defaults
}

fn parse_default_settings_doc_defaults() -> std::collections::BTreeMap<String, String> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../");
    let raw = std::fs::read_to_string(root.join("domain/vcf/docs/DEFAULT_SETTINGS.md"))
        .unwrap_or_else(|err| panic!("read domain/vcf/docs/DEFAULT_SETTINGS.md: {err}"));
    raw.lines()
        .filter_map(|line| {
            let line = line.trim();
            let stage_start = line.strip_prefix("- `")?;
            let (stage_id, rest) = stage_start.split_once("` default: `")?;
            let (tool_id, _) = rest.split_once('`')?;
            Some((stage_id.to_string(), tool_id.to_string()))
        })
        .collect()
}

fn catalog_stage_ids() -> std::collections::BTreeSet<String> {
    bijux_dna_stages_vcf::stage_specs::vcf_stage_catalog()
        .iter()
        .map(|spec| spec.stage_id.to_string())
        .collect()
}

fn domain_stage_ids() -> std::collections::BTreeSet<String> {
    bijux_dna_domain_vcf::VCF_STAGE_ID_CATALOG.iter().map(|stage| (*stage).to_string()).collect()
}
