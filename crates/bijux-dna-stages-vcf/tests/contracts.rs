mod contracts {
    #![allow(
        clippy::expect_used,
        clippy::redundant_closure_for_method_calls,
        clippy::too_many_lines,
        clippy::unreadable_literal,
        unused_imports
    )]
    use std::path::Path;

    use bijux_dna_domain_vcf::contracts::{ContigSpec, SpeciesContext};
    use bijux_dna_domain_vcf::VcfDomainStage;
    use bijux_dna_stages_vcf::engine::{run_vcf_pipeline, VcfPipelineRequest};
    use bijux_dna_stages_vcf::invariants::{run_vcf_preflight, InputRegime, InvariantConfig};
    use bijux_dna_stages_vcf::metrics::{
        parse_vcf_call_summary, parse_vcf_filter_breakdown, parse_vcf_stats,
    };
    use bijux_dna_stages_vcf::pipeline::{
        assert_bgzip_tabix_artifacts, run_admixture_stage, run_call_diploid_stage,
        run_call_gl_from_bam_stage, run_call_gl_stage, run_call_pseudohaploid_stage,
        run_chunked_regions,
        run_damage_filter_stage, run_demography_stage, run_filter_stage_real,
        run_gl_propagation_stage, run_ibd_stage, run_imputation_orchestration_stage,
        run_impute_stage, run_pca_stage, run_phasing_stage, run_population_structure_stage,
        run_postprocess_stage, run_prepare_reference_panel_stage, run_qc_stage, run_roh_stage,
        run_stats_stage_real, AdmixtureStageParams, ChunkFailurePolicy, ChunkingPlanParams,
        DamageFilterStageParams, DamageUdgRegime, DemographyStageParams, GlPropagationStageParams,
        IbdStageParams, ImputationAcceptMode, ImputeBackend, ImputeStageParams, PcaStageParams,
        PhasingBackend, PhasingStageParams, PopulationStructureStageParams, PostprocessStageParams,
        PrepareReferencePanelParams, QcStageParams, RohStageParams,
    };
    use bijux_dna_stages_vcf::stage_specs::{supported_vcf_stages, vcf_stage_catalog};
    use bijux_dna_stages_vcf::wrappers::verify_tool_wrapper;

    include!("contracts_suite/core_pipeline_tests.rs");
    include!("contracts_suite/qc_and_invariants_tests.rs");
    include!("contracts_suite/panel_and_phasing_tests.rs");
    include!("contracts_suite/imputation_tests.rs");
    include!("contracts_suite/postprocess_population_and_ibd_tests.rs");
}
