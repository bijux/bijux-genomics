mod criticality;
mod modes;
mod ordering;
mod transitions;

pub use criticality::{stage_criticality, StageCriticality};
pub use modes::FastqPipelineMode;
pub use ordering::{
    canonical_amplicon_stage_order, canonical_stage_order, default_amplicon_preprocess_stage_order,
    default_shotgun_preprocess_stage_order,
};
pub use transitions::{forbidden_transitions, optional_branches};
