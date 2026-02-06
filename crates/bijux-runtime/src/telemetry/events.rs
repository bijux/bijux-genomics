#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunEvent {
    pub timestamp: String,
    pub event: String,
    pub stage: Option<String>,
    pub tool: Option<String>,
    pub detail: Option<String>,
}
