//! BAM stage planning and contract access.

pub mod bam;
mod plan;

pub use bijux_core::{ArtifactRef, StageIO, StagePlanV1};
pub use plan::StagePlanJson;

pub use bijux_domain_bam as domain_bam;
pub use bijux_domain_bam::*;

pub mod contracts {
    pub use bijux_domain_bam::BamDomain;
}
