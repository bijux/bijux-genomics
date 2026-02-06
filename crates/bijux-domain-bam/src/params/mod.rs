//! Owner: bijux-domain-bam
//! Canonical effective parameters for BAM stages.

pub mod core;
pub mod downstream;
pub mod pre;

pub use core::{
    BqsrMode, ComplexityEffectiveParams, ContaminationScope, DamageEffectiveParams,
    DuplicateAction, MarkDupEffectiveParams, OpticalDuplicatePolicy, UdgModel, UmiPolicy,
};
pub use downstream::{
    AuthenticityEffectiveParams, BiasMitigationEffectiveParams, ContaminationEffectiveParams,
    CoverageEffectiveParams, GenotypingEffectiveParams, HaplogroupEffectiveParams,
    KinshipEffectiveParams, RecalibrationSkipCriteria, SexEffectiveParams,
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
    Filter(FilterEffectiveParams),
    Markdup(MarkDupEffectiveParams),
    Complexity(ComplexityEffectiveParams),
    Coverage(CoverageEffectiveParams),
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
