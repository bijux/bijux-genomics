// split to keep module size manageable

pub mod queries;

pub use queries::bench::*;
pub use queries::bench_results_fastq::*;
pub use queries::core::*;
pub use queries::core_other::*;
pub use queries::core_trim::*;
pub use queries::quality::*;
pub use queries::reports::*;
