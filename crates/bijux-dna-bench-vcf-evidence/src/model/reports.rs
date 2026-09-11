use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VcfReportMapRow {
    pub stage_id: String,
    pub tool_id: String,
    pub section_id: String,
    pub summary_table: String,
    pub metric_columns: Vec<String>,
    pub failure_columns: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VcfReportMapReport {
    pub schema_version: &'static str,
    pub output_path: String,
    pub row_count: usize,
    pub stage_count: usize,
    pub tool_count: usize,
    pub section_count: usize,
    pub summary_table_count: usize,
    pub section_counts: BTreeMap<String, usize>,
    pub rows: Vec<VcfReportMapRow>,
}
