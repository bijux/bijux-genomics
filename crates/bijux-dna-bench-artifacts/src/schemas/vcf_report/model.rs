use serde::Serialize;
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VcfNormalizedMetricsSchemaRow {
    pub stage_id: String,
    pub schema_version: String,
    pub schema_id: String,
    pub file_name: String,
    pub extension_id: String,
    pub required_key_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct VcfNormalizedMetricsSchemaReport {
    pub schema_version: &'static str,
    pub output_path: String,
    pub stage_dir: String,
    pub schema_id: String,
    pub stage_count: usize,
    pub extension_count: usize,
    pub rows: Vec<VcfNormalizedMetricsSchemaRow>,
}
