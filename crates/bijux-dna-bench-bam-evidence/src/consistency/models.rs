use serde::Serialize;
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BamActiveRowConsistencySurfaceCheck {
    pub surface_id: String,
    pub output_path: String,
    pub row_count: usize,
    pub stage_count: usize,
    pub tool_count: usize,
    pub binding_count: usize,
    pub ok: bool,
    pub missing_bindings: Vec<String>,
    pub extra_bindings: Vec<String>,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct BamActiveRowConsistencyReport {
    pub schema_version: &'static str,
    pub output_path: String,
    pub active_matrix_output_path: String,
    pub active_row_count: usize,
    pub active_stage_count: usize,
    pub active_tool_count: usize,
    pub checked_surface_count: usize,
    pub passed_surface_count: usize,
    pub failed_surface_count: usize,
    pub ok: bool,
    pub surfaces: Vec<BamActiveRowConsistencySurfaceCheck>,
    pub failed_surfaces: Vec<BamActiveRowConsistencySurfaceCheck>,
}
