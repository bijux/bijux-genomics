use crate::{
    catalog::{FamilyProbeSpec, ProbeKind},
    ports::{MaterializedProof, ProbeMaterializer},
    values::{is_json_path, json_pointer_string, read_json_document},
};
use anyhow::{Context, Result};
use serde_json::Value;
use std::path::Path;

pub(crate) fn materialize_family_proof(
    repo_root: &Path,
    probe_root: &Path,
    spec: &FamilyProbeSpec,
    producer: &dyn ProbeMaterializer,
) -> Result<MaterializedProof> {
    match spec.probe_kind {
        ProbeKind::LocalStage { stage_id, proof_pointer } => {
            let materialized_path = producer
                .local_stage(stage_id)
                .with_context(|| format!("materialize local stage `{stage_id}`"))?;
            let summary_path = if proof_pointer.is_some() && !is_json_path(&materialized_path) {
                let candidate = materialized_path
                    .parent()
                    .map(|parent| parent.join("report.json"))
                    .filter(|path| path.is_file());
                candidate.unwrap_or_else(|| materialized_path.clone())
            } else {
                materialized_path.clone()
            };
            let proof_path = if let Some(pointer) = proof_pointer {
                let summary = read_json_document(&summary_path)?;
                let proof_relative = json_pointer_string(&summary, pointer).with_context(|| {
                    format!(
                        "extract proof path `{pointer}` from local stage summary {}",
                        summary_path.display()
                    )
                })?;
                repo_root.join(proof_relative)
            } else {
                summary_path
            };
            let observed_tool_id = observe_tool_id(&proof_path, stage_id, producer)?;
            Ok(MaterializedProof { proof_path, observed_tool_id })
        }
        kind => producer.generate(kind, probe_root),
    }
}

fn observe_tool_id(
    proof_path: &Path,
    stage_id: &str,
    producer: &dyn ProbeMaterializer,
) -> Result<String> {
    let value = read_json_document(proof_path)?;
    if let Some(tool_id) = value
        .get("tool_id")
        .and_then(Value::as_str)
        .or_else(|| value.get("planned_tool_id").and_then(Value::as_str))
        .or_else(|| value.get("report_tool_id").and_then(Value::as_str))
    {
        return Ok(tool_id.to_string());
    }
    producer.stage_tool(stage_id).with_context(|| {
        format!("determine tool_id for stage `{stage_id}` proof {}", proof_path.display())
    })
}
