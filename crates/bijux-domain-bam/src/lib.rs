//! BAM domain definitions and contracts.
//!
//! Owns: BAM stage semantics, effective params, and canonical metrics schema.
//! Must NOT depend on: bijux-engine or runtime/container execution logic.

mod artifacts;
mod metrics;
mod params;
mod pipeline;
mod sample_meta;

pub use artifacts::{BaiPath, BamPath, BedRegions, DictPath, FaiPath, ReferenceFasta};
pub use metrics::{
    AlignmentCountsV1, BamMetricsV1, ComplexityMetricsV1, ContaminationMetricsV1,
    CoverageMetricsV1, DamageMetricsV1, FragmentLengthSummaryV1, GenotypingMetricsV1,
    MapqSummaryV1, SexInferenceV1,
};
pub use params::{
    BamEffectiveParams, BiasMitigationEffectiveParams, BqsrEffectiveParams, BqsrMode,
    ComplexityEffectiveParams, ContaminationEffectiveParams, ContaminationScope,
    CoverageEffectiveParams, DamageEffectiveParams, DuplicateAction, FilterEffectiveParams,
    GenotypingEffectiveParams, HaplogroupEffectiveParams, KinshipEffectiveParams,
    MarkDupEffectiveParams, OpticalDuplicatePolicy, QcPreEffectiveParams,
    RecalibrationSkipCriteria, SexEffectiveParams, UdgModel, UmiPolicy, ValidateEffectiveParams,
};
pub use pipeline::{BamBranchRule, BamDomain, BAM_BRANCHING_RULES, BAM_CANONICAL_STAGE_ORDER};
pub use sample_meta::{
    CaptureType, ExpectedSex, LibraryType, Platform, ReadGroupPolicy, SampleMeta, Strandedness,
};
