//! Owner: bijux-domain-bam
//! Canonical effective parameters for BAM stages.

mod align;
#[path = "derived/authenticity.rs"]
mod authenticity;
#[path = "derived/bias_mitigation.rs"]
mod bias_mitigation;
mod common;
mod complexity;
mod contamination;
mod coverage;
mod damage;
mod filter;
#[path = "derived/genotyping.rs"]
mod genotyping;
#[path = "derived/haplogroups.rs"]
mod haplogroups;
#[path = "derived/kinship.rs"]
mod kinship;
mod markdup;
#[path = "derived/qc_pre.rs"]
mod qc_pre;
#[path = "derived/recalibration.rs"]
mod recalibration;
#[path = "derived/sex.rs"]
mod sex;
mod validate;

pub use align::{AlignEffectiveParams, ReadGroupSpec};
pub use authenticity::AuthenticityEffectiveParams;
pub use bias_mitigation::BiasMitigationEffectiveParams;
pub use common::{
    BqsrMode, ContaminationScope, DuplicateAction, OpticalDuplicatePolicy, UdgModel, UmiPolicy,
};
pub use complexity::ComplexityEffectiveParams;
pub use contamination::ContaminationEffectiveParams;
pub use coverage::CoverageEffectiveParams;
pub use damage::DamageEffectiveParams;
pub use filter::FilterEffectiveParams;
pub use genotyping::GenotypingEffectiveParams;
pub use haplogroups::HaplogroupEffectiveParams;
pub use kinship::KinshipEffectiveParams;
pub use markdup::MarkDupEffectiveParams;
pub use qc_pre::QcPreEffectiveParams;
pub use recalibration::{BqsrEffectiveParams, RecalibrationSkipCriteria};
pub use sex::SexEffectiveParams;
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
