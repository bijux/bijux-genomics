use super::{summary_artifact, Result};
use crate::request_args::{PlanRequest, PlanResponse};

/// # Errors
/// Returns an error if planning fails.
pub fn plan(request: PlanRequest) -> Result<PlanResponse> {
    let graph_hash = request.graph.hash()?;
    let manifest = serde_json::json!({
        "schema_version": "bijux.run_manifest.v3",
        "contract_version": bijux_dna_core::contract::ContractVersion::v1(),
        "run_id": "plan-only",
        "pipeline_id": request.graph.pipeline_id().to_string(),
        "profile_id": request.profile_id,
        "graph_hash": graph_hash,
        "cache_key": serde_json::Value::Null,
        "toolchain_versions": [],
        "dataset_fingerprints": [],
        "tool_invocations": [],
        "output_artifacts": [
            {
                "kind": "graph",
                "schema": "bijux.execution_graph.v1",
                "path": "graph.json",
                "sha256": serde_json::Value::Null
            },
            {
                "kind": "run_manifest",
                "schema": "bijux.run_manifest.v3",
                "path": "run_manifest.json",
                "sha256": serde_json::Value::Null
            },
            {
                "kind": "run_summary",
                "schema": "bijux.run_summary.v1",
                "path": "run_summary.json",
                "sha256": serde_json::Value::Null
            }
        ],
        "stages": summary_artifact::planned_stage_manifest(&request.graph),
        "failures": [],
    });
    Ok(PlanResponse { graph: request.graph, graph_hash, manifest })
}
