//! Owner: bijux-dna-bench
//! Deterministic Markdown rendering for FASTQ profile benchmark reports.

use bijux_dna_bench_model::{
    BenchmarkComparisonStatus, BenchmarkReportVerdict, BenchmarkScientificValue,
    BenchmarkStageReport,
};

#[must_use]
pub fn render_fastq_profile_reads_markdown(report: &BenchmarkStageReport) -> String {
    let verdict = match report.verdict {
        BenchmarkReportVerdict::Passed => "passed",
        BenchmarkReportVerdict::Failed => "failed",
    };
    let mut lines = vec![
        "<!-- GENERATED FILE - DO NOT EDIT -->".to_string(),
        "<!-- source: bijux-dna-bench::workflow::fastq_profile_reads::report -->".to_string(),
        format!("# {} comparative benchmark", report.stage_id),
        String::new(),
        format!("**Verdict: {verdict}.** {}", report.interpretation),
        String::new(),
        "## Scientific Scope".to_string(),
        String::new(),
        report.scientific_question.clone(),
        String::new(),
        format!("Comparison scope: {}.", report.comparison_scope),
        String::new(),
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
    ];
    if let Some(comparability) = &report.comparability {
        lines.extend([
            "## Comparison Closure".to_string(),
            String::new(),
            format!("- Immutable inputs: `{}`", comparability.input_lock_sha256),
            format!("- Effective parameters: `{}`", comparability.parameter_lock_sha256),
            format!("- Repository source: `{}`", comparability.repository_commit),
            String::new(),
        ]);
    }
    lines.extend([
        "## Performance".to_string(),
        String::new(),
        "| Tool | Slurm job | Wall mean (range), s | CPU mean, s | CPU use, % | Peak RSS mean, MiB | Throughput, MiB/s | Output mean, bytes |".to_string(),
        "|---|---:|---:|---:|---:|---:|---:|---:|".to_string(),
    ]);
    for tool in &report.tools {
        let wall = &tool.performance["wall_seconds"];
        lines.push(format!(
            "| `{}` | `{}` | {:.3} ({:.3}-{:.3}) | {:.3} | {:.1} | {:.1} | {:.2} | {:.1} |",
            tool.tool,
            tool.slurm.job_id,
            wall.mean,
            wall.minimum,
            wall.maximum,
            tool.performance["cpu_seconds"].mean,
            tool.performance["cpu_utilization_percent"].mean,
            tool.performance["peak_rss_kib"].mean / 1024.0,
            tool.performance["throughput_input_bytes_per_second"].mean / 1_048_576.0,
            tool.performance["output_bytes"].mean,
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
                "The `{}` artifact was identical across measured repetitions by {}: `{}`.",
                tool.determinism.artifact,
                tool.determinism.digest_basis.as_deref().unwrap_or("raw bytes"),
                tool.determinism.sha256
            ),
            String::new(),
            format!(
                "Evidence: Slurm job `{}`, node `{}`, image `{}`, acceptance `{}`.",
                tool.slurm.job_id, tool.slurm.host, tool.image.sha256, tool.source_evidence_sha256
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
        "| Metric | Tool means | Observed range | Allowed range | Status |".to_string(),
        "|---|---|---:|---:|---|".to_string(),
    ]);
    for metric in &report.cross_tool_comparison.metrics {
        let observations = metric
            .observations
            .iter()
            .map(|(tool, value)| format!("`{tool}`={value:.12}"))
            .collect::<Vec<_>>()
            .join(", ");
        lines.push(format!(
            "| `{}` | {} | {:.12} | {:.12} | {} |",
            metric.metric,
            observations,
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
