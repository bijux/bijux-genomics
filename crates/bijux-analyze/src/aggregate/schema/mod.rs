//! Owner: bijux-analyze
//! Metric registry definitions and lookup helpers.
//! Owns registry constants and stage/metric lookup.
//! Must not perform IO or depend on pipeline/report layers.
//! Invariants: registry constants are exhaustive for known stages.

mod defs;
mod fields;
mod lookup;

pub use defs::*;
pub use fields::*;
pub use lookup::*;
