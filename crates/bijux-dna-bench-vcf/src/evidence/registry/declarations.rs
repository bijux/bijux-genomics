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
    #[serde(flatten)]
    pub smoke: RegistrySmokeFields,
}
#[derive(Deserialize)]
pub(super) struct RequiredToolsDocument {
    pub required_tools: Vec<String>,
}

#[derive(Deserialize, Default)]
pub(super) struct RegistrySmokeFields {
    #[serde(default)]
    pub dockerfile: String,
    #[serde(default)]
    pub apptainer_def: String,
    #[serde(default)]
    pub smoke_minimal_cmd: String,
    #[serde(default)]
    pub smoke_minimal_rationale: String,
}
