//! BAM benchmark declaration evidence, report placement and binding consistency.
//! Native scientific execution and tool-output normalization remain external authorities.
pub mod command_coverage;
pub mod consistency;
mod context;
pub mod observations;
pub mod output_contract;
pub mod parser_coverage;
pub mod ports;
mod publication;
pub mod report_map;
pub use context::{EvidenceContext, EvidencePaths};
