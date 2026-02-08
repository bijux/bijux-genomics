use std::path::Path;

use bijux_core::contract::ExecutionStep;
use bijux_core::contract::{ArtifactRef, StageIO, ToolConstraints};
use bijux_core::prelude::id_catalog;
use bijux_core::prelude::{CommandSpecV1, ContainerImageRefV1};
use bijux_core::prelude::{StageId, StepId};

#[must_use]
pub fn report_stage_step(
    out_dir: &Path,
    inputs: Vec<ArtifactRef>,
    outputs: Vec<ArtifactRef>,
) -> ExecutionStep {
    ExecutionStep {
        step_id: StepId::from_static(id_catalog::REPORT_AGGREGATE_STEP),
        stage_id: StageId::from_static(id_catalog::REPORT_AGGREGATE_STAGE),
        command: CommandSpecV1 {
            template: vec!["report-aggregate".to_string()],
        },
        image: ContainerImageRefV1 {
            image: "bijux-report".to_string(),
            digest: None,
        },
        resources: ToolConstraints::default(),
        io: StageIO { inputs, outputs },
        out_dir: out_dir.to_path_buf(),
        aux_images: std::collections::BTreeMap::new(),
        expected_artifact_ids: Vec::new(),
        metrics_schema_ids: Vec::new(),
    }
}
