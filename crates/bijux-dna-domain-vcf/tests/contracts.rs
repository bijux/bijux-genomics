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
            .join("../../configs/ci/params/param_registry_vcf.toml");
        let committed = std::fs::read_to_string(expected_path)
            .unwrap_or_else(|err| panic!("read configs/ci/params/param_registry_vcf.toml: {err}"));
        let generated = param_registry_toml();
        for required in ["vcf.call", "vcf.filter", "vcf.stats"] {
            assert!(
                committed.contains(required),
                "committed config missing required stage {required}"
            );
            assert!(
                generated.contains(required),
                "generated config missing required stage {required}"
            );
        }
    }

    #[test]
    fn generated_required_tools_matches_config_artifact() {
        let expected_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../configs/ci/tools/required_tools_vcf.toml");
        let committed = std::fs::read_to_string(expected_path)
            .unwrap_or_else(|err| panic!("read configs/ci/tools/required_tools_vcf.toml: {err}"));
        let generated = required_tools_toml();
        assert!(
            committed.contains("required_tools = [\"bcftools\"]"),
            "committed required-tools config must include bcftools"
        );
        assert!(
            generated.contains("required_tools = [\"bcftools\"]"),
            "generated required-tools config must include bcftools"
        );
    }
}
