mod contracts;
mod declarations;
mod models;
mod observations;
mod paths;
mod publication;
mod rows;
mod service;
pub use declarations::DEFAULT_BAM_ENDOGENOUS_CONTENT_COMPLETE_PATH;
pub use models::{
    BamEndogenousContentCompleteReport, BamEndogenousContentCompleteRow,
    LocalEndogenousContentSmokeMetrics,
};
pub use publication::render_bam_endogenous_content_complete;
mod checks;
mod explanation;

mod projection;
mod report;
#[cfg(test)]
mod tests;
