//! BAM domain definitions and contracts.
//!
//! Owns: BAM stage semantics, effective params, and canonical metrics schema.
//! Must NOT depend on: bijux-engine or runtime/container execution logic.

mod authenticity;
mod bam_stage_registry;
mod invariants;
pub mod metrics;
pub mod params;
mod pipeline;
mod types;

pub use authenticity::{
    authenticity_score, contamination_cross_check, infer_library_type_from_damage,
    suggest_trim_from_damage,
};
pub use bam_stage_registry::{
    contract_for_stage, required_audit_artifacts, stage_spec, stage_specs, ArtifactPolicy,
    AuditArtifact, BamArtifactKind, BamStage, BamStageContract, BamStageSpec,
};
pub use invariants::{evaluate_bam_invariants, BamInvariantEvaluation, BamInvariantThresholds};
pub use metrics::{
    AlignmentCountsV1, AuthenticityEvidenceV1, AuthenticityScoreV1, BamInvariantStatusV1,
    BamMetricsBundleV1, BamMetricsV1, BamStageVerdictV1, ComplexityMetricsV1,
    ContaminationMetricsV1, ContaminationReconciliationV1, ContaminationSufficiencyV1,
    CoverageMetricsV1, CoverageSufficiencyV1, CoverageUniformityV1, DamageMetricsV1,
    EffectiveCoverageV1, FragmentLengthSummaryV1, GenotypingMetricsV1, HaplogroupSufficiencyV1,
    IdxstatsContigV1, IdxstatsSummaryV1, KinshipSufficiencyV1, LibraryTypeInferenceV1,
    MapqSummaryV1, SexConfidenceClass, SexInferenceV1, SexSufficiencyV1, TrimSuggestionV1,
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
    should_skip_recalibration, BamDomain, BAM_CANONICAL_STAGE_ORDER,
};
pub use types::{
    BaiPath, BamPath, BedRegions, CaptureType, ContigId, DictPath, ExpectedSex, FaiPath,
    LibraryType, Platform, ReadGroupPolicy, ReferenceFasta, SampleMeta, Strandedness,
};
