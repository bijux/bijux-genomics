//! BAM domain definitions and contracts.
//!
//! Owns: BAM stage semantics, effective params, and canonical metrics schema.
//! Must NOT depend on: bijux-engine or runtime/container execution logic.

mod artifacts;
mod audit;
mod authenticity;
mod contracts;
mod invariants;
mod library;
pub mod metrics;
pub mod params;
pub mod parsers;
mod pipeline;
mod sample_meta;
mod stage;

pub use artifacts::{BaiPath, BamPath, BedRegions, DictPath, FaiPath, ReferenceFasta};
pub use audit::{required_audit_artifacts, AuditArtifact};
pub use authenticity::{
    authenticity_score, contamination_cross_check, infer_library_type_from_damage,
    suggest_trim_from_damage,
};
pub use contracts::{contract_for_stage, BamArtifactKind, BamStageContract};
pub use invariants::{evaluate_bam_invariants, BamInvariantEvaluation, BamInvariantThresholds};
pub use library::{DamageExpectation, LibraryTreatment};
pub use metrics::{
    AlignmentCountsV1, AuthenticityEvidenceV1, AuthenticityScoreV1, BamInvariantStatusV1,
    BamMetricsBundleV1, BamMetricsV1, BamStageVerdictV1, ComplexityMetricsV1,
    ContaminationMetricsV1, ContaminationReconciliationV1, ContaminationSufficiencyV1,
    CoverageMetricsV1, CoverageSufficiencyV1, CoverageUniformityV1, DamageMetricsV1,
    EffectiveCoverageV1, FragmentLengthSummaryV1, GenotypingMetricsV1, HaplogroupSufficiencyV1,
    KinshipSufficiencyV1, LibraryTypeInferenceV1, MapqSummaryV1, SexConfidenceClass,
    SexInferenceV1, SexSufficiencyV1, TrimSuggestionV1,
};
pub use params::{
    AuthenticityEffectiveParams, BamEffectiveParams, BiasMitigationEffectiveParams,
    BqsrEffectiveParams, BqsrMode, ComplexityEffectiveParams, ContaminationEffectiveParams,
    ContaminationScope, CoverageEffectiveParams, DamageEffectiveParams, DuplicateAction,
    FilterEffectiveParams, GenotypingEffectiveParams, HaplogroupEffectiveParams,
    KinshipEffectiveParams, MarkDupEffectiveParams, OpticalDuplicatePolicy, QcPreEffectiveParams,
    RecalibrationSkipCriteria, SexEffectiveParams, UdgModel, UmiPolicy, ValidateEffectiveParams,
};
pub use parsers::{
    parse_contamination_json, parse_damageprofiler_json, parse_mosdepth_summary,
    parse_preseq_estimates, parse_pydamage_json, parse_samtools_flagstat, parse_samtools_stats,
    parse_sex_json,
};
pub use pipeline::{
    policy_for_library_type, should_skip_recalibration, BamBranchRule, BamDomain, BamLibraryPolicy,
    BAM_BRANCHING_RULES, BAM_CANONICAL_STAGE_ORDER,
};
pub use sample_meta::{
    CaptureType, ExpectedSex, LibraryType, Platform, ReadGroupPolicy, SampleMeta, Strandedness,
};
pub use stage::BamStage;
