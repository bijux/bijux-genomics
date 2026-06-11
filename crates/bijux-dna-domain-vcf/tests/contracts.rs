#[path = "contracts/parsers.rs"]
mod parsers;

mod contracts {
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    use bijux_dna_domain_vcf::{
        build_vcf_scientific_drift_report,
        contracts::{
            comparable_metric_stage_ids, refuse_unsupported_regime_transition,
            stage_artifact_class_contract, stage_artifact_contract, stage_comparable_metric_specs,
            stage_failure_modes, stage_io_contract, stage_metrics_contract,
            validate_entry_vcf_invariants, validate_panel_map_invariants,
            validate_reference_panel_governance, validate_species_context, validate_vcf_invariants,
            vcf_calling_mode_contracts, vcf_cohort_analysis_boundary_contracts,
            vcf_likelihood_workflow_contracts, vcf_panel_boundary_contracts,
            vcf_parser_fixture_inventory, vcf_phasing_imputation_boundary_contracts,
            vcf_population_guardrail_contracts, ContigSpec, DamageAwareGenotypeLogicContract,
            DefaultPanelSelectionPolicy, EntryVcfInvariantState, PanelMapInvariantState,
            PanelSelectionContext, PanelSelectionPolicy, ReferencePanelGovernance, SpeciesContext,
            VcfArtifactClass, VcfComparableMetricDirection, VcfInvariantState,
            DAMAGE_AWARE_GENOTYPE_LOGIC, OUTPUT_GUARANTEE, VCF_COHORT_VALIDATION_CONTRACT,
            VCF_DAMAGE_FILTER_CONTRACT, VCF_FILTER_EVIDENCE_CONTRACT, VCF_NORMALIZATION_CONTRACT,
            VCF_NORMALIZATION_POLICY_MATRIX_CONTRACT, VCF_PRODUCTION_CORPUS_CONTRACT,
            VCF_REFERENCE_CONTEXT_CONTRACT, VCF_REPORT_COVERAGE_CONTRACT,
            VCF_SCIENTIFIC_DRIFT_CONTRACT, VCF_STATS_REPORT_CONTRACT, VCF_VALIDATION_CONTRACT,
        },
        coverage::domain_coverage_report,
        param_registry_toml, required_tools_toml, required_vcf_bench_corpus_scenarios,
        validate_downstream_transition, vcf_bench_corpus_manifest, CoverageRegime,
        VcfBenchCorpusId, VcfDomainStage, VcfScientificDriftSnapshotV1, VcfStage,
        VcfStatsMetricsV1, VCF_METRICS_CATALOG, VCF_PARAMS_CATALOG, VCF_STAGE_ORDER_DOWNSTREAM,
    };

