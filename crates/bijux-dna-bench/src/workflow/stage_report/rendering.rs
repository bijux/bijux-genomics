//! Owner: bijux-dna-bench
//! Data-driven deterministic rendering for every benchmark stage report.

use anyhow::{Context, Result};
use bijux_dna_bench_model::{
    BenchmarkComparisonStatus, BenchmarkReportComparisonPresentation,
    BenchmarkReportDeterminismPresentation, BenchmarkReportMetricFormat,
    BenchmarkReportPresentation, BenchmarkReportTitleClass, BenchmarkReportVerdict,
    BenchmarkScientificValue, BenchmarkStageReport,
};

pub fn render_stage_report_markdown(
    report: &BenchmarkStageReport,
    presentation: &BenchmarkReportPresentation,
) -> Result<String> {
    let repository_commit = &report
        .tools
        .first()
        .context("benchmark stage report has no tool observations")?
        .repository_commit;
    let verdict = match report.verdict {
        BenchmarkReportVerdict::Passed => "passed",
        BenchmarkReportVerdict::Failed => "failed",
    };
    let mut lines = vec![
        "<!-- GENERATED FILE - DO NOT EDIT -->".to_string(),
        format!("<!-- source: {} -->", presentation.generated_source),
        format!(
            "# {} {} benchmark",
            report.stage_id,
            match presentation.title_class {
                BenchmarkReportTitleClass::Qualification => "qualification",
                BenchmarkReportTitleClass::Comparative => "comparative",
            }
        ),
        String::new(),
        format!("**Verdict: {verdict}.** {}", report.interpretation),
        String::new(),
        "## Scientific Scope".to_string(),
        String::new(),
        report.scientific_question.clone(),
        String::new(),
        format!("Comparison scope: {}.", report.comparison_scope),
    ];
    if presentation.separate_accuracy_claim {
        lines.push(String::new());
    }
    lines.extend([
        format!("Accuracy claim: `{}`.", report.accuracy_claim),
        String::new(),
        "## Reproducible Design".to_string(),
        String::new(),
        format!("- Campaign: `{}`", report.run_id),
        format!("- Repository commit: `{repository_commit}`"),
        format!("- Container catalog: `{}`", report.catalog_sha256),
        format!("- Tool closure: {}", code_list(&report.design.tool_closure)),
        format!(
            "- Repetitions: {} warmup and {} measured",
            report.design.warmup_repetitions, report.design.measured_repetitions
        ),
        format!("- Corpus: {}", code_list(&report.design.corpus_ids)),
        format!("- Result contract: `{}`", report.design.result_artifact),
        String::new(),
    ]);
    if presentation.include_comparison_closure {
        let comparable = report
            .comparability
            .as_ref()
            .context("report presentation requires comparison closure")?;
        lines.extend([
            "## Comparison Closure".to_string(),
            String::new(),
            format!("- Immutable inputs: `{}`", comparable.input_lock_sha256),
            format!("- Effective parameters: `{}`", comparable.parameter_lock_sha256),
            format!("- Repository source: `{}`", comparable.repository_commit),
            String::new(),
        ]);
    }
    render_performance(report, presentation, &mut lines)?;
    render_science(report, presentation, &mut lines);
    render_comparison(report, presentation, &mut lines);
    lines.extend([String::new(), "## Limitations".to_string(), String::new()]);
    lines.extend(report.limitations.iter().map(|value| format!("- {value}")));
    Ok(lines.join("\n") + "\n")
}

fn render_performance(
    report: &BenchmarkStageReport,
    presentation: &BenchmarkReportPresentation,
    lines: &mut Vec<String>,
) -> Result<()> {
    let headings = presentation
        .performance_columns
        .iter()
        .map(|column| column.heading.as_str())
        .collect::<Vec<_>>()
        .join(" | ");
    lines.extend([
        "## Performance".to_string(),
        String::new(),
        format!("| Tool | Slurm job | {headings} |"),
        format!("|---|---:|{}", "---:|".repeat(presentation.performance_columns.len())),
    ]);
    for tool in &report.tools {
        let mut cells = Vec::with_capacity(presentation.performance_columns.len());
        for column in &presentation.performance_columns {
            let metric = tool.performance.get(&column.metric).with_context(|| {
                format!("tool {} omits report metric {}", tool.tool, column.metric)
            })?;
            let mean = format_metric(metric.mean, column.format);
            if column.include_range {
                cells.push(format!(
                    "{} ({}-{})",
                    mean,
                    format_metric(metric.minimum, column.format),
                    format_metric(metric.maximum, column.format)
                ));
            } else {
                cells.push(mean);
            }
        }
        lines.push(format!(
            "| `{}` | `{}` | {} |",
            tool.tool,
            tool.slurm.job_id,
            cells.join(" | ")
        ));
    }
    Ok(())
}

