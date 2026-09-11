use serde::Deserialize;
#[derive(Deserialize)]
pub(super) struct RegistryDocument {
    pub tools: Vec<RegistryTool>,
}
#[derive(Deserialize)]
pub(super) struct RegistryTool {
    pub id: String,
    pub expected_bin: String,
    pub stage_ids: Vec<String>,
    pub status: String,
}
#[derive(Deserialize)]
pub(super) struct RequiredToolsDocument {
    pub required_tools: Vec<String>,
}
