//! Owner: bijux-dna-bench
//! Pure aggregate benchmark report projections over admitted report rows.

use std::collections::{BTreeMap, BTreeSet};

use bijux_dna_bench_model::{
    AllDomainFailureClassificationReport, BamComparableMetricsReport, BenchError,
    FastqComparableMetricsReport, FullBenchmarkComparableMetricRow, FullBenchmarkCorpusSectionRow,
    FullBenchmarkFailureRow, FullBenchmarkFailuresSection, FullBenchmarkMemoryRow,
    FullBenchmarkMissingResultRow, FullBenchmarkPipelineSection, FullBenchmarkPipelineSectionRow,
    FullBenchmarkReport, FullBenchmarkReportRow, FullBenchmarkReportRowStatus,
    FullBenchmarkResultCollectorReport, FullBenchmarkResultSurfaceKind, FullBenchmarkRuntimeRow,
    FullBenchmarkStageSectionRow, FullBenchmarkToolSectionRow, FullBenchmarkUnsupportedPairRow,
    ScientificAcceptanceThresholdRow, VcfComparableMetricsReport,
};

/// Builds the domain-spanning comparable-metric section from admitted readiness reports.
///
/// # Errors
/// Returns [`BenchError::InvalidObservation`] when acceptance rows are duplicated or do not
/// cover every comparable metric emitted by the domain reports.
pub fn build_full_benchmark_comparable_metric_rows(
    fastq_report: &FastqComparableMetricsReport,
    bam_report: &BamComparableMetricsReport,
    vcf_report: &VcfComparableMetricsReport,
    acceptance_rows: &[ScientificAcceptanceThresholdRow],
) -> Result<Vec<FullBenchmarkComparableMetricRow>, BenchError> {
    let acceptance_by_metric = acceptance_rows_by_metric(acceptance_rows)?;
    let mut rows = Vec::new();

    for row in &fastq_report.rows {
        for metric in &row.shared_metric_fields {
            let acceptance = acceptance_row(&acceptance_by_metric, "fastq", &row.stage_id, metric)?;
            rows.push(FullBenchmarkComparableMetricRow {
                domain: "fastq".to_string(),
                stage_id: row.stage_id.clone(),
                metric_id: metric.clone(),
                metric_name: metric.clone(),
                unit: acceptance.unit.clone(),
                direction: acceptance.direction.as_str().to_string(),
                tolerance_kind: acceptance.tolerance_kind.as_str().to_string(),
                tolerance_value: acceptance.tolerance_value,
                pass_rule: acceptance.pass_rule.as_str().to_string(),
                insufficiency_behavior: acceptance.insufficiency_behavior.as_str().to_string(),
                required: acceptance.required,
                tool_ids: row.tool_ids.clone(),
                contract_status: row.comparison_contract_status.as_str().to_string(),
                reason: row.reason.clone(),
            });
        }
    }

    for row in &bam_report.rows {
        for metric in &row.shared_metric_fields {
            let acceptance = acceptance_row(&acceptance_by_metric, "bam", &row.stage_id, metric)?;
            rows.push(FullBenchmarkComparableMetricRow {
                domain: "bam".to_string(),
                stage_id: row.stage_id.clone(),
                metric_id: metric.clone(),
                metric_name: metric.clone(),
                unit: acceptance.unit.clone(),
                direction: acceptance.direction.as_str().to_string(),
                tolerance_kind: acceptance.tolerance_kind.as_str().to_string(),
                tolerance_value: acceptance.tolerance_value,
                pass_rule: acceptance.pass_rule.as_str().to_string(),
                insufficiency_behavior: acceptance.insufficiency_behavior.as_str().to_string(),
                required: acceptance.required,
                tool_ids: row.tool_ids.clone(),
                contract_status: row.comparison_contract_status.as_str().to_string(),
                reason: row.reason.clone(),
            });
        }
    }

    for row in &vcf_report.rows {
        let acceptance =
            acceptance_row(&acceptance_by_metric, "vcf", &row.stage_id, &row.metric_id)?;
        rows.push(FullBenchmarkComparableMetricRow {
            domain: "vcf".to_string(),
            stage_id: row.stage_id.clone(),
            metric_id: row.metric_id.clone(),
            metric_name: row.metric_name.clone(),
            unit: acceptance.unit.clone(),
            direction: acceptance.direction.as_str().to_string(),
            tolerance_kind: acceptance.tolerance_kind.as_str().to_string(),
            tolerance_value: acceptance.tolerance_value,
            pass_rule: acceptance.pass_rule.as_str().to_string(),
            insufficiency_behavior: acceptance.insufficiency_behavior.as_str().to_string(),
            required: acceptance.required,
            tool_ids: row.tools_covered.clone(),
            contract_status: "declared".to_string(),
            reason: format!(
                "governed VCF comparable metric published for `{}` across {} retained tools",
                row.stage_id,
                row.tools_covered.len()
            ),
        });
    }

    rows.sort_by(|left, right| {
        left.domain
            .cmp(&right.domain)
            .then_with(|| left.stage_id.cmp(&right.stage_id))
            .then_with(|| left.metric_id.cmp(&right.metric_id))
    });
    Ok(rows)
}

