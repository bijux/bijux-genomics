//! Owner: bijux-dna-analyze
//! Metric schema/validation and per-domain metric types.
//! Owns typed metric structs and validation helpers.
//! Must not perform IO or depend on load/report/pipeline layers.
//! Invariants: metric schemas are stable and validated.

mod bench;
mod fastq;
mod metrics_base;

pub use bench::*;
#[allow(unused_imports)]
pub use fastq::*;
pub use metrics_base::*;
