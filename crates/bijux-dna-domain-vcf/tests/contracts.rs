mod contracts {
    use bijux_dna_domain_vcf::{param_registry_toml, required_tools_toml, VcfStage};

    #[test]
    fn vcf_stage_catalog_is_stable() {
        let ids = VcfStage::all()
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>();
        assert_eq!(ids, vec!["vcf.call", "vcf.filter", "vcf.stats"]);
    }

    #[test]
    fn param_registry_contains_all_vcf_stages() {
        let registry = param_registry_toml();
        for stage in ["vcf.call", "vcf.filter", "vcf.stats"] {
            assert!(registry.contains(stage), "missing stage {stage}");
        }
    }

    #[test]
    fn generated_param_registry_matches_config_artifact() {
        let expected_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../configs/param_registry_vcf.toml");
        let expected = std::fs::read_to_string(expected_path)
            .unwrap_or_else(|err| panic!("read configs/param_registry_vcf.toml: {err}"));
        assert_eq!(param_registry_toml().trim(), expected.trim());
    }

    #[test]
    fn generated_required_tools_matches_config_artifact() {
        let expected_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../configs/required_tools_vcf.toml");
        let expected = std::fs::read_to_string(expected_path)
            .unwrap_or_else(|err| panic!("read configs/required_tools_vcf.toml: {err}"));
        assert_eq!(required_tools_toml().trim(), expected.trim());
    }
}
