use super::super::state::ReadinessState;
use crate::adapters::observations::VcfAdapterObservations;
use crate::evidence::commands::collect_vcf_adapter_command_row_map;
use anyhow::{anyhow, bail, Result};
use bijux_dna_bench_vcf_evidence::model::outputs::VcfAdapterOutputCoverageStatus;
use std::collections::BTreeSet;
pub(super) fn complete_pairs(
    observations: &VcfAdapterObservations<'_>,
    state: &mut ReadinessState,
) -> Result<String> {
    let tool_serving_map_report = state
        .tool_serving_map_report
        .as_ref()
        .ok_or_else(|| anyhow!("VCF tool-serving map check did not produce a report"))?;
    let adapter_output_coverage_report = state
        .adapter_output_coverage_report
        .as_ref()
        .ok_or_else(|| anyhow!("VCF adapter output coverage check did not produce a report"))?;
    let rendered_commands_report = state
        .rendered_commands_report
        .as_ref()
        .ok_or_else(|| anyhow!("VCF rendered commands check did not produce a report"))?;

    let benchmark_ready_pairs = tool_serving_map_report
        .rows
        .iter()
        .filter(|row| row.benchmark_status == "benchmark_ready")
        .map(|row| (row.stage_id.clone(), row.tool_id.clone()))
        .collect::<BTreeSet<_>>();
    let adapter_pairs = collect_vcf_adapter_command_row_map(observations)?
        .into_values()
        .filter(|row| row.benchmark_status == "benchmark_ready")
        .map(|row| (row.stage_id, row.tool_id))
        .collect::<BTreeSet<_>>();
    let output_pairs = adapter_output_coverage_report
        .rows
        .iter()
        .filter(|row| {
            row.benchmark_status == "benchmark_ready"
                && row.status == VcfAdapterOutputCoverageStatus::Complete
        })
        .map(|row| (row.stage_id.clone(), row.tool_id.clone()))
        .collect::<BTreeSet<_>>();
    let rendered_pairs = rendered_commands_report
        .rows
        .iter()
        .map(|row| (row.stage_id.clone(), row.tool_id.clone()))
        .collect::<BTreeSet<_>>();

    ensure_pair_sets_match(
        "bcftools adapter completeness",
        &benchmark_ready_pairs,
        &adapter_pairs,
    )?;
    ensure_pair_sets_match("adapter output completeness", &benchmark_ready_pairs, &output_pairs)?;
    ensure_pair_sets_match("rendered command coverage", &benchmark_ready_pairs, &rendered_pairs)?;
    state.counts.adapter_complete = adapter_pairs.len();

    Ok(format!(
                "validated {} benchmark-ready VCF pairs across tool-serving map, executable adapters, output declarations, and rendered commands",
                benchmark_ready_pairs.len()
            ))
}

fn ensure_pair_sets_match(
    surface: &str,
    expected: &BTreeSet<(String, String)>,
    observed: &BTreeSet<(String, String)>,
) -> Result<()> {
    let missing = expected.difference(observed).cloned().collect::<Vec<_>>();
    let extra = observed.difference(expected).cloned().collect::<Vec<_>>();
    if missing.is_empty() && extra.is_empty() {
        return Ok(());
    }

    let format_pairs = |pairs: &[(String, String)]| {
        if pairs.is_empty() {
            "none".to_string()
        } else {
            pairs
                .iter()
                .map(|(stage_id, tool_id)| format!("{stage_id}:{tool_id}"))
                .collect::<Vec<_>>()
                .join(", ")
        }
    };

    bail!(
        "{surface} drifted from the canonical benchmark-ready VCF pair slice: missing [{}], extra [{}]",
        format_pairs(&missing),
        format_pairs(&extra)
    );
}
