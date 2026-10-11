#[cfg(feature = "bam_downstream")]
mod contracts;
mod declarations;
mod models;
#[cfg(feature = "bam_downstream")]
mod observations;
#[cfg(feature = "bam_downstream")]
mod paths;
mod publication;
#[cfg(feature = "bam_downstream")]
mod row_input;
#[cfg(feature = "bam_downstream")]
mod rows;
#[cfg(feature = "bam_downstream")]
mod service;
pub use declarations::DEFAULT_BAM_GENOTYPING_COMPLETE_PATH;
pub use models::{BamGenotypingCompleteReport, BamGenotypingCompleteRow};
pub use publication::render_bam_genotyping_complete;
#[cfg(not(feature = "bam_downstream"))]
pub use publication::unavailable;
#[cfg(feature = "bam_downstream")]
mod checks;
#[cfg(feature = "bam_downstream")]
mod explanation;
#[cfg(feature = "bam_downstream")]
mod parser;
#[cfg(feature = "bam_downstream")]
mod projection;
#[cfg(feature = "bam_downstream")]
mod report;
#[cfg(all(test, feature = "bam_downstream"))]
mod tests;