fn acceptance_rows_by_metric(
    rows: &[ScientificAcceptanceThresholdRow],
) -> Result<BTreeMap<(String, String, String), ScientificAcceptanceThresholdRow>, BenchError> {
    let mut by_metric = BTreeMap::new();
    for row in rows {
        let key = (row.domain.clone(), row.stage_id.clone(), row.metric_id.clone());
        if by_metric.insert(key.clone(), row.clone()).is_some() {
            return Err(BenchError::InvalidObservation {
                reason: format!(
                    "full benchmark report found duplicate scientific acceptance threshold row for `{}:{}/{}`",
                    key.0, key.1, key.2
                ),
            });
        }
    }
    Ok(by_metric)
}

fn acceptance_row<'a>(
    by_metric: &'a BTreeMap<(String, String, String), ScientificAcceptanceThresholdRow>,
    domain: &str,
    stage_id: &str,
    metric_id: &str,
) -> Result<&'a ScientificAcceptanceThresholdRow, BenchError> {
    by_metric
        .get(&(domain.to_string(), stage_id.to_string(), metric_id.to_string()))
        .ok_or_else(|| BenchError::InvalidObservation {
            reason: format!(
                "full benchmark report is missing scientific acceptance semantics for `{domain}` / `{stage_id}` / `{metric_id}`"
            ),
        })
}

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

pub fn build_full_benchmark_failures_section(
    collector_report: &FullBenchmarkResultCollectorReport,
    classification_report: &AllDomainFailureClassificationReport,
) -> Result<FullBenchmarkFailuresSection, BenchError> {
    let mut simulated_failure_domain_counts = BTreeMap::<String, usize>::new();
    let mut rows = Vec::new();
    for row in collector_report
        .rows
        .iter()
        .filter(|row| row.surface_kind == FullBenchmarkResultSurfaceKind::FakeFailure)
    {
        *simulated_failure_domain_counts.entry(row.domain.clone()).or_default() += 1;
        rows.push(FullBenchmarkFailureRow {
            record_id: row.record_id.clone(),
            domain: row.domain.clone(),
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            result_id: row.result_id.clone(),
            evidence_path: row.evidence_path.clone(),
            detail: row.detail.clone(),
        });
    }
    if rows.len() != collector_report.fake_failure_row_count {
        return Err(BenchError::InvalidObservation {
            reason:
                "full benchmark report failure section drifted from governed fake-failure coverage"
                    .to_string(),
        });
    }

    Ok(FullBenchmarkFailuresSection {
        simulated_failure_row_count: rows.len(),
        failure_class_row_count: classification_report.rows.len(),
        simulated_failure_domain_counts,
        rows,
        classification_rows: classification_report.rows.clone(),
    })
}