    fn assert_snapshot_json(name: &str, value: &serde_json::Value) {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("snapshots")
            .join(format!("bijux-dna-domain-vcf__contracts__{name}.json"));
        let actual = serde_json::to_string_pretty(value)
            .unwrap_or_else(|err| panic!("serialize snapshot json: {err}"));
        if std::env::var("UPDATE_SNAPSHOTS").ok().as_deref() == Some("1") {
            let parent = path
                .parent()
                .unwrap_or_else(|| panic!("snapshot parent missing for {}", path.display()));
            std::fs::create_dir_all(parent)
                .unwrap_or_else(|err| panic!("create snapshot dir {}: {err}", parent.display()));
            std::fs::write(&path, format!("{actual}\n"))
                .unwrap_or_else(|err| panic!("write snapshot {}: {err}", path.display()));
            return;
        }
        let expected = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("read snapshot {}: {err}", path.display()));
        assert_eq!(actual, expected.trim_end(), "snapshot mismatch for {}", path.display());
    }

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
                "vcf.imputation_metrics",
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
        assert!(validate_downstream_transition(
            VcfDomainStage::ImputationMetrics,
            VcfDomainStage::Call,
        )
        .is_err());
        assert_eq!(
            VCF_STAGE_ORDER_DOWNSTREAM.first().map(|s| s.as_str()),
            Some("vcf.prepare_reference_panel")
        );
    }

    #[test]
    fn vcf_stage_contracts_expose_io_metrics_and_failure_modes() {
        let Some(io) = stage_io_contract(VcfDomainStage::ImputationMetrics) else {
            panic!("missing stage IO contract for imputation");
        };
        assert!(io.required_inputs.contains(&"vcf"));
        assert!(io.required_indices.contains(&"vcf.tbi"));
        assert_eq!(io.required_outputs, vec!["imputation_metrics_json"]);

        let metrics = stage_metrics_contract(VcfDomainStage::ImputationMetrics);
        assert_eq!(metrics.metrics_schema_id, "bijux.vcf.imputation_metrics.v1");
        assert!(metrics.required_metrics.contains(&"mean_info_score"));
        assert!(metrics.required_metrics.contains(&"masked_truth_sites"));

        let failure_modes = stage_failure_modes(VcfDomainStage::Phasing);
        assert!(failure_modes.iter().any(|m| m.code == "insufficient_markers"));
    }

    #[test]
    fn authored_imputation_metrics_catalog_matches_governed_contract_ids() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../");
        let stage_raw = std::fs::read_to_string(
            repo_root.join("domain/vcf/stages/imputation_metrics.yaml"),
        )
        .unwrap_or_else(|err| panic!("read imputation_metrics stage yaml: {err}"));
        let artifacts_raw = std::fs::read_to_string(repo_root.join("domain/vcf/artifacts.yaml"))
            .unwrap_or_else(|err| panic!("read VCF artifact vocabulary: {err}"));
        let metrics_raw = std::fs::read_to_string(repo_root.join("domain/vcf/metrics.yaml"))
            .unwrap_or_else(|err| panic!("read VCF metric vocabulary: {err}"));

        assert!(stage_raw.contains("- name: \"imputation_metrics_json\""));
        assert!(stage_raw.contains("required_outputs: [\"imputation_metrics_json\"]"));
        assert!(!stage_raw.contains("imputation_out"));
        assert!(!stage_raw.contains("imputation_status"));

        for metric_id in [
            "status",
            "mean_info_score",
            "r2_available",
            "low_confidence_sites",
            "masked_truth_sites",
            "missing_quality_fields",
        ] {
            assert!(
                stage_raw.contains(&format!("  - name: \"{metric_id}\"")),
                "authored stage yaml is missing `{metric_id}`"
            );
            assert!(
                metrics_raw.contains(&format!("- id: {metric_id}")),
                "VCF metric vocabulary is missing `{metric_id}`"
            );
        }

        assert!(artifacts_raw.contains("- id: imputation_metrics_json"));
        assert!(!artifacts_raw.contains("imputation_out"));
        assert!(!metrics_raw.contains("- id: imputation_status"));
    }

    #[test]
    fn authored_qc_pca_admixture_and_stats_catalogs_match_governed_contract_ids() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../");
        let qc_raw = std::fs::read_to_string(repo_root.join("domain/vcf/stages/qc.yaml"))
            .unwrap_or_else(|err| panic!("read qc stage yaml: {err}"));
        let pca_raw = std::fs::read_to_string(repo_root.join("domain/vcf/stages/pca.yaml"))
            .unwrap_or_else(|err| panic!("read pca stage yaml: {err}"));
        let admixture_raw =
            std::fs::read_to_string(repo_root.join("domain/vcf/stages/admixture.yaml"))
                .unwrap_or_else(|err| panic!("read admixture stage yaml: {err}"));
        let stats_raw = std::fs::read_to_string(repo_root.join("domain/vcf/stages/stats.yaml"))
            .unwrap_or_else(|err| panic!("read stats stage yaml: {err}"));
        let artifacts_raw = std::fs::read_to_string(repo_root.join("domain/vcf/artifacts.yaml"))
            .unwrap_or_else(|err| panic!("read VCF artifact vocabulary: {err}"));
        let metrics_raw = std::fs::read_to_string(repo_root.join("domain/vcf/metrics.yaml"))
            .unwrap_or_else(|err| panic!("read VCF metric vocabulary: {err}"));

        assert!(qc_raw.contains("status: \"supported\""));
        assert!(qc_raw.contains("- name: \"qc_report\""));
        assert!(qc_raw.contains("required_outputs: [\"qc_report\"]"));
        assert!(!qc_raw.contains("- name: \"qc_out\""));
        assert!(!qc_raw.contains("required_outputs: [\"qc_out\"]"));

        for metric_id in [
            "variant_count",
            "sample_missingness",
            "variant_missingness",
            "maf_summary",
            "heterozygosity",
            "hwe_summary",
            "excluded_samples",
            "excluded_variants",
            "sample_missingness_exclusion_threshold",
            "variant_missingness_exclusion_threshold",
        ] {
            assert!(
                qc_raw.contains(&format!("  - name: \"{metric_id}\"")),
                "authored qc stage yaml is missing `{metric_id}`"
            );
            assert!(
                metrics_raw.contains(&format!("- id: {metric_id}")),
                "VCF metric vocabulary is missing `{metric_id}`"
            );
        }

        assert!(pca_raw.contains("- name: \"pca_report\""));
        assert!(pca_raw.contains("required_outputs: [\"pca_report\"]"));
        assert!(!pca_raw.contains("- name: \"pca_out\""));
        assert!(!pca_raw.contains("required_outputs: [\"pca_out\"]"));
        for metric_id in [
            "sample_count",
            "variant_count",
            "excluded_samples",
            "unexpected_samples",
            "eigenvalues",
        ] {
            assert!(
                pca_raw.contains(&format!("  - name: \"{metric_id}\"")),
                "authored pca stage yaml is missing `{metric_id}`"
            );
            assert!(
                metrics_raw.contains(&format!("- id: {metric_id}")),
                "VCF metric vocabulary is missing `{metric_id}`"
            );
        }

        assert!(admixture_raw.contains("status: \"supported\""));
        assert!(admixture_raw.contains("- name: \"admixture_report\""));
        assert!(admixture_raw.contains("required_outputs: [\"admixture_report\"]"));
        assert!(!admixture_raw.contains("- name: \"admixture_out\""));
        assert!(!admixture_raw.contains("required_outputs: [\"admixture_out\"]"));
        for metric_id in ["selected_k", "sample_count", "population_count", "status"] {
            assert!(
                admixture_raw.contains(&format!("  - name: \"{metric_id}\"")),
                "authored admixture stage yaml is missing `{metric_id}`"
            );
            assert!(
                metrics_raw.contains(&format!("- id: {metric_id}")),
                "VCF metric vocabulary is missing `{metric_id}`"
            );
        }

        for metric_id in [
            "variant_count",
            "snp_count",
            "indel_count",
            "transition_count",
            "transversion_count",
            "ti_tv",
            "sample_count",
        ] {
            assert!(
                stats_raw.contains(&format!("  - name: \"{metric_id}\"")),
                "authored stats stage yaml is missing `{metric_id}`"
            );
            assert!(
                metrics_raw.contains(&format!("- id: {metric_id}")),
                "VCF metric vocabulary is missing `{metric_id}`"
            );
        }

        for artifact_id in ["qc_report", "pca_report", "admixture_report"] {
            assert!(
                artifacts_raw.contains(&format!("- id: {artifact_id}")),
                "VCF artifact vocabulary is missing `{artifact_id}`"
            );
        }
        assert!(!artifacts_raw.contains("qc_out"));
        assert!(!artifacts_raw.contains("pca_out"));
        assert!(!artifacts_raw.contains("admixture_out"));
    }

    #[test]
    fn stats_and_qc_metric_contracts_cover_governed_summary_ids() {
        let stats = stage_metrics_contract(VcfDomainStage::Stats);
        assert!(stats.required_metrics.contains(&"sample_count"));
        assert!(stats.required_metrics.contains(&"transition_count"));

        let qc = stage_metrics_contract(VcfDomainStage::Qc);
        assert!(qc.required_metrics.contains(&"sample_missingness"));
        assert!(qc.required_metrics.contains(&"variant_missingness"));
        assert!(qc.required_metrics.contains(&"hwe_summary"));
    }

    #[test]
    fn stage_io_contracts_use_report_specific_output_ids() {
        let qc = stage_io_contract(VcfDomainStage::Qc)
            .unwrap_or_else(|| panic!("missing stage IO contract for qc"));
        assert_eq!(qc.required_outputs, vec!["qc_report"]);

        let pca = stage_io_contract(VcfDomainStage::Pca)
            .unwrap_or_else(|| panic!("missing stage IO contract for pca"));
        assert_eq!(pca.required_outputs, vec!["pca_report"]);

        let admixture = stage_io_contract(VcfDomainStage::Admixture)
            .unwrap_or_else(|| panic!("missing stage IO contract for admixture"));
        assert_eq!(admixture.required_outputs, vec!["admixture_report"]);

        let stats = stage_io_contract(VcfDomainStage::Stats)
            .unwrap_or_else(|| panic!("missing stage IO contract for stats"));
        assert_eq!(stats.required_outputs, vec!["stats_json"]);
    }

    #[test]
    fn stage_metrics_contract_matches_governed_stage_schema_ids() {
        let pca = stage_metrics_contract(VcfDomainStage::Pca);
        assert_eq!(pca.metrics_schema_id, "bijux.vcf.pca.v1");
        assert!(pca.required_metrics.contains(&"eigenvalues"));

        let admixture = stage_metrics_contract(VcfDomainStage::Admixture);
        assert_eq!(admixture.metrics_schema_id, "bijux.vcf.admixture.v1");
        assert!(admixture.required_metrics.contains(&"selected_k"));

        let impute = stage_metrics_contract(VcfDomainStage::Impute);
        assert_eq!(impute.metrics_schema_id, "bijux.vcf.impute.v1");
        assert!(impute.required_metrics.contains(&"masked_truth_match_count"));
        assert!(impute.required_metrics.contains(&"sample_ids"));

        let postprocess = stage_metrics_contract(VcfDomainStage::Postprocess);
        assert_eq!(postprocess.metrics_schema_id, "bijux.vcf.postprocess.v1");
        assert!(postprocess.required_metrics.contains(&"left_align_applied"));

        let panel = stage_metrics_contract(VcfDomainStage::PrepareReferencePanel);
        assert_eq!(panel.metrics_schema_id, "bijux.vcf.prepare_reference_panel.v1");
        assert!(panel.required_metrics.contains(&"duplicate_sites_removed"));

        let gl_propagation = stage_metrics_contract(VcfDomainStage::GlPropagation);
        assert_eq!(gl_propagation.metrics_schema_id, "bijux.vcf.gl_propagation.v1");
        assert!(gl_propagation.required_metrics.contains(&"input_likelihood_fields"));

        let roh = stage_metrics_contract(VcfDomainStage::Roh);
        assert_eq!(roh.metrics_schema_id, "bijux.vcf.roh.v1");
        assert!(roh.required_metrics.contains(&"segment_count"));
        assert!(roh.required_metrics.contains(&"per_sample_summary"));

        let ibd = stage_metrics_contract(VcfDomainStage::Ibd);
        assert_eq!(ibd.metrics_schema_id, "bijux.vcf.ibd.v1");
        assert!(ibd.required_metrics.contains(&"rows"));
        assert!(ibd.required_metrics.contains(&"insufficient_reason"));

        let demography = stage_metrics_contract(VcfDomainStage::Demography);
        assert_eq!(demography.metrics_schema_id, "bijux.vcf.demography.v1");
        assert!(demography.required_metrics.contains(&"time_bins"));
        assert!(demography.required_metrics.contains(&"insufficient_data_probe"));
    }

    #[test]
    fn vcf_comparable_metric_contracts_cover_retained_multi_tool_stage_slice() {
        let stage_ids = comparable_metric_stage_ids()
            .into_iter()
            .map(|stage| stage.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            stage_ids,
            vec![
                "vcf.admixture",
                "vcf.call_gl",
                "vcf.call_pseudohaploid",
                "vcf.damage_filter",
                "vcf.gl_propagation",
                "vcf.ibd",
                "vcf.imputation_metrics",
                "vcf.impute",
                "vcf.pca",
                "vcf.phasing",
                "vcf.population_structure",
                "vcf.qc",
            ]
        );

        let call_gl = stage_comparable_metric_specs(VcfDomainStage::CallGl);
        assert!(call_gl.iter().any(|metric| {
            metric.metric_id == "sites_with_likelihoods"
                && metric.unit == "sites"
                && metric.direction == VcfComparableMetricDirection::HigherIsBetter
                && metric.required
        }));
        assert!(call_gl.iter().any(|metric| {
            metric.metric_id == "missing_likelihoods"
                && metric.direction == VcfComparableMetricDirection::LowerIsBetter
        }));

        let qc = stage_comparable_metric_specs(VcfDomainStage::Qc);
        assert!(qc.iter().any(|metric| {
            metric.metric_id == "concordance"
                && metric.unit == "fraction"
                && metric.direction == VcfComparableMetricDirection::HigherIsBetter
        }));

        let phasing = stage_comparable_metric_specs(VcfDomainStage::Phasing);
        assert!(phasing.iter().any(|metric| {
            metric.metric_id == "phase_block_n50"
                && metric.unit == "bases"
                && metric.direction == VcfComparableMetricDirection::HigherIsBetter
        }));

        let impute = stage_comparable_metric_specs(VcfDomainStage::Impute);
        assert!(impute.iter().any(|metric| {
            metric.metric_id == "masked_truth_match_count"
                && metric.direction == VcfComparableMetricDirection::HigherIsBetter
        }));
    }

    #[test]
    fn vcf_parser_fixture_inventory_covers_governed_tool_stage_rows() {
        let rows = vcf_parser_fixture_inventory();
        assert_eq!(rows.len(), 39);

        let unique_rows = rows
            .iter()
            .map(|row| format!("{}:{}", row.tool_id, row.stage.as_str()))
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(unique_rows.len(), rows.len());

        assert!(rows.iter().any(|row| {
            row.tool_id == "bcftools"
                && row.stage == VcfDomainStage::Call
                && row.parser_id == "parse_bcftools_call_metrics"
                && row.fixture_path
                    == "benchmarks/tests/fixtures/bench/parsers/vcf/bcftools/vcf.call"
        }));
        assert!(rows.iter().any(|row| {
            row.tool_id == "bcftools"
                && row.stage == VcfDomainStage::Stats
                && row.parser_id == "parse_bcftools_stats_metrics"
        }));
        assert!(rows.iter().any(|row| {
            row.tool_id == "shapeit5"
                && row.stage == VcfDomainStage::Phasing
                && row.parser_id == "parse_shapeit5_phasing_metrics"
        }));
        assert!(rows.iter().any(|row| {
            row.tool_id == "beagle"
                && row.stage == VcfDomainStage::ImputationMetrics
                && row.parser_id == "parse_beagle_imputation_metrics"
        }));
        assert!(rows.iter().all(|row| !row.fixture_path.is_empty()));
        assert!(rows.iter().all(|row| stage_metrics_contract(row.stage)
            .metrics_schema_id
            .starts_with("bijux.vcf.")));
    }

    #[test]
    fn vcf_workflow_surface_contracts_are_governed_and_explicit() {
        assert!(VCF_VALIDATION_CONTRACT.rejects.contains(&"bad_info_or_format_definitions"));
        assert!(VCF_REFERENCE_CONTEXT_CONTRACT.required_context.contains(&"alias_map"));
        assert!(VCF_FILTER_EVIDENCE_CONTRACT.preserved_fields.contains(&"damage_filter_policy"));
        assert!(VCF_NORMALIZATION_CONTRACT
            .declared_behaviors
            .contains(&"multiallelic_decomposition"));
        assert!(VCF_NORMALIZATION_POLICY_MATRIX_CONTRACT.policy_rows.iter().any(|row| {
            row.policy_id == "lowcov_gl_production"
                && row.split_multiallelic
                && row.duplicate_handling == "retain_likelihood_safe_records_only"
        }));
        assert!(VCF_COHORT_VALIDATION_CONTRACT
            .checked_before_analysis
            .contains(&"sex_and_ploidy_assumptions"));
        assert!(VCF_DAMAGE_FILTER_CONTRACT.declared_actions.contains(&"annotate_damage_risk"));
        assert!(VCF_STATS_REPORT_CONTRACT.stable_metric_ids.contains(&"annotation_coverage"));
        assert!(VCF_REPORT_COVERAGE_CONTRACT
            .per_sample_sections
            .contains(&"missingness_by_sample"));
        assert!(VCF_PRODUCTION_CORPUS_CONTRACT.covered_cases.iter().any(|case| {
            case.case_id == "panel_mismatch" && case.expectation.contains("refuse")
        }));
        assert!(VCF_SCIENTIFIC_DRIFT_CONTRACT
            .tracked_change_surfaces
            .contains(&"imputation_backend"));
    }

    #[test]
    fn vcf_artifact_classes_and_calling_modes_cover_core_surfaces() {
        let postprocess_artifacts = stage_artifact_class_contract(VcfDomainStage::Postprocess);
        assert!(postprocess_artifacts.artifact_classes.contains(&VcfArtifactClass::NormalizedVcf));
        assert!(postprocess_artifacts
            .artifact_classes
            .contains(&VcfArtifactClass::AnnotationReport));

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

        let phasing = vcf_phasing_imputation_boundary_contracts();
        assert!(phasing.iter().any(|contract| {
            contract.stage == VcfDomainStage::Phasing
                && contract.required_outputs.contains(&"switch_error_proxy")
        }));
        assert!(phasing.iter().any(|contract| {
            contract.stage == VcfDomainStage::Impute && contract.accepted_tools.contains(&"glimpse")
        }));

        let likelihood = vcf_likelihood_workflow_contracts();
        assert!(likelihood.iter().any(|contract| {
            contract.stage == VcfDomainStage::CallGl && contract.accepted_tools.contains(&"angsd")
        }));
        assert!(likelihood.iter().any(|contract| {
            contract.stage == VcfDomainStage::CallPseudohaploid
                && contract
                    .output_caveats
                    .contains(&"pseudo_haploid_calls_are_not_diploid_genotypes")
        }));

        let cohort = vcf_cohort_analysis_boundary_contracts();
        assert!(cohort.iter().any(|contract| {
            contract.stage == VcfDomainStage::Ibd
                && contract.minimum_cohort_requirement == "at_least_two_samples"
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
    fn production_vcf_corpus_manifest_covers_required_scenarios() {
        let manifest = vcf_bench_corpus_manifest(VcfBenchCorpusId::ProductionRegression);
        let required = required_vcf_bench_corpus_scenarios();
        assert_eq!(manifest.schema_version, "bijux.vcf.bench_corpus_manifest.v1");
        assert!(manifest.covered_cases.contains(&"panel_mismatch".to_string()));
        for scenario in required {
            assert!(
                manifest.scenarios_covered.contains(&scenario),
                "missing governed VCF corpus scenario {scenario:?}"
            );
        }
        assert!(manifest.datasets.iter().any(|dataset| {
            dataset.dataset_id == "SYNTHETIC_LOWCOV_GL"
                && dataset.scientific_scope == "likelihood_workflow_regression"
        }));
    }

    #[test]
    fn vcf_scientific_drift_report_snapshot_stays_stable() {
        let baseline = VcfScientificDriftSnapshotV1 {
            label: "baseline".to_string(),
            stage_id: "vcf.postprocess".to_string(),
            tool_id: "bcftools".to_string(),
            backend_version: Some("1.20".to_string()),
            defaults_fingerprint: Some("defaults-a".to_string()),
            normalization_policy_id: Some("diploid_production".to_string()),
            filter_policy_id: Some("strict_filter_a".to_string()),
            metrics: BTreeMap::from([
                ("variants_total".to_string(), 1200.0),
                ("annotation_coverage".to_string(), 0.92),
                ("missingness_post".to_string(), 0.03),
            ]),
            artifacts: BTreeMap::from([
                ("normalized_vcf".to_string(), "sha256:a".to_string()),
                ("stats_json".to_string(), "sha256:b".to_string()),
            ]),
            caveats: vec!["baseline generated from promoted defaults".to_string()],
        };
        let candidate = VcfScientificDriftSnapshotV1 {
            label: "candidate".to_string(),
            stage_id: "vcf.postprocess".to_string(),
            tool_id: "bcftools".to_string(),
            backend_version: Some("1.21".to_string()),
            defaults_fingerprint: Some("defaults-b".to_string()),
            normalization_policy_id: Some("lowcov_gl_production".to_string()),
            filter_policy_id: Some("strict_filter_b".to_string()),
            metrics: BTreeMap::from([
                ("variants_total".to_string(), 1175.0),
                ("annotation_coverage".to_string(), 0.89),
                ("missingness_post".to_string(), 0.05),
            ]),
            artifacts: BTreeMap::from([
                ("normalized_vcf".to_string(), "sha256:c".to_string()),
                ("stats_json".to_string(), "sha256:d".to_string()),
            ]),
            caveats: vec!["candidate uses stronger normalization and filtering".to_string()],
        };
        let report = build_vcf_scientific_drift_report(&baseline, &candidate);
        assert_snapshot_json(
            "vcf_scientific_drift_report",
            &serde_json::to_value(report)
                .unwrap_or_else(|err| panic!("serialize scientific drift report: {err}")),
        );
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
            .any(|row| row.stage_id == "vcf.imputation_metrics" && row.domain_only));
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

    #[test]
    fn generated_vcf_registry_keeps_bcftools_planned_stage_bindings() {
        let registry_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../configs/ci/registry/tool_registry_vcf.toml");
        let committed = std::fs::read_to_string(&registry_path)
            .unwrap_or_else(|err| panic!("read {}: {err}", registry_path.display()));
        assert!(
            committed.contains("\"vcf.postprocess\""),
            "bcftools registry row must retain vcf.postprocess"
        );
        assert!(
            committed.contains("\"vcf.prepare_reference_panel\""),
            "bcftools registry row must retain vcf.prepare_reference_panel"
        );
    }
}
