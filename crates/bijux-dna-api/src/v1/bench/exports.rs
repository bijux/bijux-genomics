//! Benchmarking helpers for v1.
//!
//! Stability: v1 (stable).

use std::path::Path;

use anyhow::{Context, Result};

pub use bijux_dna_analyze::compare::compare_runs_with_baseline;
pub use bijux_dna_analyze::{build_rankings, compare_runs, print_bench_schema, RankInput};

pub use crate::internal::public_bridge::handlers::bam::BamBenchOutcome;
pub use crate::internal::public_bridge::handlers::bam::{bench_bam_pipeline, bench_bam_stage};
pub use crate::surface::request_contracts::{BamRunArgs, BenchBamPipelineArgs, BenchBamStageArgs};
pub use bijux_dna_core::contract::objective_spec;
pub use bijux_dna_core::contract::{Objective, ObjectiveSpec, ObjectiveWeights};
pub use bijux_dna_planner_bam::stage_api::{bam_stage_completeness, BamStage};
pub use bijux_dna_planner_fastq::stage_api as fastq_banks;
pub use bijux_dna_planner_fastq::stage_api::args as fastq_args;
pub use bijux_dna_planner_fastq::stage_api::banks as fastq_bank_ops;
pub use bijux_dna_planner_fastq::stage_api::*;

/// Stability: v1 (stable).
/// Alias for `BenchOutcome<M: StageMetricSchema>` from the fastq handlers.
pub type BenchOutcome<M> = crate::internal::public_bridge::handlers::fastq::BenchOutcome<M>;
pub use crate::internal::public_bridge::handlers::fastq::*;

/// Render the governed FASTQ normalized metrics schema.
#[must_use]
pub fn render_fastq_normalized_metrics_schema() -> serde_json::Value {
    crate::internal::fastq::stages::preprocess::render_governed_fastq_normalized_metrics_schema()
}

/// Write the governed FASTQ normalized metrics schema to disk.
pub fn write_fastq_normalized_metrics_schema(path: &Path) -> Result<()> {
    let schema = render_fastq_normalized_metrics_schema();
    bijux_dna_infra::atomic_write_json(path, &schema)
        .with_context(|| format!("write {}", path.display()))
}

/// Render the governed BAM normalized metrics schema.
#[must_use]
pub fn render_bam_normalized_metrics_schema() -> serde_json::Value {
    crate::internal::handlers::cross::render_governed_bam_normalized_metrics_schema()
}

/// Write the governed BAM normalized metrics schema to disk.
pub fn write_bam_normalized_metrics_schema(path: &Path) -> Result<()> {
    let schema = render_bam_normalized_metrics_schema();
    bijux_dna_infra::atomic_write_json(path, &schema)
        .with_context(|| format!("write {}", path.display()))
}
