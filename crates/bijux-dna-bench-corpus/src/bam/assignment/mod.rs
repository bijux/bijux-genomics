//! Governed BAM corpus assignment, fixture identity and scientific input admission.
mod models;
pub use models::{BamCorpusAssignmentReport, BamCorpusAssignmentRow};
mod context;
pub mod ports;
pub use context::AssignmentContext;
mod admission;
mod compatibility;
mod configuration;
mod declarations;
mod evidence;
mod formatting;
mod paths;
mod rows;
mod service;
mod tsv;
pub use rows::collect_bam_corpus_assignment_rows;
pub use service::render_bam_corpus_assignment;

#[cfg(test)]
mod tests;
