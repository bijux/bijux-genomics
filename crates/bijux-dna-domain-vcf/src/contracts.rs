mod panel_governance;
mod stage_delivery;
mod stage_metrics;
mod stage_io;
mod invariants;

pub use invariants::{
    ContigSpec, EntryVcfInvariantState, PanelMapInvariantState, RefusalReason, SpeciesContext,
    VcfInvariantState, refuse_unsupported_regime_transition, validate_entry_vcf_invariants,
    validate_panel_map_invariants, validate_species_context, validate_vcf_invariants,
};
pub use panel_governance::{
    DefaultPanelSelectionPolicy, PanelSelectionContext, PanelSelectionPolicy,
    ReferencePanelGovernance, validate_reference_panel_governance,
};
pub use stage_delivery::{
    DAMAGE_AWARE_GENOTYPE_LOGIC, OUTPUT_GUARANTEE, DamageAwareGenotypeLogicContract,
    StageArtifactContract, StageFailureMode, StageOutputGuarantee, stage_artifact_contract,
    stage_failure_modes,
};
pub use stage_io::{PortCardinality, StageIoContract, StagePortContract, stage_io_contract};
pub use stage_metrics::{StageMetricsContract, stage_metrics_contract};
