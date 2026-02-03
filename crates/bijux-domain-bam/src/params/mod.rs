//! Owner: bijux-domain-bam
//! Canonical effective parameters for BAM stages.

mod align;
mod common;
mod complexity;
mod damage;
mod filter;
mod markdup;
#[path = "derived/qc_pre.rs"]
mod qc_pre;
#[path = "derived/recalibration.rs"]
mod recalibration;
mod validate;

pub mod downstream;

pub use align::{AlignEffectiveParams, ReadGroupSpec};
pub use common::{
    BqsrMode, ContaminationScope, DuplicateAction, OpticalDuplicatePolicy, UdgModel, UmiPolicy,
};
pub use complexity::ComplexityEffectiveParams;
pub use damage::DamageEffectiveParams;
pub use downstream::{
    AuthenticityEffectiveParams, BiasMitigationEffectiveParams, ContaminationEffectiveParams,
    CoverageEffectiveParams, GenotypingEffectiveParams, HaplogroupEffectiveParams,
    KinshipEffectiveParams, SexEffectiveParams,
};
pub use filter::FilterEffectiveParams;
pub use markdup::MarkDupEffectiveParams;
pub use qc_pre::QcPreEffectiveParams;
pub use recalibration::{BqsrEffectiveParams, RecalibrationSkipCriteria};
pub use validate::ValidateEffectiveParams;

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