fn render_science(
    report: &BenchmarkStageReport,
    presentation: &BenchmarkReportPresentation,
    lines: &mut Vec<String>,
) {
    lines.extend([String::new(), "## Scientific Observations".to_string(), String::new()]);
    for tool in &report.tools {
        lines.extend([
            format!("### {}", tool.tool),
            String::new(),
            "| Metric | Measured repetitions |".to_string(),
            "|---|---|".to_string(),
        ]);
        for (metric, values) in &tool.scientific_observations {
            let values = values.iter().map(scientific_value).collect::<Vec<_>>().join(", ");
            lines.push(format!("| `{metric}` | {values} |"));
        }
        if !tool.scientific_strata.is_empty() {
            lines.extend([
                String::new(),
                "#### Retained sample strata".to_string(),
                String::new(),
                "| Sample | Metric | Measured repetitions |".to_string(),
                "|---|---|---|".to_string(),
            ]);
            for stratum in &tool.scientific_strata {
                for (metric, values) in &stratum.observations {
                    let values = values.iter().map(scientific_value).collect::<Vec<_>>().join(", ");
                    lines.push(format!("| `{}` | `{metric}` | {values} |", stratum.sample_id));
                }
            }
        }
        match presentation.determinism {
            BenchmarkReportDeterminismPresentation::DetailedEvidence => lines.extend([
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
                    tool.slurm.job_id,
                    tool.slurm.host,
                    tool.image.sha256,
                    tool.source_evidence_sha256
                ),
                String::new(),
            ]),
            BenchmarkReportDeterminismPresentation::CompactEvidence => lines.extend([
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
            ]),
        }
    }
}

fn render_comparison(
    report: &BenchmarkStageReport,
    presentation: &BenchmarkReportPresentation,
    lines: &mut Vec<String>,
) {
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
    ]);
    match presentation.comparison {
        BenchmarkReportComparisonPresentation::StatusOnly => return,
        BenchmarkReportComparisonPresentation::RangeOnly => lines.extend([
            "| Metric | Observed range | Allowed range | Status |".to_string(),
            "|---|---:|---:|---|".to_string(),
        ]),
        BenchmarkReportComparisonPresentation::ToolMeans => lines.extend([
            "| Metric | Tool means | Observed range | Allowed range | Status |".to_string(),
            "|---|---|---:|---:|---|".to_string(),
        ]),
    }
    for metric in &report.cross_tool_comparison.metrics {
        let status = if metric.within_tolerance { "within" } else { "outside" };
        match presentation.comparison {
            BenchmarkReportComparisonPresentation::StatusOnly => {}
            BenchmarkReportComparisonPresentation::RangeOnly => lines.push(format!(
                "| `{}` | {:.12} | {:.12} | {status} |",
                metric.metric, metric.absolute_range, metric.allowed_range
            )),
            BenchmarkReportComparisonPresentation::ToolMeans => {
                let observations = metric
                    .observations
                    .iter()
                    .map(|(tool, value)| format!("`{tool}`={value:.12}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                lines.push(format!(
                    "| `{}` | {} | {:.12} | {:.12} | {status} |",
                    metric.metric, observations, metric.absolute_range, metric.allowed_range
                ));
            }
        }
    }
}

fn format_metric(value: f64, format: BenchmarkReportMetricFormat) -> String {
    match format {
        BenchmarkReportMetricFormat::Seconds3 => format!("{value:.3}"),
        BenchmarkReportMetricFormat::Mebibytes1 => format!("{:.1}", value / 1024.0),
        BenchmarkReportMetricFormat::MebibytesPerSecond2 => {
            format!("{:.2}", value / 1_048_576.0)
        }
        BenchmarkReportMetricFormat::Number0 => format!("{value:.0}"),
        BenchmarkReportMetricFormat::Number1 | BenchmarkReportMetricFormat::Percent1 => {
            format!("{value:.1}")
        }
    }
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
