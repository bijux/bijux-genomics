//! Owner: bijux-dna-bench
//! Human-readable projection of an evaluated dashboard.

use bijux_dna_bench_model::FullBenchmarkDashboardReport;

/// Renders the governed dashboard table without filesystem access.
#[must_use]
pub fn render_full_benchmark_dashboard_markdown(report: &FullBenchmarkDashboardReport) -> String {
    let mut markdown = String::new();
    markdown.push_str("# Full Benchmark Dashboard\n\n");
    markdown.push_str(
        "This dashboard is generated from governed machine-readable FASTQ, BAM, and VCF readiness outputs.\n\n",
    );
    markdown.push_str("| metric | count | source path | source field | detail |\n");
    markdown.push_str("| --- | ---: | --- | --- | --- |\n");
    for metric in &report.metrics {
        markdown.push_str(&format!(
            "| {} | {} | `{}` | `{}` | {} |\n",
            metric.metric_id, metric.count, metric.source_path, metric.source_field, metric.detail
        ));
    }
    markdown.push('\n');
    markdown.push_str(&format!(
        "Unsupported pairs tracked outside the expected-job slice: {}.\n",
        report.explicit_unsupported_pairs
    ));
    markdown
}
