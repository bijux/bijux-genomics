use super::super::adapters::FastqCommandAdapterCoverageRow;
use super::model::{FastqParserCoverageKind, FastqParserCoverageRow};
use super::serialization::benchmark_scope_label;
use anyhow::Result;

/// Evaluate admitted FASTQ readiness inputs.
///
/// # Errors
/// Rejects invalid identities, incompatible evidence or failed publication.
pub fn collect_fastq_parser_coverage_rows(
    input: super::super::inputs::FastqAdapterCoverage,
) -> Result<(usize, usize, Vec<FastqParserCoverageRow>)> {
    let (stage_count, tool_count, rows) = input;
    super::super::inputs::admit_rows(
        stage_count,
        tool_count,
        rows.iter().map(|row| (row.stage_id.as_str(), row.tool_id.as_str())),
    )?;
    let rows = rows
        .into_iter()
        .filter(row_requires_parser)
        .map(render_parser_coverage_row)
        .collect::<Vec<_>>();
    Ok((stage_count, tool_count, rows))
}

pub(super) fn row_requires_parser(row: &FastqCommandAdapterCoverageRow) -> bool {
    row_has_governed_support(&row.support_status)
        && row_has_adapter(&row.adapter_status)
        && row_has_benchmark_scope(&row.corpus_status)
}

pub(super) fn render_parser_coverage_row(
    row: FastqCommandAdapterCoverageRow,
) -> FastqParserCoverageRow {
    let parser_coverage = if row_has_normalized_parser(&row.parser_status) {
        FastqParserCoverageKind::Covered
    } else {
        FastqParserCoverageKind::Missing
    };
    let reason = match parser_coverage {
        FastqParserCoverageKind::Covered => format!(
            "row `{}` / `{}` has governed support, adapter-backed command rendering, {} benchmark coverage, and normalized parser output",
            row.stage_id,
            row.tool_id,
            benchmark_scope_label(&row.corpus_status)
        ),
        FastqParserCoverageKind::Missing => format!(
            "row `{}` / `{}` has governed support, adapter-backed command rendering, and {} benchmark coverage but no normalized parser contract (`{}`)",
            row.stage_id,
            row.tool_id,
            benchmark_scope_label(&row.corpus_status),
            row.parser_status
        ),
    };

    FastqParserCoverageRow {
        tool_id: row.tool_id,
        stage_id: row.stage_id,
        parser_coverage,
        parser_status: row.parser_status,
        support_status: row.support_status,
        adapter_status: row.adapter_status,
        corpus_status: row.corpus_status,
        reason,
    }
}

pub(super) fn row_has_governed_support(value: &str) -> bool {
    matches!(
        value,
        "governed_execution" | "governed_benchmark_cohort" | "observer_specialized_benchmark"
    )
}

pub(super) fn row_has_adapter(value: &str) -> bool {
    matches!(value, "runnable" | "plannable")
}

pub(super) fn row_has_normalized_parser(value: &str) -> bool {
    super::super::policy::has_normalized_parser(value)
}

pub(super) fn row_has_benchmark_scope(value: &str) -> bool {
    value.starts_with("fixture:") || value.starts_with("asset:")
}
