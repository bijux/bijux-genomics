use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StageExecutionRecordV1 {
    pub stage_id: String,
    pub attempt: u32,
    pub success: bool,
    pub cached: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunRecordV1 {
    pub schema_version: String,
    pub stages: Vec<StageExecutionRecordV1>,
}

impl RunRecordV1 {
    #[must_use]
    pub fn new(stages: Vec<StageExecutionRecordV1>) -> Self {
        Self {
            schema_version: "bijux.run_record.v1".to_string(),
            stages,
        }
    }
}
