//! Owner: bijux-analyze
//! Metric schema/validation and per-domain metric types.
//! Owns typed metric structs and validation helpers.
//! Must not perform IO or depend on load/report/pipeline layers.
//! Invariants: metric schemas are stable and validated.

mod bench;
mod metrics_base;
mod fastq;

pub use bench::*;
pub use metrics_base::*;
#[allow(unused_imports)]
pub use fastq::*;
