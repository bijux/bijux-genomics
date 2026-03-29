mod catalog;
mod stage;

pub use catalog::{
    validate_downstream_transition, VcfStageTaxonomyRecord, VCF_FORBIDDEN_TRANSITIONS,
    VCF_STAGE_ORDER_DOWNSTREAM, VCF_STAGE_TAXONOMY,
};
pub use stage::{CoverageRegime, DomainSupportStatus, VcfDomainStage, VcfStageKind};
