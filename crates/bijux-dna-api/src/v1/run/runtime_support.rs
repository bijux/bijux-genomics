use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Context;
use serde::Serialize;

pub use bijux_dna_core::contract::ExecutionManifest;
pub use bijux_dna_core::prelude::{
    run_dir, PathSpec, Profile, RunSpec, StageId, ToolId, ToolRegistry, ToolRole,
};
pub use bijux_dna_environment::api::{load_image_catalog, load_platform, RuntimeKind};
pub use bijux_dna_infra::RUN_LAYOUT_CONTRACT;
pub use bijux_dna_infra::{
    atomic_write_bytes, ensure_dir, init_logging, temp_dir, temp_dir_in, write_bytes,
};
pub use bijux_dna_runner::backend::docker::execution_spec::build_tool_execution_spec;
pub use bijux_dna_runner::backend::docker::replay::replay_run;
pub use bijux_dna_runner::command_runner::{
    run_command, run_command_with_context, CommandOutputV1,
};
pub use bijux_dna_runtime::manifests::load_manifests;
pub use bijux_dna_runtime::run::{load_profile, new_run_id, resolve_run_base_dir};
pub use bijux_dna_runtime::FactsRowV1;
pub use bijux_dna_stage_contract::StagePlanV1;
pub use bijux_dna_stage_contract::{execution_step_from_stage_plan, DryRunExecutor, Executor};

#[derive(Debug, Serialize)]
struct PlanArtifactManifest<'a> {
    schema_version: &'static str,
    run_id: &'a str,
    stage_id: &'a str,
    tool_id: &'a str,
    artifacts: &'a [TypedPlannedArtifact],
}

#[derive(Debug, Serialize)]
struct DecisionTraceSelection<'a> {
    stage_id: &'a str,
    tool_id: &'a str,
    reason: &'a serde_json::Value,
    provenance_notes: &'a [String],
    comparability_notes: &'a [String],
}

#[derive(Debug, Serialize)]
struct DecisionTraceProvenanceLinks<'a> {
    defaults_ledger: &'a str,
    run_manifest: &'a str,
    tool_invocation: &'a str,
    metrics_envelope: &'a str,
    stage_report: &'a str,
}

#[derive(Debug, Serialize)]
struct DecisionTrace<'a> {
    schema_version: &'static str,
    run_id: &'a str,
    stage_id: &'a str,
    tool_id: &'a str,
    selection: DecisionTraceSelection<'a>,
    provenance_links: DecisionTraceProvenanceLinks<'a>,
}

#[derive(Debug, Serialize)]
struct TypedPlannedArtifact {
    artifact_id: String,
    role: String,
    path: String,
    kind: String,
    schema: String,
}

/// Materialize API-owned plan support artifacts for a run execution plan.
///
/// # Errors
/// Returns an error when support artifacts cannot be serialized or written.
pub fn write_plan_support_artifacts(plan: &bijux_dna_stage_contract::RunExecutionPlan) -> anyhow::Result<()> {
    let planned_artifacts = plan
        .planned_artifacts
        .iter()
        .map(|artifact| TypedPlannedArtifact {
            artifact_id: artifact.artifact_id.clone(),
            role: artifact.role.clone(),
            path: artifact.path.clone(),
            kind: artifact.kind.clone(),
            schema: artifact.schema.clone(),
        })
        .collect::<Vec<_>>();

    let stage_id = plan.stage.stage_id.to_string();
    let tool_id = plan.stage.tool_id.to_string();
    let run_id = plan.run_id.0.as_str();

    let manifest = PlanArtifactManifest {
        schema_version: "bijux.plan_artifacts.v1",
        run_id,
        stage_id: stage_id.as_str(),
        tool_id: tool_id.as_str(),
        artifacts: &planned_artifacts,
    };
    let provenance_notes =
        vec![format!("planner_stage={stage_id}"), format!("selected_tool={tool_id}")];
    let comparability_notes =
        vec!["compare against runs with same stage id and artifact schema set".to_string()];
    let reason = serde_json::to_value(&plan.stage.reason)?;
    let decision_trace = DecisionTrace {
        schema_version: "bijux.decision_trace.v1",
        run_id,
        stage_id: stage_id.as_str(),
        tool_id: tool_id.as_str(),
        selection: DecisionTraceSelection {
            stage_id: stage_id.as_str(),
            tool_id: tool_id.as_str(),
            reason: &reason,
            provenance_notes: &provenance_notes,
            comparability_notes: &comparability_notes,
        },
        provenance_links: DecisionTraceProvenanceLinks {
            defaults_ledger: "defaults_ledger.json",
            run_manifest: "run_manifest.json",
            tool_invocation: "tool_invocation.json",
            metrics_envelope: "metrics_envelope.json",
            stage_report: "stage_report.json",
        },
    };
    write_bytes(
        plan.artifacts_dir.join("plan_artifact_manifest.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )
    .context("write plan_artifact_manifest.json")?;
    write_bytes(
        plan.artifacts_dir.join("decision_trace.json"),
        serde_json::to_vec_pretty(&decision_trace)?,
    )
    .context("write decision_trace.json")?;

    let commit_hash = crate::v1::env::run_shell_capture("git rev-parse HEAD")
        .ok()
        .map(|raw| raw.trim().to_string())
        .filter(|value| !value.is_empty());
    let checks: serde_json::Value = std::env::var("BIJUX_POLICY_CLEAN_REPORT_JSON")
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_else(
            || serde_json::json!({"schema_version":"bijux.policy.clean.v1","ok":false}),
        );
    let payload = serde_json::json!({
        "schema_version": "bijux.policy_snapshot.v1",
        "git_commit": commit_hash,
        "checked_at_unix_s": SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs()),
        "checks": checks,
    });
    write_bytes(
        plan.artifacts_dir.join("policy_snapshot.json"),
        serde_json::to_vec_pretty(&payload)?,
    )
    .context("write policy_snapshot.json")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::write_plan_support_artifacts;
    use bijux_dna_stage_contract::RunExecutionPlan;

    #[test]
    fn plan_support_artifacts_are_materialized_from_the_api_surface() {
        let temp = tempfile::tempdir().expect("tempdir");
        let plan: RunExecutionPlan = serde_json::from_str(include_str!(
            "../../../../bijux-dna-stage-contract/tests/fixtures/public_types/default/run_execution_plan.json"
        ))
        .expect("parse run execution plan fixture");
        let mut plan = plan;
        plan.artifacts_dir = temp.path().join("artifacts");
        std::fs::create_dir_all(&plan.artifacts_dir).expect("create artifacts dir");

        write_plan_support_artifacts(&plan).expect("write plan support artifacts");

        for name in [
            "plan_artifact_manifest.json",
            "decision_trace.json",
            "policy_snapshot.json",
        ] {
            assert!(
                plan.artifacts_dir.join(name).is_file(),
                "expected {name} to be materialized"
            );
        }
    }
}
