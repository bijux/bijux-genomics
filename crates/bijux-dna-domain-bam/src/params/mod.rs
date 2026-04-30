//! Owner: bijux-dna-domain-bam
//! Canonical effective parameters for BAM stages.

mod catalog;
pub mod core;
pub mod downstream;
pub mod pre;

pub use core::{
    BqsrMode, ComplexityEffectiveParams, ContaminationScope, DamageEffectiveParams,
    DuplicateAction, MarkDupEffectiveParams, OpticalDuplicatePolicy, UdgModel, UmiPolicy,
};
pub use downstream::{
    AuthenticityEffectiveParams, BiasMitigationEffectiveParams, BqsrEffectiveParams,
    ContaminationEffectiveParams, CoverageEffectiveParams, EndogenousContentEffectiveParams,
    GenotypingEffectiveParams, HaplogroupEffectiveParams, KinshipEffectiveParams,
    RecalibrationSkipCriteria, SexEffectiveParams,
};
pub use pre::{
    AlignEffectiveParams, FilterEffectiveParams, QcPreEffectiveParams, ReadGroupSpec,
    ValidateEffectiveParams,
};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "stage", rename_all = "snake_case")]
pub enum BamEffectiveParams {
    Align(AlignEffectiveParams),
    Validate(ValidateEffectiveParams),
    QcPre(QcPreEffectiveParams),
    MappingSummary(QcPreEffectiveParams),
    Filter(FilterEffectiveParams),
    MapqFilter(FilterEffectiveParams),
    LengthFilter(FilterEffectiveParams),
    Markdup(MarkDupEffectiveParams),
    DuplicationMetrics(MarkDupEffectiveParams),
    Complexity(ComplexityEffectiveParams),
    Coverage(CoverageEffectiveParams),
    InsertSize(CoverageEffectiveParams),
    GcBias(CoverageEffectiveParams),
    EndogenousContent(EndogenousContentEffectiveParams),
    OverlapCorrection(FilterEffectiveParams),
    Damage(DamageEffectiveParams),
    Authenticity(AuthenticityEffectiveParams),
    Contamination(ContaminationEffectiveParams),
    Sex(SexEffectiveParams),
    BiasMitigation(BiasMitigationEffectiveParams),
    Recalibration(BqsrEffectiveParams),
    Haplogroups(HaplogroupEffectiveParams),
    Genotyping(GenotypingEffectiveParams),
    Kinship(KinshipEffectiveParams),
}
