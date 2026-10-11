//! Owner: deterministic Markdown presentation of admitted micro-benchmark report rows.
use bijux_dna_bench_model::MicroBenchmarkReport;

/// Render the stable report sections without acquiring or publishing evidence.
#[must_use]
pub fn render_micro_benchmark_report_markdown(report: &MicroBenchmarkReport) -> String {
    let mut rendered = String::from("# Micro Benchmark Report\n\n");
    rendered.push_str(&format!(
        "- Result rows: {}\n- Complete rows: {}\n- Failed rows: {}\n- Missing rows: {}\n- Unavailable rows: {}\n- Insufficient-data rows: {}\n- Runtime rows: {}\n- Memory-source rows: {}\n- Science-threshold rows: {}\n\n",
        report.result_row_count,
        report.complete_row_count,
        report.failed_row_count,
        report.missing_row_count,
        report.unavailable_row_count,
        report.insufficient_data_row_count,
        report.runtime_row_count,
        report.memory_source_row_count,
        report.science_threshold_row_count
    ));

    write_complete(report, &mut rendered);
    write_failed(report, &mut rendered);
    write_missing(report, &mut rendered);
    write_unavailable(report, &mut rendered);
    write_insufficient_data(report, &mut rendered);
    write_runtime(report, &mut rendered);
    write_memory_sources(report, &mut rendered);
    write_science_thresholds(report, &mut rendered);
    rendered
}

fn write_complete(report: &MicroBenchmarkReport, rendered: &mut String) {
    rendered.push_str("## Complete\n\n");
    rendered.push_str("| Execution ID | Component | Domain | Stage | Tool | Metrics | Outputs | Logs | Reason |\n");
    rendered.push_str("| --- | --- | --- | --- | --- | ---: | ---: | ---: | --- |\n");
    for row in &report.complete_rows {
        rendered.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
            sanitize_markdown_cell(&row.execution_id),
            sanitize_markdown_cell(&row.component_id),
            sanitize_markdown_cell(&row.domain),
            sanitize_markdown_cell(&row.stage_id),
            sanitize_markdown_cell(&row.tool_id),
            row.normalized_metric_count,
            row.output_count,
            row.log_count,
            sanitize_markdown_cell(&row.reason)
        ));
    }
    rendered.push('\n');
}

fn write_failed(report: &MicroBenchmarkReport, rendered: &mut String) {
    rendered.push_str("## Failed\n\n");
    rendered.push_str(
        "| Row ID | Source Surface | Component | Execution ID | Domain | Stage | Tool | Detail |\n",
    );
    rendered.push_str("| --- | --- | --- | --- | --- | --- | --- | --- |\n");
    for row in &report.failed_rows {
        rendered.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} |\n",
            sanitize_markdown_cell(&row.row_id),
            sanitize_markdown_cell(&row.source_surface),
            sanitize_markdown_cell(row.component_id.as_deref().unwrap_or("")),
            sanitize_markdown_cell(row.execution_id.as_deref().unwrap_or("")),
            sanitize_markdown_cell(row.domain.as_deref().unwrap_or("")),
            sanitize_markdown_cell(row.stage_id.as_deref().unwrap_or("")),
            sanitize_markdown_cell(row.tool_id.as_deref().unwrap_or("")),
            sanitize_markdown_cell(&row.detail)
        ));
    }
    rendered.push('\n');
}

fn write_missing(report: &MicroBenchmarkReport, rendered: &mut String) {
    rendered.push_str("## Missing\n\n");
    rendered.push_str(
        "| Component | Execution ID | Domain | Stage | Tool | Source Report | Detail |\n",
    );
    rendered.push_str("| --- | --- | --- | --- | --- | --- | --- |\n");
    for row in &report.missing_rows {
        rendered.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} |\n",
            sanitize_markdown_cell(&row.component_id),
            sanitize_markdown_cell(&row.execution_id),
            sanitize_markdown_cell(&row.domain),
            sanitize_markdown_cell(&row.stage_id),
            sanitize_markdown_cell(&row.tool_id),
            sanitize_markdown_cell(&row.source_report_path),
            sanitize_markdown_cell(&row.detail)
        ));
    }
    rendered.push('\n');
}

