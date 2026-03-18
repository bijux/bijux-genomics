use bijux_dna_api::v1::api::run::{
    init_logging, new_run_id, DryRunExecutor, Executor, PathSpec, RunSpec, ToolRegistry,
};
use serde::Serialize;
use tracing::{info, warn};

use crate::commands::cli;
use crate::commands::cli::render;
use crate::commands::command_prelude::{anyhow, Cli, Context, DnaCommand, Path, PathBuf, Result};
use crate::commands::validation::{ensure_profile_run_base_dir, load_profile_for_cli};
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) fn run_plan(
    cli: &Cli,
    dna_command: &DnaCommand,
    registry: &ToolRegistry,
    domain_dir: &Path,
) -> Result<()> {
    let (stage, tool, common) = cli::resolve_stage_tool(dna_command);

    let run_id = new_run_id();
    let run_spec = RunSpec {
        stage: stage.clone(),
        tool: tool.clone(),
        paths: PathSpec {
            input: Vec::new(),
            output: Vec::new(),
            work: PathBuf::new(),
        },
        params: BTreeMap::new(),
    };

    let mut profile = load_profile_for_cli(cli)?;
    ensure_profile_run_base_dir(&stage, &tool, &mut profile);
    let plan = bijux_dna_api::v1::api::plan::plan_run(
        bijux_dna_api::v1::api::plan::PlanRunRequest {
            run_spec,
            profile,
            run_id: run_id.clone(),
        },
        registry,
    )
    .map_err(|err| anyhow!("failed to build plan: {err}"))?
    .plan;

    bijux_dna_api::v1::api::run::ensure_dir(&plan.logs_dir).context("create logs directory")?;
    bijux_dna_api::v1::api::run::ensure_dir(&plan.artifacts_dir)
        .context("create artifacts directory")?;
    let log_path = plan.logs_dir.join("bijux.log");
    let _log_guard = init_logging(&log_path)?;

    render::json::print_pretty(&plan)?;
    println!("manifests: {}", domain_dir.display());

    write_plan_artifacts(
        &plan.artifacts_dir,
        &plan.run_id.0,
        &plan.stage.stage_id.to_string(),
        &plan.stage.tool_id.to_string(),
        &serde_json::to_value(&plan.stage.reason)?,
        &plan
            .planned_artifacts
            .iter()
            .map(|artifact| TypedPlannedArtifact {
                artifact_id: artifact.artifact_id.clone(),
                role: artifact.role.clone(),
                path: artifact.path.clone(),
                kind: artifact.kind.clone(),
                schema: artifact.schema.clone(),
            })
            .collect::<Vec<_>>(),
    )?;
    write_policy_snapshot(&plan.artifacts_dir)?;

    if !common.dry_run {
        warn!(
            run_id = %plan.run_id.0,
            stage = %plan.stage.stage_id,
            tool = %plan.tool.tool_id,
            "no executor implemented yet, falling back to dry-run"
        );
    }

    let executor = DryRunExecutor;
    executor.run(&plan)?;
    info!(
        run_id = %plan.run_id.0,
        stage = %plan.stage.stage_id,
        tool = %plan.tool.tool_id,
        "report written"
    );

    Ok(())
}

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

fn write_plan_artifacts(
    artifacts_dir: &Path,
    run_id: &str,
    stage_id: &str,
    tool_id: &str,
    reason: &serde_json::Value,
    planned_artifacts: &[TypedPlannedArtifact],
) -> Result<()> {
    let manifest = PlanArtifactManifest {
        schema_version: "bijux.plan_artifacts.v1",
        run_id,
        stage_id,
        tool_id,
        artifacts: planned_artifacts,
    };
    let provenance_notes = vec![
        format!("planner_stage={stage_id}"),
        format!("selected_tool={tool_id}"),
    ];
    let comparability_notes =
        vec!["compare against runs with same stage id and artifact schema set".to_string()];
    let decision_trace = DecisionTrace {
        schema_version: "bijux.decision_trace.v1",
        run_id,
        stage_id,
        tool_id,
        selection: DecisionTraceSelection {
            stage_id,
            tool_id,
            reason,
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
    bijux_dna_api::v1::api::run::write_bytes(
        artifacts_dir.join("plan_artifact_manifest.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )
    .context("write plan_artifact_manifest.json")?;
    bijux_dna_api::v1::api::run::write_bytes(
        artifacts_dir.join("decision_trace.json"),
        serde_json::to_vec_pretty(&decision_trace)?,
    )
    .context("write decision_trace.json")?;
    Ok(())
}

fn write_policy_snapshot(artifacts_dir: &Path) -> Result<()> {
    let commit_hash = bijux_dna_api::v1::api::env::run_shell_capture("git rev-parse HEAD")
        .ok()
        .map(|raw| raw.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "unknown".to_string());
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
    bijux_dna_api::v1::api::run::write_bytes(
        artifacts_dir.join("policy_snapshot.json"),
        serde_json::to_vec_pretty(&payload)?,
    )
    .context("write policy_snapshot.json")?;
    Ok(())
}
