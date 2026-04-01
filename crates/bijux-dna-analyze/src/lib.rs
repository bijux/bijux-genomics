#![allow(ambiguous_glob_reexports)]
//! Analyze pipeline for Bijux runs.
//!
//! Contract: `analyze_run` is the single public entrypoint. It accepts typed input paths and
//! options, enforces the load → validate → compute → report → render pipeline, and returns
//! versioned artifacts. Outputs are stable, deterministic, and only grow in a backward-compatible
//! way for published schemas.

pub mod aggregate;
mod api;
pub mod contract;
pub mod decision;
pub mod export;
pub mod failure;
pub mod load;
pub mod model;
mod pipeline;
pub mod public_api;
pub mod report;
mod semantics;

pub use public_api::*;

/// Analyze a run through the canonical pipeline.
///
/// Delegates to the pipeline implementation (load → validate → normalize → aggregate → compare
/// → rank → explain → render).
///
/// # Errors
/// Returns an error if any pipeline stage fails.
pub fn analyze_run(input: &AnalyzeInput) -> anyhow::Result<AnalyzeOutput> {
    pipeline::analyze_run_pipeline(input)
}
