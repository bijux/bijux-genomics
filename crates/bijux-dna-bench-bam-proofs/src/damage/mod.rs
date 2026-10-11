mod acquisition;
mod contracts;
mod declarations;
mod models;
mod observations;
mod paths;
mod publication;
mod rows;
mod service;
pub use declarations::DEFAULT_BAM_DAMAGE_COMPLETE_PATH;
pub use models::{
    BamDamageCompleteReport, BamDamageCompleteRow, LocalDamageSmokeMetrics, ParsedDamageToolOutput,
};
pub use publication::render_bam_damage_complete;
mod checks;
mod explanation;
mod projection;
mod report;
