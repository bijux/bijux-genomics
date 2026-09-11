#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalArtifactToolContract {
    pub tool_id: String,
    pub domains: Vec<String>,
    pub stage_ids: Vec<String>,
    pub expected_bin: String,
    pub version_command: String,
    pub help_command: String,
    pub expected_version_regex: String,
    pub declared_version: String,
}
