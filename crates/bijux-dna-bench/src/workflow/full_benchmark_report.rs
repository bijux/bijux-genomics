//! Owner: bijux-dna-bench
//! Pure aggregate benchmark report projections over admitted report rows.

use std::collections::{BTreeMap, BTreeSet};

use bijux_dna_bench_model::{
    FullBenchmarkCorpusSectionRow, FullBenchmarkMemoryRow, FullBenchmarkPipelineSection,
    FullBenchmarkPipelineSectionRow, FullBenchmarkReportRow, FullBenchmarkReportRowStatus,
    FullBenchmarkRuntimeRow, FullBenchmarkStageSectionRow, FullBenchmarkToolSectionRow,
};

#[must_use]
pub fn build_full_benchmark_stage_rows(
    rows: &[FullBenchmarkReportRow],
) -> Vec<FullBenchmarkStageSectionRow> {
    let mut grouped = BTreeMap::<(String, String), Vec<&FullBenchmarkReportRow>>::new();
    for row in rows {
        grouped.entry((row.domain.clone(), row.stage_id.clone())).or_default().push(row);
    }

    grouped
        .into_iter()
        .map(|((domain, stage_id), grouped_rows)| FullBenchmarkStageSectionRow {
            domain,
            stage_id,
            row_count: grouped_rows.len(),
            present_row_count: status_count(&grouped_rows, FullBenchmarkReportRowStatus::Present),
            missing_result_row_count: status_count(
                &grouped_rows,
                FullBenchmarkReportRowStatus::MissingResult,
            ),
            unsupported_pair_row_count: status_count(
                &grouped_rows,
                FullBenchmarkReportRowStatus::UnsupportedPair,
            ),
            tool_ids: unique(grouped_rows.iter().map(|row| row.tool_id.clone())),
            corpus_ids: unique(grouped_rows.iter().map(|row| row.corpus_id.clone())),
            pipeline_ids: unique(
                grouped_rows.iter().flat_map(|row| row.pipeline_ids.iter().cloned()),
            ),
            report_sections: unique(grouped_rows.iter().map(|row| row.report_section.clone())),
        })
        .collect()
}

#[must_use]
pub fn build_full_benchmark_tool_rows(
    rows: &[FullBenchmarkReportRow],
) -> Vec<FullBenchmarkToolSectionRow> {
    let mut grouped = BTreeMap::<String, Vec<&FullBenchmarkReportRow>>::new();
    for row in rows {
        grouped.entry(row.tool_id.clone()).or_default().push(row);
    }

    grouped
        .into_iter()
        .map(|(tool_id, grouped_rows)| FullBenchmarkToolSectionRow {
            tool_id,
            row_count: grouped_rows.len(),
            present_row_count: status_count(&grouped_rows, FullBenchmarkReportRowStatus::Present),
            missing_result_row_count: status_count(
                &grouped_rows,
                FullBenchmarkReportRowStatus::MissingResult,
            ),
            unsupported_pair_row_count: status_count(
                &grouped_rows,
                FullBenchmarkReportRowStatus::UnsupportedPair,
            ),
            domains: unique(grouped_rows.iter().map(|row| row.domain.clone())),
            stage_ids: unique(grouped_rows.iter().map(|row| row.stage_id.clone())),
            corpus_ids: unique(grouped_rows.iter().map(|row| row.corpus_id.clone())),
            pipeline_ids: unique(
                grouped_rows.iter().flat_map(|row| row.pipeline_ids.iter().cloned()),
            ),
        })
        .collect()
}

#[must_use]
pub fn build_full_benchmark_corpus_rows(
    rows: &[FullBenchmarkReportRow],
) -> Vec<FullBenchmarkCorpusSectionRow> {
    let mut grouped = BTreeMap::<String, Vec<&FullBenchmarkReportRow>>::new();
    for row in rows {
        grouped.entry(row.corpus_id.clone()).or_default().push(row);
    }

    grouped
        .into_iter()
        .map(|(corpus_id, grouped_rows)| FullBenchmarkCorpusSectionRow {
            corpus_id,
            row_count: grouped_rows.len(),
            present_row_count: status_count(&grouped_rows, FullBenchmarkReportRowStatus::Present),
            missing_result_row_count: status_count(
                &grouped_rows,
                FullBenchmarkReportRowStatus::MissingResult,
            ),
            unsupported_pair_row_count: status_count(
                &grouped_rows,
                FullBenchmarkReportRowStatus::UnsupportedPair,
            ),
            domains: unique(grouped_rows.iter().map(|row| row.domain.clone())),
            stage_ids: unique(grouped_rows.iter().map(|row| row.stage_id.clone())),
            tool_ids: unique(grouped_rows.iter().map(|row| row.tool_id.clone())),
            pipeline_ids: unique(
                grouped_rows.iter().flat_map(|row| row.pipeline_ids.iter().cloned()),
            ),
        })
        .collect()
}

