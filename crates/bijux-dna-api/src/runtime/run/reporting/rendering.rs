use super::Result;
use crate::request_args::{ExecuteRunRequest, RenderReportRequest, RenderReportResult};
use crate::runtime::run::execute_run;
use std::path::{Path, PathBuf};

/// # Errors
/// Returns an error if execution or report rendering fails.
pub fn execute_and_report(
    exec: &ExecuteRunRequest,
    report: &RenderReportRequest,
) -> Result<RenderReportResult> {
    execute_run(exec)?;
    render_report(report)
}

/// # Errors
/// Returns an error if report rendering fails.
pub fn render_report(request: &RenderReportRequest) -> Result<RenderReportResult> {
    let report_path = render_report_from_facts(&request.base_dir, &request.facts_path)?;
    let evidence_bundle_path = bijux_dna_analyze::write_evidence_bundle_json(
        &request.base_dir,
        Some(&request.facts_path),
    )?;
    Ok(RenderReportResult { report_path, evidence_bundle_path })
}

fn render_report_from_facts(base_dir: &Path, facts_path: &Path) -> Result<PathBuf> {
    let facts = bijux_dna_analyze::load::load_facts(facts_path)?;
    let report_path = bijux_dna_analyze::report::write_run_report_from_facts(base_dir, &facts)?;
    Ok(report_path)
}
