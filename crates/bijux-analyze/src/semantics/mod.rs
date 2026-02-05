//! Owner: bijux-analyze
//! Metric semantics resolution and normalization.
//! Owns: semantics lookup, normalization, missing-data policies.
//! Must not: perform IO.

pub mod metrics;

pub use metrics::*;
