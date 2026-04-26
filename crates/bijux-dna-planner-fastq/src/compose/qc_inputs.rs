use bijux_dna_core::prelude::ArtifactId;
use bijux_dna_stage_contract::{ArtifactRef, StagePlanV1};

use crate::STAGE_REPORT_QC;

use super::input_resolution::stage_node_id_for_plan;

pub(super) fn qc_input_artifacts_for_stage(stage_id: &str, plan: &StagePlanV1) -> Vec<ArtifactRef> {
    if stage_id == STAGE_REPORT_QC.as_str() {
        return Vec::new();
    }
    let governed_output_ids = crate::qc_contract::governed_qc_output_ids_for_stage(stage_id);
    if governed_output_ids.is_empty() {
        return Vec::new();
    }
    plan.io
        .outputs
        .iter()
        .filter(|artifact| {
            governed_output_ids.iter().any(|artifact_id| artifact.name.as_str() == artifact_id)
        })
        .map(|artifact| report_qc_input_artifact(stage_node_id_for_plan(plan), artifact))
        .collect()
}

pub(super) fn report_qc_input_artifact(
    source_stage_node_id: &str,
    artifact: &ArtifactRef,
) -> ArtifactRef {
    ArtifactRef {
        name: ArtifactId::new(format!("{}.{}", source_stage_node_id, artifact.name.as_str())),
        path: artifact.path.clone(),
        role: artifact.role,
        optional: artifact.optional,
    }
}