#[must_use]
pub fn build_full_benchmark_pipeline_rows(
    report_rows: &[FullBenchmarkReportRow],
) -> FullBenchmarkPipelineSection {
    let mut grouped = BTreeMap::<String, Vec<&FullBenchmarkReportRow>>::new();
    let mut unmapped_row_count = 0usize;
    for row in report_rows {
        if row.pipeline_ids.is_empty() {
            unmapped_row_count += 1;
            continue;
        }
        for pipeline_id in &row.pipeline_ids {
            grouped.entry(pipeline_id.clone()).or_default().push(row);
        }
    }

    let rows = grouped
        .into_iter()
        .map(|(pipeline_id, grouped_rows)| FullBenchmarkPipelineSectionRow {
            pipeline_id,
            row_count: grouped_rows.len(),
            present_row_count: status_count(&grouped_rows, FullBenchmarkReportRowStatus::Present),
            missing_result_row_count: status_count(
                &grouped_rows,
                FullBenchmarkReportRowStatus::MissingResult,
            ),
            unsupported_pair_row_count: status_count(
                &grouped_rows,
                FullBenchmarkReportRowStatus::UnsupportedPair,
            ),
            domains: unique(grouped_rows.iter().map(|row| row.domain.clone())),
            stage_ids: unique(grouped_rows.iter().map(|row| row.stage_id.clone())),
            tool_ids: unique(grouped_rows.iter().map(|row| row.tool_id.clone())),
        })
        .collect::<Vec<_>>();

    FullBenchmarkPipelineSection { row_count: rows.len(), unmapped_row_count, rows }
}

#[must_use]
pub fn build_full_benchmark_runtime_rows(
    rows: &[FullBenchmarkReportRow],
) -> Vec<FullBenchmarkRuntimeRow> {
    rows.iter()
        .map(|row| FullBenchmarkRuntimeRow {
            report_row_id: row.report_row_id.clone(),
            domain: row.domain.clone(),
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            row_status: row.row_status,
            simulated_elapsed_seconds: row.simulated_elapsed_seconds,
            real_smoke_execution_id: row.real_smoke_execution_id.clone(),
            real_smoke_elapsed_seconds: row.real_smoke_elapsed_seconds,
            runtime_source: runtime_source(row).to_string(),
        })
        .collect()
}

#[must_use]
pub fn build_full_benchmark_memory_rows(
    rows: &[FullBenchmarkReportRow],
) -> Vec<FullBenchmarkMemoryRow> {
    rows.iter()
        .map(|row| FullBenchmarkMemoryRow {
            report_row_id: row.report_row_id.clone(),
            domain: row.domain.clone(),
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            row_status: row.row_status,
            declared_memory_mb: row.declared_memory_mb,
            declared_cpu_threads: row.declared_cpu_threads,
            real_smoke_memory_mb: row.real_smoke_memory_mb,
            real_smoke_cpu_threads: row.real_smoke_cpu_threads,
            memory_source: memory_source(row).to_string(),
        })
        .collect()
}

fn status_count(rows: &[&FullBenchmarkReportRow], status: FullBenchmarkReportRowStatus) -> usize {
    rows.iter().filter(|row| row.row_status == status).count()
}

fn unique(values: impl Iterator<Item = String>) -> Vec<String> {
    values.collect::<BTreeSet<_>>().into_iter().collect()
}

fn runtime_source(row: &FullBenchmarkReportRow) -> &'static str {
    if row.row_status == FullBenchmarkReportRowStatus::UnsupportedPair {
        "not_applicable"
    } else if row.real_smoke_execution_id.is_some() || row.real_smoke_elapsed_seconds.is_some() {
        "fake_run_and_real_smoke"
    } else {
        "fake_run_simulated"
    }
}

fn memory_source(row: &FullBenchmarkReportRow) -> &'static str {
    if row.row_status == FullBenchmarkReportRowStatus::UnsupportedPair {
        "not_applicable"
    } else if row.real_smoke_memory_mb.is_some() || row.real_smoke_cpu_threads.is_some() {
        "real_smoke_manifest"
    } else if row.declared_memory_mb.is_some() || row.declared_cpu_threads.is_some() {
        "declared_stage_tool_resource"
    } else {
        "not_available"
    }
}
