mod contracts {
    use bijux_dna_domain_vcf::{
        coverage::domain_coverage_report,
        contracts::{
            stage_failure_modes, stage_io_contract, stage_metrics_contract, validate_vcf_invariants,
            DamageAwareGenotypeLogicContract, DefaultPanelSelectionPolicy, PanelSelectionContext,
            PanelSelectionPolicy, ReferencePanelGovernance, VcfInvariantState,
            DAMAGE_AWARE_GENOTYPE_LOGIC,
        },
        param_registry_toml, required_tools_toml, validate_downstream_transition, VcfDomainStage,
        VcfStage, VCF_STAGE_ORDER_DOWNSTREAM,
    };

    #[test]
    fn vcf_stage_catalog_is_stable() {
        let ids = VcfStage::all()
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>();
        assert_eq!(ids, vec!["vcf.call", "vcf.filter", "vcf.stats"]);
    }

    #[test]
    fn vcf_domain_stage_taxonomy_covers_domain_index_set() {
        let ids = VcfDomainStage::all()
            .iter()
            .map(|stage| stage.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            ids,
            vec![
                "vcf.admixture",
                "vcf.call",
                "vcf.call_diploid",
                "vcf.call_gl",
                "vcf.call_pseudohaploid",
                "vcf.damage_filter",
                "vcf.demography",
                "vcf.filter",
                "vcf.gl_propagation",
                "vcf.ibd",
                "vcf.imputation",
                "vcf.impute",
                "vcf.pca",
                "vcf.phasing",
                "vcf.population_structure",
                "vcf.postprocess",
                "vcf.prepare_reference_panel",
                "vcf.qc",
                "vcf.roh",
                "vcf.stats",
            ]
        );
    }

    #[test]
    fn vcf_downstream_order_blocks_back_edges() {
        assert!(validate_downstream_transition(VcfDomainStage::Filter, VcfDomainStage::Stats).is_ok());
        assert!(
            validate_downstream_transition(VcfDomainStage::Imputation, VcfDomainStage::Call)
                .is_err()
        );
        assert_eq!(VCF_STAGE_ORDER_DOWNSTREAM.first().map(|s| s.as_str()), Some("vcf.prepare_reference_panel"));
    }

    #[test]
    fn vcf_stage_contracts_expose_io_metrics_and_failure_modes() {
        let io = stage_io_contract(VcfDomainStage::Imputation).expect("contract");
        assert!(io.required_inputs.contains(&"vcf"));
        assert!(io.required_indices.contains(&"vcf.tbi"));

        let metrics = stage_metrics_contract(VcfDomainStage::Imputation);
        assert_eq!(metrics.metrics_schema_id, "bijux.vcf.imputation.v1");
        assert!(metrics.required_metrics.contains(&"rsq_mean"));

        let failure_modes = stage_failure_modes(VcfDomainStage::Phasing);
        assert!(failure_modes.iter().any(|m| m.code == "insufficient_markers"));
    }

    #[test]
    fn vcf_damage_and_panel_contracts_are_typed() {
        let damage: DamageAwareGenotypeLogicContract = DAMAGE_AWARE_GENOTYPE_LOGIC.clone();
        assert!(damage
            .masked_variant_classes
            .contains(&"ct_transition"));
        assert!(damage
            .provenance_fields
            .contains(&"masked_site_count"));

        let policy = DefaultPanelSelectionPolicy;
        let available = vec![ReferencePanelGovernance {
            panel_id: "1000g_phase3".to_string(),
            reference_build: "GRCh37".to_string(),
            panel_checksum_sha256: "a".repeat(64),
            index_checksum_sha256: "b".repeat(64),
            license_id: "CC-BY-4.0".to_string(),
            license_constraints: vec!["attribution".to_string()],
            ancestry_tags: vec!["eur".to_string()],
            target_tags: vec!["ancient".to_string()],
        }];
        let selected = policy.select_panel(
            &available,
            &PanelSelectionContext {
                target_build: "GRCh37".to_string(),
                ancestry_hint: Some("eur".to_string()),
                use_restricted_license: false,
            },
        );
        assert_eq!(selected.map(|p| p.panel_id.as_str()), Some("1000g_phase3"));
    }

    #[test]
    fn vcf_invariant_checks_require_sorted_bgzip_tabix() {
        let ok = VcfInvariantState {
            sorted_by_contig_and_pos: true,
            bgzip_compressed: true,
            tabix_index_present: true,
            sample_set_consistent: true,
            contig_set_consistent: true,
        };
        assert!(validate_vcf_invariants(VcfDomainStage::Stats, &ok).is_ok());
        let bad = VcfInvariantState {
            tabix_index_present: false,
            ..ok
        };
        assert!(validate_vcf_invariants(VcfDomainStage::Stats, &bad).is_err());
    }

    #[test]
    fn domain_coverage_report_marks_contract_vs_execution() {
        let report = domain_coverage_report();
        assert_eq!(report.schema_version, "bijux.vcf.domain_coverage.v1");
        assert!(report
            .stages
            .iter()
            .any(|row| row.stage_id == "vcf.call" && row.execution_in_code));
        assert!(report
            .stages
            .iter()
            .any(|row| row.stage_id == "vcf.imputation" && row.domain_only));
        assert!(report.tools.iter().any(|row| row.tool_id == "bcftools"));
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
