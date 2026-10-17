//! Owner: bijux-dna-domain-fastq
//! Shared FASTQ domain types grouped by responsibility.

mod adapter;
mod artifacts;
mod layout;
mod record;
mod retention;
mod tool;

pub use adapter::{AdapterContributionV1, AdapterTrimmingReportV1};
pub use artifacts::{FastqArtifact, FastqArtifactKind, FastqPE, FastqSE, FastqStats};
pub use layout::{FastqReadLayout, FASTQ_DECLARED_READ_LAYOUTS};
pub(crate) use record::{parse_header_pairing, FastqRecord};
pub use retention::RetentionReportV1;
pub use tool::ToolReferenceV1;

pub use bijux_dna_core::prelude::input_assessment::{FastqLayout, FastqSampleId};

pub type FastqSingleEnd = FastqSE;
pub type FastqPairedEnd = FastqPE;
