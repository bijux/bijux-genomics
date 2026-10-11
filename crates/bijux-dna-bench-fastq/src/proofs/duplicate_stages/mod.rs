mod binding;
mod contracts;
mod declarations;
mod models;
mod paths;
mod projection;
mod publication;
mod schema;
mod service;
pub use declarations::DEFAULT_FASTQ_DUPLICATE_STAGES_READY_PATH;
pub use models::{FastqDuplicateStagesReadyReport, FastqDuplicateStagesReadyRow};
pub use publication::render_fastq_duplicate_stages_ready;

mod assessment;
mod evaluation_inputs;

mod acquisition;
mod admission;
mod collection;
mod report;
mod source_inputs;

pub use declarations::FASTQ_DUPLICATE_STAGES_READY_SCHEMA_VERSION;
