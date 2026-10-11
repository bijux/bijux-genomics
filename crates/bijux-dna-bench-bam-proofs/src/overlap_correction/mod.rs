mod declarations;
mod models;
mod observations;
mod paths;
mod publication;
mod rows;
mod service;
pub use declarations::DEFAULT_BAM_OVERLAP_CORRECTION_COMPLETE_PATH;
pub use models::{
    BamOverlapCorrectionCompleteReport, BamOverlapCorrectionCompleteRow,
    LocalOverlapCorrectionSmokeMetrics,
};
pub use publication::render_bam_overlap_correction_complete;
mod checks;
mod explanation;
mod projection;
mod report;
