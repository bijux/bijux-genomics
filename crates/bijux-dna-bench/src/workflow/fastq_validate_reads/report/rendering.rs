//! Owner: bijux-dna-bench
//! Deterministic Markdown rendering for FASTQ validation benchmark reports.

use bijux_dna_bench_model::{
    BenchmarkComparisonStatus, BenchmarkReportVerdict, BenchmarkScientificValue,
    BenchmarkStageReport,
};

#[must_use]
pub fn render_fastq_validate_reads_markdown(report: &BenchmarkStageReport) -> String {
    let verdict = match report.verdict {
        BenchmarkReportVerdict::Passed => "passed",
        BenchmarkReportVerdict::Failed => "failed",
    };
    let mut lines = vec![
        "<!-- GENERATED FILE - DO NOT EDIT -->".to_string(),
        "<!-- source: bijux-dna-bench::workflow::fastq_validate_reads::report -->".to_string(),
        format!("# {} comparative benchmark", report.stage_id),
        String::new(),
        format!("**Verdict: {verdict}.** {}", report.interpretation),
        String::new(),
        "## Scientific Scope".to_string(),
        String::new(),
        report.scientific_question.clone(),
        String::new(),
        format!("Comparison scope: {}.", report.comparison_scope),
        format!("Accuracy claim: `{}`.", report.accuracy_claim),
        String::new(),
        "## Reproducible Design".to_string(),
        String::new(),
        format!("- Campaign: `{}`", report.run_id),
        format!("- Repository commit: `{}`", report.tools[0].repository_commit),
        format!("- Container catalog: `{}`", report.catalog_sha256),
        format!("- Tool closure: {}", code_list(&report.design.tool_closure)),
        format!(
            "- Repetitions: {} warmup and {} measured",
            report.design.warmup_repetitions, report.design.measured_repetitions
        ),
        format!("- Corpus: {}", code_list(&report.design.corpus_ids)),
        format!("- Result contract: `{}`", report.design.result_artifact),
        String::new(),
        "## Performance".to_string(),
        String::new(),
        "| Tool | Slurm job | Wall mean (range), s | CPU mean, s | Peak RSS mean, MiB | Throughput, MiB/s |".to_string(),
        "|---|---:|---:|---:|---:|---:|".to_string(),
    ];
    for tool in &report.tools {
        let wall = &tool.performance["wall_seconds"];
        lines.push(format!(
            "| `{}` | `{}` | {:.3} ({:.3}-{:.3}) | {:.3} | {:.1} | {:.2} |",
            tool.tool,
            tool.slurm.job_id,
            wall.mean,
            wall.minimum,
            wall.maximum,
            tool.performance["cpu_seconds"].mean,
            tool.performance["peak_rss_kib"].mean / 1024.0,
            tool.performance["throughput_input_bytes_per_second"].mean / 1_048_576.0,
        ));
    }
    lines.extend([String::new(), "## Scientific Observations".to_string(), String::new()]);
    for tool in &report.tools {
        lines.extend([
            format!("### {}", tool.tool),
            String::new(),
            "| Metric | Measured repetitions |".to_string(),
            "|---|---|".to_string(),
        ]);
        for (metric, values) in &tool.scientific_observations {
            lines.push(format!(
                "| `{metric}` | {} |",
                values.iter().map(scientific_value).collect::<Vec<_>>().join(", ")
            ));
        }
        lines.extend([
            String::new(),
            format!(
                "Deterministic `{}` digest: `{}`. Slurm job `{}`, node `{}`, image `{}`.",
                tool.determinism.artifact,
                tool.determinism.sha256,
                tool.slurm.job_id,
                tool.slurm.host,
                tool.image.sha256
            ),
            String::new(),
        ]);
    }
    lines.extend([
        "## Cross-tool Agreement".to_string(),
        String::new(),
        format!(
            "Agreement status: **{}**.",
            match report.cross_tool_comparison.status {
                BenchmarkComparisonStatus::NotApplicable => "not applicable",
                BenchmarkComparisonStatus::WithinTolerance => "within tolerance",
                BenchmarkComparisonStatus::OutsideTolerance => "outside tolerance",
            }
        ),
        String::new(),
        "| Metric | Observed range | Allowed range | Status |".to_string(),
        "|---|---:|---:|---|".to_string(),
    ]);
    for metric in &report.cross_tool_comparison.metrics {
        lines.push(format!(
            "| `{}` | {:.12} | {:.12} | {} |",
            metric.metric,
            metric.absolute_range,
            metric.allowed_range,
            if metric.within_tolerance { "within" } else { "outside" }
        ));
    }
    lines.extend([String::new(), "## Limitations".to_string(), String::new()]);
    lines.extend(report.limitations.iter().map(|value| format!("- {value}")));
    lines.join("\n") + "\n"
}

fn code_list(values: &[String]) -> String {
    values.iter().map(|value| format!("`{value}`")).collect::<Vec<_>>().join(", ")
}

fn scientific_value(value: &BenchmarkScientificValue) -> String {
    match value {
        BenchmarkScientificValue::Number(value) => format!("{value:.12}"),
        BenchmarkScientificValue::Text(value) => format!("`{value}`"),
    }
}
