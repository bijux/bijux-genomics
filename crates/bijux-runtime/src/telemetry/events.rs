#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunEventKind {
    RunStarted,
    StageStarted,
    ToolInvoked,
    ArtifactEmitted,
    MetricsEmitted,
    StageFinished,
    RunFinished,
    RunFailed,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunEvent {
    pub timestamp: String,
    pub event: RunEventKind,
    pub stage: Option<String>,
    pub tool: Option<String>,
    pub detail: Option<String>,
}
