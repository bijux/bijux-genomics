use super::super::inputs::FastqServingRow as ToolServingMapRow;
use super::model::{
    FastqAdapterCoverageKind, FastqBenchmarkStatus, FastqCommandAdapterCoverageRow,
    FastqReadinessGapKind,
};
use anyhow::Result;

/// Evaluate admitted FASTQ readiness inputs.
///
/// # Errors
/// Rejects invalid identities, incompatible evidence or failed publication.
pub fn collect_fastq_command_adapter_coverage_rows(
    input: super::super::inputs::FastqServingSnapshot,
) -> Result<(usize, usize, Vec<FastqCommandAdapterCoverageRow>)> {
    let super::super::inputs::FastqServingSnapshot { stage_count, tool_count, rows: serving_rows } =
        input;
    super::super::inputs::admit_rows(
        stage_count,
        tool_count,
        serving_rows.iter().map(|row| (row.stage_id.as_str(), row.tool_id.as_str())),
    )?;
    let rows = serving_rows.iter().map(render_coverage_row).collect::<Vec<_>>();
    Ok((stage_count, tool_count, rows))
}

pub(super) fn render_coverage_row(row: &ToolServingMapRow) -> FastqCommandAdapterCoverageRow {
    let support_eligible = row_has_governed_support(row);
    let adapter_covered = row_has_adapter(row);
    let parser_eligible = row_has_normalized_parser(row);
    let benchmark_scope_ready = row_has_benchmark_scope(row);
    let readiness_gap =
        if support_eligible && adapter_covered && parser_eligible && benchmark_scope_ready {
            FastqReadinessGapKind::None
        } else if support_eligible && adapter_covered && parser_eligible {
            FastqReadinessGapKind::Corpus
        } else if support_eligible && adapter_covered {
            FastqReadinessGapKind::Parser
        } else if support_eligible {
            FastqReadinessGapKind::Adapter
        } else {
            FastqReadinessGapKind::Support
        };
    let benchmark_status = if readiness_gap == FastqReadinessGapKind::None {
        FastqBenchmarkStatus::BenchmarkReady
    } else {
        FastqBenchmarkStatus::NotBenchmarkReady
    };
    let adapter_coverage = if adapter_covered {
        FastqAdapterCoverageKind::Covered
    } else {
        FastqAdapterCoverageKind::Missing
    };
    let reason = match readiness_gap {
        FastqReadinessGapKind::None => format!(
            "row `{}` / `{}` is benchmark_ready with governed support, adapter-backed command rendering, normalized parser output, and governed benchmark-scope coverage",
            row.stage_id, row.tool_id
        ),
        FastqReadinessGapKind::Corpus => format!(
            "row `{}` / `{}` has governed support, adapter-backed command rendering, and normalized parser output but still resolves only `{}` benchmark-scope coverage",
            row.stage_id, row.tool_id, row.corpus_status
        ),
        FastqReadinessGapKind::Parser => format!(
            "row `{}` / `{}` has governed support and adapter-backed command rendering but no normalized parser contract (`{}`)",
            row.stage_id, row.tool_id, row.parser_status
        ),
        FastqReadinessGapKind::Adapter => format!(
            "row `{}` / `{}` has governed benchmark support but no runnable or plannable command adapter (`{}`)",
            row.stage_id, row.tool_id, row.adapter_status
        ),
        FastqReadinessGapKind::Support => format!(
            "row `{}` / `{}` is not yet in governed benchmark support for HPC readiness (`{}`)",
            row.stage_id, row.tool_id, row.support_status
        ),
    };

    FastqCommandAdapterCoverageRow {
        tool_id: row.tool_id.clone(),
        stage_id: row.stage_id.clone(),
        benchmark_status,
        adapter_coverage,
        readiness_gap,
        support_status: row.support_status.clone(),
        adapter_status: row.adapter_status.clone(),
        parser_status: row.parser_status.clone(),
        corpus_status: row.corpus_status.clone(),
        reason,
    }
}

pub(super) fn row_has_governed_support(row: &ToolServingMapRow) -> bool {
    matches!(
        row.support_status.as_str(),
        "governed_execution" | "governed_benchmark_cohort" | "observer_specialized_benchmark"
    )
}

pub(super) fn row_has_adapter(row: &ToolServingMapRow) -> bool {
    matches!(row.adapter_status.as_str(), "runnable" | "plannable")
}

pub(super) fn row_has_normalized_parser(row: &ToolServingMapRow) -> bool {
    super::super::policy::has_normalized_parser(&row.parser_status)
}

pub(super) fn row_has_benchmark_scope(row: &ToolServingMapRow) -> bool {
    row.corpus_status.starts_with("fixture:") || row.corpus_status.starts_with("asset:")
}
