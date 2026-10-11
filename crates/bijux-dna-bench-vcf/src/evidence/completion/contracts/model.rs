use serde::Deserialize;
#[derive(Deserialize)]
pub(crate) struct PublishedAdapterReport {
    pub row_count: Option<u64>,
    pub tool_count: Option<u64>,
    pub benchmark_ready_row_count: Option<u64>,
    pub parser_output_row_count: Option<u64>,
    pub indexed_row_count: Option<u64>,
    pub normalized_output_row_count: Option<u64>,
    pub missing_input_test_passed_row_count: Option<u64>,
    pub rows: Option<Vec<PublishedAdapterRow>>,
}
#[derive(Deserialize)]
pub(crate) struct PublishedAdapterRow {
    pub tool_id: Option<String>,
    pub stage_id: Option<String>,
    pub benchmark_status: Option<String>,
    pub argv_validation_passed: Option<bool>,
    pub missing_input_test_passed: Option<bool>,
}
#[derive(Deserialize)]
pub(crate) struct PublishedReadinessReport {
    pub retained_row_count: Option<u64>,
    pub active_row_count: Option<u64>,
    pub complete_row_count: Option<u64>,
    pub incomplete_row_count: Option<u64>,
    pub checked_surface_count: Option<u64>,
    pub violation_count: Option<u64>,
    pub ok: Option<bool>,
}
