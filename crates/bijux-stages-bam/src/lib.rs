//! BAM stage planning and contract access.

pub mod bam;
pub mod bam_tools_registry;
mod plan;
pub mod stages_adna;
#[cfg(feature = "bam_downstream")]
pub mod stages_downstream;
pub mod stages_post;
pub mod stages_pre;
pub mod stages_support;
pub mod tools;

pub use bijux_core::{ArtifactRef, StageIO, StagePlanV1};
pub use plan::StagePlanJson;

pub use bijux_domain_bam as domain_bam;
