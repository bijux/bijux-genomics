#[derive(Debug, Clone, serde::Serialize)]
pub struct RawFailure {
    pub stage: String,
    pub tool: String,
    pub reason: String,
}
