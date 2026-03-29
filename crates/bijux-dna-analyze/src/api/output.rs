use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct RenderOptions {
    pub json: bool,
    pub html: bool,
    pub output_dir: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct AnalyzeOutput {
    pub run_id: Option<String>,
    pub report_json: Option<PathBuf>,
    pub report_html: Option<PathBuf>,
    pub summary_json: Option<PathBuf>,
    pub compare_json: Option<PathBuf>,
    pub ranking_json: Option<PathBuf>,
    pub decision_trace_json: Option<PathBuf>,
}