fn write_unavailable(report: &MicroBenchmarkReport, rendered: &mut String) {
    rendered.push_str("## Unavailable\n\n");
    rendered.push_str("| Execution ID | Component | Domain | Stage | Tool | Status | Reason |\n");
    rendered.push_str("| --- | --- | --- | --- | --- | --- | --- |\n");
    for row in &report.unavailable_rows {
        rendered.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} |\n",
            sanitize_markdown_cell(&row.execution_id),
            sanitize_markdown_cell(&row.component_id),
            sanitize_markdown_cell(&row.domain),
            sanitize_markdown_cell(&row.stage_id),
            sanitize_markdown_cell(&row.tool_id),
            row.execution_status.as_str(),
            sanitize_markdown_cell(&row.reason)
        ));
    }
    rendered.push('\n');
}

fn write_insufficient_data(report: &MicroBenchmarkReport, rendered: &mut String) {
    rendered.push_str("## Insufficient Data\n\n");
    rendered.push_str("| Execution ID | Component | Domain | Stage | Tool | Detail |\n");
    rendered.push_str("| --- | --- | --- | --- | --- | --- |\n");
    for row in &report.insufficient_data_rows {
        rendered.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} |\n",
            sanitize_markdown_cell(&row.execution_id),
            sanitize_markdown_cell(&row.component_id),
            sanitize_markdown_cell(&row.domain),
            sanitize_markdown_cell(&row.stage_id),
            sanitize_markdown_cell(&row.tool_id),
            sanitize_markdown_cell(&row.detail)
        ));
    }
    rendered.push('\n');
}

fn write_runtime(report: &MicroBenchmarkReport, rendered: &mut String) {
    rendered.push_str("## Runtime\n\n");
    rendered.push_str("| Execution ID | Component | Domain | Stage | Tool | Status | Elapsed Seconds | Source |\n");
    rendered.push_str("| --- | --- | --- | --- | --- | --- | ---: | --- |\n");
    for row in &report.runtime_rows {
        rendered.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} |\n",
            sanitize_markdown_cell(&row.execution_id),
            sanitize_markdown_cell(&row.component_id),
            sanitize_markdown_cell(&row.domain),
            sanitize_markdown_cell(&row.stage_id),
            sanitize_markdown_cell(&row.tool_id),
            row.execution_status.as_str(),
            format_optional_f64(row.elapsed_seconds),
            sanitize_markdown_cell(&row.runtime_source)
        ));
    }
    rendered.push('\n');
}

fn write_memory_sources(report: &MicroBenchmarkReport, rendered: &mut String) {
    rendered.push_str("## Memory Sources\n\n");
    rendered.push_str("| Execution ID | Component | Domain | Stage | Tool | Status | Declared Memory MB | Declared CPU Threads | Observed Memory MB | Observed CPU Threads | Source |\n");
    rendered.push_str("| --- | --- | --- | --- | --- | --- | ---: | ---: | ---: | ---: | --- |\n");
    for row in &report.memory_source_rows {
        rendered.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
            sanitize_markdown_cell(&row.execution_id),
            sanitize_markdown_cell(&row.component_id),
            sanitize_markdown_cell(&row.domain),
            sanitize_markdown_cell(&row.stage_id),
            sanitize_markdown_cell(&row.tool_id),
            row.execution_status.as_str(),
            format_optional_f64(row.declared_memory_mb),
            format_optional_u32(row.declared_cpu_threads),
            format_optional_f64(row.observed_memory_mb),
            format_optional_u32(row.observed_cpu_threads),
            sanitize_markdown_cell(&row.memory_source)
        ));
    }
    rendered.push('\n');
}

fn write_science_thresholds(report: &MicroBenchmarkReport, rendered: &mut String) {
    rendered.push_str("## Science Thresholds\n\n");
    rendered.push_str("| Domain | Stage | Metric ID | Metric Name | Unit | Direction | Tolerance Kind | Tolerance | Pass Rule | Insufficiency Behavior | Required | Covered Tools |\n");
    rendered
        .push_str("| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |\n");
    for row in &report.science_threshold_rows {
        rendered.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
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
            sanitize_markdown_cell(&row.covered_tool_ids.join(", "))
        ));
    }
}

fn format_optional_f64(value: Option<f64>) -> String {
    value.map_or_else(String::new, |value| format!("{value:.3}"))
}

fn format_optional_u32(value: Option<u32>) -> String {
    value.map_or_else(String::new, |value| value.to_string())
}

fn format_threshold_value(value: f64) -> String {
    format!("{value:.6}")
}

fn sanitize_markdown_cell(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', "<br>")
}
