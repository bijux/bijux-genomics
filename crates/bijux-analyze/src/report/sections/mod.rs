// split to keep module size manageable

mod qc;
pub mod schema;

include!("sections/core.rs");
include!("sections/metrics.rs");
include!("sections/findings.rs");
