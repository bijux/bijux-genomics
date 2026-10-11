use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LocalReportQcConfig {
    pub schema_version: String,
    pub tool_id: String,
    pub fixture_root: PathBuf,
    pub report_template: PathBuf,
    pub manifest_template: PathBuf,
    pub multiqc_report: PathBuf,
    pub multiqc_data_dir: PathBuf,
    pub aggregation_engine: Option<String>,
    pub aggregation_scope: Option<String>,
    pub output_dir: Option<PathBuf>,
}
