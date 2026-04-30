mod contracts {
    use bijux_dna_domain_vcf::{
        contracts::{
            refuse_unsupported_regime_transition, stage_artifact_contract, stage_failure_modes,
            stage_artifact_class_contract, stage_io_contract, stage_metrics_contract,
            validate_entry_vcf_invariants, validate_panel_map_invariants,
            validate_reference_panel_governance, validate_species_context,
            validate_vcf_invariants, vcf_calling_mode_contracts, vcf_panel_boundary_contracts,
            vcf_population_guardrail_contracts, ContigSpec,
            DamageAwareGenotypeLogicContract, DefaultPanelSelectionPolicy, EntryVcfInvariantState,
            PanelMapInvariantState, PanelSelectionContext, PanelSelectionPolicy,
            ReferencePanelGovernance, SpeciesContext, VcfArtifactClass, VcfInvariantState,
            VCF_FILTER_EVIDENCE_CONTRACT, VCF_NORMALIZATION_CONTRACT,
            VCF_REFERENCE_CONTEXT_CONTRACT, VCF_STATS_REPORT_CONTRACT,
            VCF_VALIDATION_CONTRACT, DAMAGE_AWARE_GENOTYPE_LOGIC, OUTPUT_GUARANTEE,
        },
        coverage::domain_coverage_report,
        param_registry_toml, required_tools_toml, validate_downstream_transition, CoverageRegime,
        VcfDomainStage, VcfStage, VcfStatsMetricsV1, VCF_METRICS_CATALOG, VCF_PARAMS_CATALOG,
        VCF_STAGE_ORDER_DOWNSTREAM,
    };

    #[test]
    fn vcf_stage_catalog_is_stable() {
        let ids = VcfStage::all().iter().map(|s| s.as_str()).collect::<Vec<_>>();
        assert_eq!(ids, vec!["vcf.call", "vcf.filter", "vcf.stats"]);
    }

    #[test]
    fn vcf_domain_stage_taxonomy_covers_domain_index_set() {
        let ids = VcfDomainStage::all().iter().map(|stage| stage.as_str()).collect::<Vec<_>>();
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
        assert!(
            validate_downstream_transition(VcfDomainStage::Filter, VcfDomainStage::Stats).is_ok()
        );
        assert!(
            validate_downstream_transition(VcfDomainStage::Filter, VcfDomainStage::Filter).is_err()
        );
        assert!(validate_downstream_transition(VcfDomainStage::Imputation, VcfDomainStage::Call)
            .is_err());
        assert_eq!(
            VCF_STAGE_ORDER_DOWNSTREAM.first().map(|s| s.as_str()),
            Some("vcf.prepare_reference_panel")
        );
    }

    #[test]
    fn vcf_stage_contracts_expose_io_metrics_and_failure_modes() {
        let Some(io) = stage_io_contract(VcfDomainStage::Imputation) else {
            panic!("missing stage IO contract for imputation");
        };
        assert!(io.required_inputs.contains(&"vcf"));
        assert!(io.required_indices.contains(&"vcf.tbi"));

        let metrics = stage_metrics_contract(VcfDomainStage::Imputation);
        assert_eq!(metrics.metrics_schema_id, "bijux.vcf.imputation.v1");
        assert!(metrics.required_metrics.contains(&"rsq_mean"));

        let failure_modes = stage_failure_modes(VcfDomainStage::Phasing);
        assert!(failure_modes.iter().any(|m| m.code == "insufficient_markers"));
    }

    #[test]
    fn vcf_workflow_surface_contracts_are_governed_and_explicit() {
        assert!(
            VCF_VALIDATION_CONTRACT
                .rejects
                .contains(&"bad_info_or_format_definitions")
        );
        assert!(
            VCF_REFERENCE_CONTEXT_CONTRACT
                .required_context
                .contains(&"alias_map")
        );
        assert!(
            VCF_FILTER_EVIDENCE_CONTRACT
                .preserved_fields
                .contains(&"damage_filter_policy")
        );
        assert!(
            VCF_NORMALIZATION_CONTRACT
                .declared_behaviors
                .contains(&"multiallelic_decomposition")
        );
        assert!(
            VCF_STATS_REPORT_CONTRACT
                .stable_metric_ids
                .contains(&"annotation_coverage")
        );
    }

    #[test]
    fn vcf_artifact_classes_and_calling_modes_cover_core_surfaces() {
        let postprocess_artifacts = stage_artifact_class_contract(VcfDomainStage::Postprocess);
        assert!(
            postprocess_artifacts
                .artifact_classes
                .contains(&VcfArtifactClass::NormalizedVcf)
        );
        assert!(
            postprocess_artifacts
                .artifact_classes
                .contains(&VcfArtifactClass::AnnotationReport)
        );

        let calling_modes = vcf_calling_mode_contracts();
        assert!(calling_modes.iter().any(|contract| {
            contract.stage == VcfDomainStage::CallDiploid
                && contract.assumptions.contains(&"diploid_gt_fields")
        }));
        assert!(calling_modes.iter().any(|contract| {
            contract.stage == VcfDomainStage::CallGl
                && contract.refusal_rules.contains(&"gl_fields_required")
        }));
    }

    #[test]
    fn vcf_panel_and_population_boundaries_require_explicit_context() {
        let panel = vcf_panel_boundary_contracts();
        assert!(panel.iter().any(|contract| {
            contract.stage == VcfDomainStage::Impute
                && contract.required_context.contains(&"panel_identity")
        }));

        let population = vcf_population_guardrail_contracts();
        assert!(population.iter().any(|contract| {
            contract.stage == VcfDomainStage::Pca
                && contract.required_inputs.contains(&"ld_pruning_policy")
        }));
        assert!(population.iter().any(|contract| {
            contract.stage == VcfDomainStage::Demography
                && contract.report_caveats.contains(&"demography_estimates_are_model_based")
        }));
    }

    #[test]
    fn vcf_damage_and_panel_contracts_are_typed() {
        let damage: DamageAwareGenotypeLogicContract = DAMAGE_AWARE_GENOTYPE_LOGIC.clone();
        assert!(damage.masked_variant_classes.contains(&"ct_transition"));
        assert!(damage.provenance_fields.contains(&"masked_site_count"));

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
    fn panel_governance_rejects_non_hex_checksum_locks() {
        let panel = ReferencePanelGovernance {
            panel_id: "1000g_phase3".to_string(),
            reference_build: "GRCh37".to_string(),
            panel_checksum_sha256: "z".repeat(64),
            index_checksum_sha256: "b".repeat(64),
            license_id: "CC-BY-4.0".to_string(),
            license_constraints: vec!["attribution".to_string()],
            ancestry_tags: vec!["eur".to_string()],
            target_tags: vec!["ancient".to_string()],
        };
        assert!(validate_reference_panel_governance(&panel).is_err());
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
        let bad = VcfInvariantState { tabix_index_present: false, ..ok };
        assert!(validate_vcf_invariants(VcfDomainStage::Stats, &bad).is_err());
    }

    #[test]
    fn species_context_and_species_keyed_invariants_are_enforced() {
        let species = SpeciesContext {
            species_id: "homo_sapiens".to_string(),
            build_id: "GRCh37".to_string(),
            contig_set_digest: "contigs-sha256".to_string(),
            contigs: vec![
                ContigSpec { name: "1".to_string(), length_bp: 249_250_621 },
                ContigSpec { name: "2".to_string(), length_bp: 243_199_373 },
            ],
            sex_system: "xy".to_string(),
            par_policy: "grch37_par".to_string(),
            default_coverage_regime: Some(CoverageRegime::LowCovGl),
        };
        assert!(validate_species_context(&species).is_ok());

        let entry = EntryVcfInvariantState {
            build_id: "GRCh37".to_string(),
            contig_set_digest: "contigs-sha256".to_string(),
            sorted_by_contig_and_pos: true,
            bgzip_compressed: true,
            tabix_index_present: true,
            sample_ids_non_empty_unique: true,
            ploidy_constraints_ok: true,
        };
        assert!(validate_entry_vcf_invariants(&species, &entry).is_ok());

        let panel_map = PanelMapInvariantState {
            species_id: "homo_sapiens".to_string(),
            build_id: "GRCh37".to_string(),
            contig_set_digest: "contigs-sha256".to_string(),
            phased_or_gl_compatible: true,
            format_requirements_ok: true,
            sample_count_ok: true,
            license_allowed: true,
            checksums_match: true,
        };
        assert!(validate_panel_map_invariants(&species, &panel_map).is_ok());
    }

    #[test]
    fn species_context_rejects_invalid_contig_records() {
        let mut species = SpeciesContext {
            species_id: "homo_sapiens".to_string(),
            build_id: "GRCh37".to_string(),
            contig_set_digest: "contigs-sha256".to_string(),
            contigs: vec![ContigSpec { name: "1".to_string(), length_bp: 249_250_621 }],
            sex_system: "xy".to_string(),
            par_policy: "grch37_par".to_string(),
            default_coverage_regime: Some(CoverageRegime::LowCovGl),
        };

        species.contigs.push(ContigSpec { name: "1".to_string(), length_bp: 243_199_373 });
        assert!(validate_species_context(&species).is_err());

        species.contigs = vec![ContigSpec { name: "2".to_string(), length_bp: 0 }];
        assert!(validate_species_context(&species).is_err());

        species.contigs = vec![ContigSpec { name: " ".to_string(), length_bp: 1 }];
        assert!(validate_species_context(&species).is_err());
    }

    #[test]
    fn species_keyed_invariants_reject_invalid_species_context() {
        let species = SpeciesContext {
            species_id: "homo_sapiens".to_string(),
            build_id: "GRCh37".to_string(),
            contig_set_digest: "contigs-sha256".to_string(),
            contigs: vec![ContigSpec { name: "1".to_string(), length_bp: 0 }],
            sex_system: "xy".to_string(),
            par_policy: "grch37_par".to_string(),
            default_coverage_regime: Some(CoverageRegime::LowCovGl),
        };
        let entry = EntryVcfInvariantState {
            build_id: "GRCh37".to_string(),
            contig_set_digest: "contigs-sha256".to_string(),
            sorted_by_contig_and_pos: true,
            bgzip_compressed: true,
            tabix_index_present: true,
            sample_ids_non_empty_unique: true,
            ploidy_constraints_ok: true,
        };
        assert!(validate_entry_vcf_invariants(&species, &entry).is_err());

        let panel_map = PanelMapInvariantState {
            species_id: "homo_sapiens".to_string(),
            build_id: "GRCh37".to_string(),
            contig_set_digest: "contigs-sha256".to_string(),
            phased_or_gl_compatible: true,
            format_requirements_ok: true,
            sample_count_ok: true,
            license_allowed: true,
            checksums_match: true,
        };
        assert!(validate_panel_map_invariants(&species, &panel_map).is_err());
    }

    #[test]
    fn pseudohaploid_to_diploid_imputation_is_refused() {
        let err = match refuse_unsupported_regime_transition(CoverageRegime::Pseudohaploid, true) {
            Ok(()) => panic!("pseudohaploid to diploid imputation transition must be refused"),
            Err(err) => err,
        };
        assert!(
            err.to_string().contains("UnsupportedPseudohaploidToDiploid"),
            "unexpected refusal error: {err}"
        );
    }

    #[test]
    fn imputation_stage_contracts_include_standard_artifacts_and_output_guarantee() {
        let artifact_contract = stage_artifact_contract(VcfDomainStage::Impute);
        assert!(artifact_contract.required_artifacts.contains(&"imputation_accept_decision.json"));
        assert_eq!(OUTPUT_GUARANTEE.final_primary_format, "vcf.gz");
        let requires_bgzip_tabix = OUTPUT_GUARANTEE.requires_bgzip_tabix;
        let deterministic_header_normalization =
            OUTPUT_GUARANTEE.deterministic_header_normalization;
        assert!(requires_bgzip_tabix);
        assert!(deterministic_header_normalization);
    }

    #[test]
    fn filter_postprocess_and_stats_stage_contracts_list_governed_artifacts() {
        let filter = stage_artifact_contract(VcfDomainStage::Filter);
        assert!(filter.required_artifacts.contains(&"filter_explain.json"));

        let postprocess = stage_artifact_contract(VcfDomainStage::Postprocess);
        assert!(postprocess.required_artifacts.contains(&"normalization_contract.json"));

        let stats = stage_artifact_contract(VcfDomainStage::Stats);
        assert!(stats.required_artifacts.contains(&"stats.json"));
        assert!(stats.required_artifacts.contains(&"bcftools_stats.txt"));
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
    fn public_param_catalog_matches_registered_vcf_params() {
        assert_eq!(
            VCF_PARAMS_CATALOG,
            [
                "bijux.vcf.call.params",
                "bijux.vcf.filter.params",
                "bijux.vcf.stats.params",
                "bijux.vcf.call_gl.params",
                "bijux.vcf.call_diploid.params",
                "bijux.vcf.call_pseudohaploid.params",
                "bijux.vcf.damage_filter.params",
                "bijux.vcf.gl_propagation.params",
            ]
        );
    }

    #[test]
    fn public_metrics_catalog_matches_exported_vcf_metrics() {
        assert_eq!(
            VCF_METRICS_CATALOG,
            ["bijux.vcf.call_summary.v1", "bijux.vcf.filter_breakdown.v1", "bijux.vcf.stats.v1",]
        );
    }

    #[test]
    fn stats_metrics_constructor_preserves_sample_identity() {
        let metrics = VcfStatsMetricsV1::empty_for_sample("HG00096");
        assert_eq!(metrics.sample_name, "HG00096");
        assert_eq!(metrics.call_summary.sample_name, "HG00096");
        assert_eq!(metrics.filter_summary.sample_name, "HG00096");
    }

    #[test]
    fn generated_param_registry_matches_config_artifact() {
        let expected_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../configs/ci/params/param_registry_vcf.toml");
        let committed = std::fs::read_to_string(expected_path)
            .unwrap_or_else(|err| panic!("read configs/ci/params/param_registry_vcf.toml: {err}"));
        let generated = param_registry_toml();
        assert_eq!(generated, committed);
        assert!(
            generated.contains("# owner = bijux-dna-domain-vcf"),
            "VCF param registry must name the domain crate as owner"
        );
    }

    #[test]
    fn generated_required_tools_matches_config_artifact() {
        let expected_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../configs/ci/tools/required_tools_vcf.toml");
        let committed = std::fs::read_to_string(expected_path)
            .unwrap_or_else(|err| panic!("read configs/ci/tools/required_tools_vcf.toml: {err}"));
        let generated = required_tools_toml();
        assert_eq!(generated, committed);
        assert!(
            generated.contains("# owner = bijux-dna-domain-vcf"),
            "VCF required tools registry must name the domain crate as owner"
        );
        assert!(
            !generated.contains("# source_commit: 53b050a6d117e40e0122777655e9d8cc428be9ad"),
            "VCF required tools registry must not embed a stale static source commit"
        );
    }
}
