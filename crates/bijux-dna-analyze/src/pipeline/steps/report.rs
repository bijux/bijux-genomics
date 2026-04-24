//! Owner: bijux-dna-analyze
//! Report model step for analyze pipeline.

use anyhow::Result;

use crate::report::build::build_run_report_model;
use crate::report::render_model::ReportModel;
use crate::{AnalyzeMode, AnalyzeOptions};

use super::compute::AnalysisCore;

pub(crate) fn build_report(
    core: &AnalysisCore,
    options: &AnalyzeOptions,
) -> Result<Option<ReportModel>> {
    if matches!(options.mode, AnalyzeMode::Report) {
        Ok(Some(build_run_report_model(&core.base_dir, &core.facts_rows)?))
    } else {
        Ok(None)
    }
}
