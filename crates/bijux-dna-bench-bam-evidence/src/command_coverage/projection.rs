use super::models::{
    BamAdapterCoverageKind, BamBenchmarkStatus, BamCommandAdapterCoverageRow, BamReadinessGapKind,
};
use crate::observations::ServingBinding;
pub(super) fn render_coverage_row(row: &ServingBinding) -> BamCommandAdapterCoverageRow {
    let support_eligible = row.support_status == "supported";
    let adapter_covered = row_has_adapter(row);
    let parser_eligible = row.parser_status == "parser_fixture_validated";
    let fixture_backed = row_has_fixture_corpus(row);
    let readiness_gap = if support_eligible && adapter_covered && parser_eligible && fixture_backed
    {
        BamReadinessGapKind::None
    } else if support_eligible && adapter_covered && parser_eligible {
        BamReadinessGapKind::Corpus
    } else if support_eligible && adapter_covered {
        BamReadinessGapKind::Parser
    } else if support_eligible {
        BamReadinessGapKind::Adapter
    } else {
        BamReadinessGapKind::Support
    };
    let benchmark_status = if readiness_gap == BamReadinessGapKind::None {
        BamBenchmarkStatus::BenchmarkReady
    } else {
        BamBenchmarkStatus::NotBenchmarkReady
    };
    let adapter_coverage = if adapter_covered {
        BamAdapterCoverageKind::Covered
    } else {
        BamAdapterCoverageKind::Missing
    };
    let reason = match readiness_gap {
        BamReadinessGapKind::None => format!(
            "row `{}` / `{}` is benchmark_ready with governed support, adapter-backed command rendering, parser-fixture-validated output, and fixture-backed corpus coverage",
            row.stage_id, row.tool_id
        ),
        BamReadinessGapKind::Corpus => format!(
            "row `{}` / `{}` has governed support, adapter-backed command rendering, and parser-fixture-validated output but still resolves only `{}` corpus coverage",
            row.stage_id, row.tool_id, row.corpus_status
        ),
        BamReadinessGapKind::Parser => format!(
            "row `{}` / `{}` has governed support and adapter-backed command rendering but no parser-fixture-validated BAM result normalizer (`{}`)",
            row.stage_id, row.tool_id, row.parser_status
        ),
        BamReadinessGapKind::Adapter => format!(
            "row `{}` / `{}` has governed benchmark support but no runnable or plannable command adapter (`{}`)",
            row.stage_id, row.tool_id, row.adapter_status
        ),
        BamReadinessGapKind::Support => format!(
            "row `{}` / `{}` is not yet in governed BAM benchmark support for HPC readiness (`{}`)",
            row.stage_id, row.tool_id, row.support_status
        ),
    };

    BamCommandAdapterCoverageRow {
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
pub(super) fn row_has_adapter(row: &ServingBinding) -> bool {
    matches!(row.adapter_status.as_str(), "runnable" | "plannable")
}
pub(super) fn row_has_fixture_corpus(row: &ServingBinding) -> bool {
    row.corpus_status.starts_with("fixture:")
}
