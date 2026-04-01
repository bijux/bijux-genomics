use std::collections::BTreeMap;

use bijux_dna_core::ids::LibraryModel;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ProfileManifestV1 {
    pub schema_version: &'static str,
    pub pipeline_id: String,
    pub invariants_preset: Option<String>,
    pub library_model: LibraryModel,
    pub stage_list: Vec<String>,
    pub tool_ids: BTreeMap<String, String>,
    pub param_hashes: BTreeMap<String, String>,
    pub schema_versions: BTreeMap<String, String>,
}
