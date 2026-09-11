mod acquisition;
mod contracts;
mod declarations;
mod models;
mod observations;
mod paths;
mod publication;
mod rows;
mod service;
pub use declarations::DEFAULT_BAM_AUTHENTICITY_COMPLETE_PATH;
pub use models::{
    AuthenticityCompositeReport, BamAuthenticityCompleteReport, BamAuthenticityCompleteRow,
    LocalAuthenticitySmokeMetrics, LocalAuthenticitySmokeReport,
};
pub use publication::render_bam_authenticity_complete;
mod checks;
mod explanation;
mod projection;
mod report;
pub use models::ConsumedMetricEntry;
