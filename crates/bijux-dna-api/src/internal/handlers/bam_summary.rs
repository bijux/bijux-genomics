use std::path::Path;

use anyhow::{Context, Result};
use bijux_dna_core::contract::ExecutionStep;
use bijux_dna_core::contract::{ArtifactRef, ArtifactRole};
use bijux_dna_core::prelude::ArtifactId;
use bijux_dna_planner_bam::report_stage_step as build_report_stage_step;

use super::fastq::StageExecutionSummary;

#[derive(Debug, Clone)]
#[allow(dead_code, clippy::struct_field_names)]
pub(crate) struct ReportArtifacts {
    pub summary_json_path: std::path::PathBuf,
    pub summary_tsv_path: std::path::PathBuf,
    pub report_html_path: std::path::PathBuf,
}

pub(crate) fn render_bam_summary(
    out_dir: &Path,
    stage_runs: &[StageExecutionSummary],
    failures: &[serde_json::Value],
) -> Result<ReportArtifacts> {
    let root = bijux_dna_runtime::recording::run_artifacts_dir_for_out(out_dir);
    bijux_dna_infra::ensure_dir(&root).context("create bam run artifacts dir")?;

    let stages: Vec<serde_json::Value> = stage_runs
        .iter()
        .map(|entry| {
            serde_json::json!({
                "stage_id": entry.plan.step_id.0,
                "tool_id": entry.plan.image.image,
                "exit_code": entry.result.exit_code,
                "runtime_s": entry.result.runtime_s,
                "memory_mb": entry.result.memory_mb,
                "out_dir": entry.plan.out_dir,
            })
        })
        .collect();
    let authenticity_composite = find_authenticity_composite(stage_runs);
    let total_runtime_s: f64 = stage_runs.iter().map(|entry| entry.result.runtime_s).sum();
    let summary = serde_json::json!({
        "schema_version": "bijux.run_summary.v1",
        "total_runtime_s": total_runtime_s,
        "stages": stages,
        "authenticity_composite": authenticity_composite,
        "failures": failures,
    });
    let summary_json_path = root.join("summary.json");
    bijux_dna_infra::atomic_write_json(&summary_json_path, &summary)
        .context("write summary.json")?;
    let summary_tsv_path = root.join("summary.tsv");
    let mut tsv = String::from("stage_id\ttool_id\truntime_s\texit_code\n");
    for entry in stage_runs {
        let _ = std::fmt::Write::write_fmt(
            &mut tsv,
            format_args!(
                "{}\t{}\t{:.3}\t{}\n",
                entry.plan.step_id.0,
                entry.plan.image.image,
                entry.result.runtime_s,
                entry.result.exit_code
            ),
        );
    }
    bijux_dna_infra::atomic_write_bytes(&summary_tsv_path, tsv.as_bytes())
        .context("write summary.tsv")?;
    let report_html_path = root.join("report.html");
    let html = format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>BAM summary</title></head><body><pre>{}</pre></body></html>",
        serde_json::to_string_pretty(&summary).unwrap_or_default()
    );
    bijux_dna_infra::atomic_write_bytes(&report_html_path, html.as_bytes())
        .context("write report.html")?;
    Ok(ReportArtifacts {
        summary_json_path,
        summary_tsv_path,
        report_html_path,
    })
}

fn find_authenticity_composite(
    stage_runs: &[StageExecutionSummary],
) -> Option<serde_json::Value> {
    let authenticity = stage_runs
        .iter()
        .find(|entry| entry.plan.step_id.0 == "bam.authenticity")?;
    let path = authenticity.plan.out_dir.join("authenticity_composite.json");
    let raw = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&raw).ok()
}

pub(crate) fn report_stage_step(out_dir: &Path, steps: &[ExecutionStep]) -> ExecutionStep {
    let mut inputs = Vec::new();
    for entry in steps {
        let artifacts_dir = bijux_dna_runtime::recording::run_artifacts_dir_for_out(&entry.out_dir);
        let metrics_path = artifacts_dir.join("metrics_envelope.json");
        inputs.push(ArtifactRef::optional(
            ArtifactId::new(format!("metrics_envelope_{}", entry.step_id.0)),
            metrics_path,
            ArtifactRole::MetricsEnvelope,
        ));
    }
    let root = bijux_dna_runtime::recording::run_artifacts_dir_for_out(out_dir);
    let outputs = vec![
        ArtifactRef::required(
            ArtifactId::from_static("summary"),
            root.join("summary.json"),
            ArtifactRole::SummaryJson,
        ),
        ArtifactRef::required(
            ArtifactId::from_static("summary_tsv"),
            root.join("summary.tsv"),
            ArtifactRole::SummaryTsv,
        ),
        ArtifactRef::required(
            ArtifactId::from_static("report_html"),
            root.join("report.html"),
            ArtifactRole::ReportHtml,
        ),
    ];
    build_report_stage_step(out_dir, inputs, outputs)
}
