//! Call bam support benchmark boundary.
pub(crate) mod admission;
pub(crate) mod execution;
pub(crate) mod model;
pub(crate) mod publication;
#[cfg(test)]
#[path = "../../tests/contracts/call_bam_support.rs"]
mod tests;