pub fn build_full_benchmark_missing_result_rows(
    rows: &[FullBenchmarkReportRow],
) -> Result<Vec<FullBenchmarkMissingResultRow>, BenchError> {
    rows.iter()
        .filter(|row| row.row_status == FullBenchmarkReportRowStatus::MissingResult)
        .map(|row| {
            let result_id =
                row.result_id.clone().ok_or_else(|| BenchError::InvalidObservation {
                    reason: format!(
                        "full benchmark missing-result row `{}` has no result identity",
                        row.report_row_id
                    ),
                })?;
            Ok(FullBenchmarkMissingResultRow {
                result_id,
                domain: row.domain.clone(),
                stage_id: row.stage_id.clone(),
                tool_id: row.tool_id.clone(),
                corpus_id: row.corpus_id.clone(),
                asset_profile_id: row.asset_profile_id.clone(),
                report_section: row.report_section.clone(),
                audit_manifest_path: row.manifest_path.clone(),
                evidence_path: row.evidence_path.clone(),
                reason: row.detail.clone(),
            })
        })
        .collect()
}

#[must_use]
pub fn build_full_benchmark_unsupported_pair_rows(
    rows: &[FullBenchmarkReportRow],
) -> Vec<FullBenchmarkUnsupportedPairRow> {
    rows.iter()
        .filter(|row| row.row_status == FullBenchmarkReportRowStatus::UnsupportedPair)
        .map(|row| FullBenchmarkUnsupportedPairRow {
            domain: row.domain.clone(),
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            evidence_path: row.evidence_path.clone(),
            detail: row.detail.clone(),
        })
        .collect()
}

