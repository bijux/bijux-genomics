//! Stage planning and contract access.

pub mod artifacts;
pub mod fastq;
mod plan;

pub use plan::{ArtifactRef, StageIO, StagePlan, StagePlanJson};

pub use bijux_domain_fastq as domain_fastq;
pub use bijux_domain_fastq::*;

pub const TOOL_SEQKIT: &str = "seqkit";

pub mod contracts {
    pub use bijux_domain_fastq::contract_for_stage;
    pub use bijux_domain_fastq::FastqStageContract as StageContract;
}
