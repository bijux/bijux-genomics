//! BAM domain definitions and contracts.
//!
//! Owns: BAM stage semantics, effective params, and canonical metrics schema.
//! Must NOT depend on: bijux-engine or runtime/container execution logic.

mod artifacts;
mod authenticity;
mod contracts;
mod invariants;
pub mod metrics;
pub mod params;
mod pipeline;
mod sample_meta;

pub use artifacts::{BaiPath, BamPath, BedRegions, DictPath, FaiPath, ReferenceFasta};
pub use authenticity::{
    authenticity_score, contamination_cross_check, infer_library_type_from_damage,
    suggest_trim_from_damage,
};
pub use contracts::{contract_for_stage, BamArtifactKind, BamStageContract};
pub use invariants::{evaluate_bam_invariants, BamInvariantEvaluation, BamInvariantThresholds};
pub use metrics::{
    AlignmentCountsV1, AuthenticityEvidenceV1, AuthenticityScoreV1, BamMetricsV1,
    ComplexityMetricsV1, ContaminationMetricsV1, ContaminationReconciliationV1, CoverageMetricsV1,
    CoverageUniformityV1, DamageMetricsV1, EffectiveCoverageV1, FragmentLengthSummaryV1,
    GenotypingMetricsV1, HaplogroupSufficiencyV1, KinshipSufficiencyV1, LibraryTypeInferenceV1,
    MapqSummaryV1, SexConfidenceClass, SexInferenceV1, TrimSuggestionV1,
};
pub use params::{
    AuthenticityEffectiveParams, BamEffectiveParams, BiasMitigationEffectiveParams,
    BqsrEffectiveParams, BqsrMode, ComplexityEffectiveParams, ContaminationEffectiveParams,
    ContaminationScope, CoverageEffectiveParams, DamageEffectiveParams, DuplicateAction,
    FilterEffectiveParams, GenotypingEffectiveParams, HaplogroupEffectiveParams,
    KinshipEffectiveParams, MarkDupEffectiveParams, OpticalDuplicatePolicy, QcPreEffectiveParams,
    RecalibrationSkipCriteria, SexEffectiveParams, UdgModel, UmiPolicy, ValidateEffectiveParams,
};
pub use pipeline::{
    policy_for_library_type, should_skip_recalibration, BamBranchRule, BamDomain, BamLibraryPolicy,
    BAM_BRANCHING_RULES, BAM_CANONICAL_STAGE_ORDER,
};
pub use sample_meta::{
    CaptureType, ExpectedSex, LibraryType, Platform, ReadGroupPolicy, SampleMeta, Strandedness,
};