#[must_use]
pub fn render_full_benchmark_report_markdown(report: &FullBenchmarkReport) -> String {
    let mut rendered = String::from("# FASTQ + BAM + VCF Benchmark Report\n\n");
    rendered.push_str(&format!(
        "- Report rows: {}\n- Expected-result rows: {}\n- Explicit unsupported rows: {}\n- Present rows: {}\n- Missing-result rows: {}\n- Unsupported-pair rows: {}\n- Failure rows: {}\n- Comparable metric rows: {}\n\n",
        report.row_count,
        report.expected_result_row_count,
        report.explicit_unsupported_row_count,
        report.present_row_count,
        report.missing_result_row_count,
        report.unsupported_pair_row_count,
        report.failure_row_count,
        report.comparable_metric_row_count
    ));

    rendered.push_str("## Stage-Centric\n\n");
    rendered.push_str(
        "| Domain | Stage | Rows | Present | Missing | Unsupported | Tools | Pipelines |\n",
    );
    rendered.push_str("| --- | --- | ---: | ---: | ---: | ---: | --- | --- |\n");
    for row in &report.stage_centric {
        rendered.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} |\n",
            sanitize_markdown_cell(&row.domain),
            sanitize_markdown_cell(&row.stage_id),
            row.row_count,
            row.present_row_count,
            row.missing_result_row_count,
            row.unsupported_pair_row_count,
            sanitize_markdown_cell(&row.tool_ids.join(", ")),
            sanitize_markdown_cell(&row.pipeline_ids.join(", "))
        ));
    }
    rendered.push('\n');

    rendered.push_str("## Tool-Centric\n\n");
    rendered.push_str("| Tool | Rows | Present | Missing | Unsupported | Domains | Stages |\n");
    rendered.push_str("| --- | ---: | ---: | ---: | ---: | --- | --- |\n");
    for row in &report.tool_centric {
        rendered.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} |\n",
            sanitize_markdown_cell(&row.tool_id),
            row.row_count,
            row.present_row_count,
            row.missing_result_row_count,
            row.unsupported_pair_row_count,
            sanitize_markdown_cell(&row.domains.join(", ")),
            sanitize_markdown_cell(&row.stage_ids.join(", "))
        ));
    }
    rendered.push('\n');

    rendered.push_str("## Corpus-Centric\n\n");
    rendered.push_str("| Corpus | Rows | Present | Missing | Unsupported | Domains | Stages |\n");
    rendered.push_str("| --- | ---: | ---: | ---: | ---: | --- | --- |\n");
    for row in &report.corpus_centric {
        rendered.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} |\n",
            sanitize_markdown_cell(&row.corpus_id),
            row.row_count,
            row.present_row_count,
            row.missing_result_row_count,
            row.unsupported_pair_row_count,
            sanitize_markdown_cell(&row.domains.join(", ")),
            sanitize_markdown_cell(&row.stage_ids.join(", "))
        ));
    }
    rendered.push('\n');

    rendered.push_str("## Pipeline-Centric\n\n");
    rendered.push_str(&format!(
        "- Pipeline rows: {}\n- Unmapped report rows: {}\n\n",
        report.pipeline_centric.row_count, report.pipeline_centric.unmapped_row_count
    ));
    rendered.push_str("| Pipeline | Rows | Present | Missing | Unsupported | Domains | Stages |\n");
    rendered.push_str("| --- | ---: | ---: | ---: | ---: | --- | --- |\n");
    for row in &report.pipeline_centric.rows {
        rendered.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} |\n",
            sanitize_markdown_cell(&row.pipeline_id),
            row.row_count,
            row.present_row_count,
            row.missing_result_row_count,
            row.unsupported_pair_row_count,
            sanitize_markdown_cell(&row.domains.join(", ")),
            sanitize_markdown_cell(&row.stage_ids.join(", "))
        ));
    }
    rendered.push('\n');

    rendered.push_str("## Runtime\n\n");
    rendered.push_str("| Report Row | Domain | Stage | Tool | Status | Simulated Elapsed Seconds | Real-Smoke Elapsed Seconds | Source |\n");
    rendered.push_str("| --- | --- | --- | --- | --- | ---: | ---: | --- |\n");
    for row in &report.runtime {
        rendered.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} |\n",
            sanitize_markdown_cell(&row.report_row_id),
            sanitize_markdown_cell(&row.domain),
            sanitize_markdown_cell(&row.stage_id),
            sanitize_markdown_cell(&row.tool_id),
            report_row_status_label(row.row_status),
            format_optional_f64(row.simulated_elapsed_seconds),
            format_optional_f64(row.real_smoke_elapsed_seconds),
            sanitize_markdown_cell(&row.runtime_source)
        ));
    }
    rendered.push('\n');

    rendered.push_str("## Memory\n\n");
    rendered.push_str("| Report Row | Domain | Stage | Tool | Status | Declared Memory MB | Declared CPU Threads | Real-Smoke Memory MB | Real-Smoke CPU Threads | Source |\n");
    rendered.push_str("| --- | --- | --- | --- | --- | ---: | ---: | ---: | ---: | --- |\n");
    for row in &report.memory {
        rendered.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
            sanitize_markdown_cell(&row.report_row_id),
            sanitize_markdown_cell(&row.domain),
            sanitize_markdown_cell(&row.stage_id),
            sanitize_markdown_cell(&row.tool_id),
            report_row_status_label(row.row_status),
            format_optional_f64(row.declared_memory_mb),
            format_optional_u32(row.declared_cpu_threads),
            format_optional_f64(row.real_smoke_memory_mb),
            format_optional_u32(row.real_smoke_cpu_threads),
            sanitize_markdown_cell(&row.memory_source)
        ));
    }
    rendered.push('\n');

    rendered.push_str("## Failures\n\n");
    rendered.push_str(&format!(
        "- Simulated failure rows: {}\n- Failure classification rows: {}\n\n",
        report.failures.simulated_failure_row_count, report.failures.failure_class_row_count
    ));
    rendered
        .push_str("| Failure Class | Domain | Stage | Tool | Source Surface | Status | Detail |\n");
    rendered.push_str("| --- | --- | --- | --- | --- | --- | --- |\n");
    for row in &report.failures.classification_rows {
        rendered.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} |\n",
            sanitize_markdown_cell(&row.class_id),
            sanitize_markdown_cell(&row.domain),
            sanitize_markdown_cell(&row.stage_id),
            sanitize_markdown_cell(&row.tool_id),
            sanitize_markdown_cell(&row.source_surface),
            sanitize_markdown_cell(&row.observed_status),
            sanitize_markdown_cell(&row.detail)
        ));
    }
    rendered.push('\n');

    rendered.push_str("## Missing Results\n\n");
    rendered.push_str(
        "| Result ID | Domain | Stage | Tool | Corpus | Section | Manifest Path | Reason |\n",
    );
    rendered.push_str("| --- | --- | --- | --- | --- | --- | --- | --- |\n");
    for row in &report.missing_results {
        rendered.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} |\n",
            sanitize_markdown_cell(&row.result_id),
            sanitize_markdown_cell(&row.domain),
            sanitize_markdown_cell(&row.stage_id),
            sanitize_markdown_cell(&row.tool_id),
            sanitize_markdown_cell(&row.corpus_id),
            sanitize_markdown_cell(&row.report_section),
            sanitize_markdown_cell(row.audit_manifest_path.as_deref().unwrap_or("")),
            sanitize_markdown_cell(&row.reason)
        ));
    }
    rendered.push('\n');

    rendered.push_str("## Comparable Metrics\n\n");
    rendered.push_str("| Domain | Stage | Metric ID | Metric Name | Unit | Direction | Tolerance Kind | Tolerance | Pass Rule | Insufficiency Behavior | Required | Tools | Contract Status |\n");
    rendered.push_str(
        "| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |\n",
    );
    for row in &report.comparable_metrics {
        rendered.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
            sanitize_markdown_cell(&row.domain),
            sanitize_markdown_cell(&row.stage_id),
            sanitize_markdown_cell(&row.metric_id),
            sanitize_markdown_cell(&row.metric_name),
            sanitize_markdown_cell(row.unit.as_deref().unwrap_or("")),
            sanitize_markdown_cell(&row.direction),
            sanitize_markdown_cell(&row.tolerance_kind),
            format_threshold_value(row.tolerance_value),
            sanitize_markdown_cell(&row.pass_rule),
            sanitize_markdown_cell(&row.insufficiency_behavior),
            row.required,
            sanitize_markdown_cell(&row.tool_ids.join(", ")),
            sanitize_markdown_cell(&row.contract_status)
        ));
    }
    rendered.push('\n');

    rendered.push_str("## Unsupported Pairs\n\n");
    rendered.push_str("| Domain | Stage | Tool | Evidence Path | Detail |\n");
    rendered.push_str("| --- | --- | --- | --- | --- |\n");
    for row in &report.unsupported_pairs {
        rendered.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            sanitize_markdown_cell(&row.domain),
            sanitize_markdown_cell(&row.stage_id),
            sanitize_markdown_cell(&row.tool_id),
            sanitize_markdown_cell(&row.evidence_path),
            sanitize_markdown_cell(&row.detail)
        ));
    }

    rendered
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

fn report_row_status_label(status: FullBenchmarkReportRowStatus) -> &'static str {
    match status {
        FullBenchmarkReportRowStatus::Present => "present",
        FullBenchmarkReportRowStatus::MissingResult => "missing_result",
        FullBenchmarkReportRowStatus::UnsupportedPair => "unsupported_pair",
    }
}

fn sanitize_markdown_cell(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', " ")
}

fn format_optional_f64(value: Option<f64>) -> String {
    value.map(|value| format!("{value:.3}")).unwrap_or_default()
}

fn format_optional_u32(value: Option<u32>) -> String {
    value.map(|value| value.to_string()).unwrap_or_default()
}

fn format_threshold_value(value: f64) -> String {
    let rendered = format!("{value:.6}");
    rendered.trim_end_matches('0').trim_end_matches('.').to_string()
}
