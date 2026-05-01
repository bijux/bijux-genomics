mod invariants;
mod panel_governance;
mod stage_delivery;
mod stage_io;
mod stage_metrics;
mod workflow_surfaces;

pub use invariants::{
    refuse_unsupported_regime_transition, validate_entry_vcf_invariants,
    validate_panel_map_invariants, validate_species_context, validate_vcf_invariants, ContigSpec,
    EntryVcfInvariantState, PanelMapInvariantState, RefusalReason, SpeciesContext,
    VcfInvariantState,
};
pub use panel_governance::{
    validate_reference_panel_governance, DefaultPanelSelectionPolicy, PanelSelectionContext,
    PanelSelectionPolicy, ReferencePanelGovernance,
};
pub use stage_delivery::{
    stage_artifact_contract, stage_failure_modes, DamageAwareGenotypeLogicContract,
    StageArtifactContract, StageFailureMode, StageOutputGuarantee, DAMAGE_AWARE_GENOTYPE_LOGIC,
    OUTPUT_GUARANTEE,
};
pub use stage_io::{stage_io_contract, PortCardinality, StageIoContract, StagePortContract};
pub use stage_metrics::{stage_metrics_contract, StageMetricsContract};
pub use workflow_surfaces::{
    stage_artifact_class_contract, vcf_calling_mode_contracts,
    vcf_cohort_analysis_boundary_contracts, vcf_likelihood_workflow_contracts,
    vcf_panel_boundary_contracts, vcf_phasing_imputation_boundary_contracts,
    vcf_population_guardrail_contracts, VcfArtifactClass, VcfArtifactClassContract,
    VcfCallingModeContract, VcfCohortAnalysisBoundaryContract, VcfCohortValidationContract,
    VcfDamageFilterContract, VcfFilterEvidenceContract, VcfLikelihoodWorkflowContract,
    VcfNormalizationContract, VcfNormalizationPolicyMatrixContract, VcfNormalizationPolicyRow,
    VcfPanelBoundaryContract, VcfPhasingImputationBoundaryContract, VcfPopulationGuardrailContract,
    VcfProductionCorpusCase, VcfProductionCorpusContract, VcfReferenceContextContract,
    VcfReportCoverageContract, VcfScientificDriftContract, VcfStatsReportContract,
    VcfValidationContract, VCF_COHORT_VALIDATION_CONTRACT, VCF_DAMAGE_FILTER_CONTRACT,
    VCF_FILTER_EVIDENCE_CONTRACT, VCF_NORMALIZATION_CONTRACT,
    VCF_NORMALIZATION_POLICY_MATRIX_CONTRACT, VCF_PRODUCTION_CORPUS_CONTRACT,
    VCF_REFERENCE_CONTEXT_CONTRACT, VCF_REPORT_COVERAGE_CONTRACT, VCF_SCIENTIFIC_DRIFT_CONTRACT,
    VCF_STATS_REPORT_CONTRACT, VCF_VALIDATION_CONTRACT,
};
