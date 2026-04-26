//! Owner: bijux-dna-bench
//! Canonical structured benchmark artifact writer.

use std::path::Path;

use anyhow::{Context, Result};

use bijux_dna_bench_model::{BenchmarkSummary, GateDecision};
use bijux_dna_runtime::recording::write_atomic_bytes;

pub(super) fn write_summary_json(path: &Path, summary: &BenchmarkSummary) -> Result<()> {
    let json = serde_json::to_value(summary)?;
    let canonical = bijux_dna_core::contract::canonical::canonicalize_json_value(&json);
    let bytes = serde_json::to_vec_pretty(&canonical)?;
    write_atomic_bytes(path, &bytes).with_context(|| format!("write summary {}", path.display()))
}

pub(super) fn write_decision_json(path: &Path, decision: &GateDecision) -> Result<()> {
    let json = serde_json::to_value(decision)?;
    let canonical = bijux_dna_core::contract::canonical::canonicalize_json_value(&json);
    let bytes = serde_json::to_vec_pretty(&canonical)?;
    write_atomic_bytes(path, &bytes).with_context(|| format!("write decision {}", path.display()))
}

pub(super) fn write_decisions_json(path: &Path, decisions: &[GateDecision]) -> Result<()> {
    let json = serde_json::to_value(decisions)?;
    let canonical = bijux_dna_core::contract::canonical::canonicalize_json_value(&json);
    let bytes = serde_json::to_vec_pretty(&canonical)?;
    write_atomic_bytes(path, &bytes).with_context(|| format!("write decisions {}", path.display()))
}
