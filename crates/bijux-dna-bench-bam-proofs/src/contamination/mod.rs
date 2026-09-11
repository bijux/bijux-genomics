mod contracts;
mod declarations;
mod models;
mod observations;
mod paths;
mod publication;
mod rows;
mod service;
pub use declarations::DEFAULT_BAM_CONTAMINATION_COMPLETE_PATH;
pub use models::{BamContaminationCompleteReport, BamContaminationCompleteRow};
pub use publication::render_bam_contamination_complete;
mod checks;
mod explanation;

mod projection;
mod report;
#[cfg(test)]
mod tests;
