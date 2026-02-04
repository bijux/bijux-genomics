//! Owner: bijux-analyze
//! Render step for analyze pipeline.

use std::path::PathBuf;

use anyhow::{Context, Result};
use bijux_infra::atomic_write_bytes;

use crate::export::write_run_summary_json;
use crate::report::model::ReportModel;
use crate::report::render::bundle::write_report_bundle;
use crate::report::render::json::write_report_json;
use crate::report::write_run_report_from_facts;
use crate::AnalyzeMode;
use crate::{AnalyzeOptions, AnalyzeOutput};

use super::compute_step::AnalysisCore;

#[derive(Debug)]
pub(crate) struct RenderedArtifacts {
    pub(crate) summary: Option<PathBuf>,
    pub(crate) report: Option<PathBuf>,
    pub(crate) ranking: Option<PathBuf>,
}

pub(crate) fn render_outputs(
    core: &AnalysisCore,
    report_model: Option<ReportModel>,
    options: &AnalyzeOptions,
) -> Result<RenderedArtifacts> {
    let mut rendered = RenderedArtifacts {
        summary: None,
        report: None,
        ranking: None,
    };

    if matches!(options.mode, AnalyzeMode::Summary | AnalyzeMode::Report) {
        let summary_path = core.base_dir.join("run_summary.json");
        write_run_summary_json(&summary_path, &core.facts_rows)?;
        rendered.summary = Some(summary_path);
    }

    if matches!(options.mode, AnalyzeMode::Report) {
        if let Some(model) = report_model {
            let report_path = core.base_dir.join("report.json");
            write_report_json(&report_path, &model).context("write report.json")?;
            let bundle_dir = core.base_dir.join("report_bundle");
            write_report_bundle(&bundle_dir, &model).context("write report bundle")?;
            rendered.report = Some(report_path);
        } else {
            let report_path = write_run_report_from_facts(&core.base_dir, &core.facts_rows)?;
            rendered.report = Some(report_path);
        }
    }

    if let Some(rankings) = &core.rankings {
        let rank_path = core.base_dir.join("ranking.json");
        atomic_write_bytes(&rank_path, &serde_json::to_vec_pretty(rankings)?)
            .map_err(anyhow::Error::from)
            .context("write ranking.json")?;
        rendered.ranking = Some(rank_path);
    }

    Ok(rendered)
}

pub(crate) fn merge_output(output: &mut AnalyzeOutput, rendered: RenderedArtifacts) {
    output.summary_json = rendered.summary;
    output.report_json = rendered.report;
    output.ranking_json = rendered.ranking;
}
