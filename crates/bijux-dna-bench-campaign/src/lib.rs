//! Corpus benchmark orchestration with explicit input, execution and publication boundaries.
pub mod artifacts;
mod cache;
mod campaign;
pub mod catalog;
mod execution;
mod planning;
pub mod ports;
mod qc;
pub mod request;
pub use campaign::run_corpus_campaign;

mod admission;
