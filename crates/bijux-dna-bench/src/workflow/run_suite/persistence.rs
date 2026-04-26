use anyhow::Result;

use crate::artifacts::{
    write_decision_json, write_decisions_json, write_observations_jsonl, write_summary_json,
    WriteMode,
};
use bijux_dna_bench_model::policy::GateDecision;
use bijux_dna_bench_model::{BenchError, BenchmarkObservation, BenchmarkSummary};

use super::BenchRunOptions;

pub(super) fn write_suite_artifacts(
    observations: &[BenchmarkObservation],
    summary: &BenchmarkSummary,
    decisions: &[GateDecision],
    options: &BenchRunOptions,
) -> Result<()> {
    let Some(out_dir) = &options.output_dir else {
        return Ok(());
    };

    let mode = if options.force {
        WriteMode::Force
    } else if options.resume {
        WriteMode::Resume
    } else {
        WriteMode::Force
    };
    write_observations_jsonl(&out_dir.join("observations.jsonl"), observations, mode)
        .map_err(|err: anyhow::Error| BenchError::ArtifactWriteError(err.to_string()))?;
    write_summary_json(&out_dir.join("summary.json"), summary)
        .map_err(|err: anyhow::Error| BenchError::ArtifactWriteError(err.to_string()))?;
    if let Some(decision) = decisions.first() {
        write_decision_json(&out_dir.join("decision.json"), decision)
            .map_err(|err: anyhow::Error| BenchError::ArtifactWriteError(err.to_string()))?;
    }
    write_decisions_json(&out_dir.join("decisions.json"), decisions)
        .map_err(|err: anyhow::Error| BenchError::ArtifactWriteError(err.to_string()))?;
    Ok(())
}
