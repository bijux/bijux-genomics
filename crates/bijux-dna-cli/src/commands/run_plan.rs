use bijux_dna_api::v1::api::run::{
    init_logging, new_run_id, DryRunExecutor, Executor, PathSpec, RunSpec, ToolRegistry,
};
use tracing::{info, warn};

use std::collections::BTreeMap;
use crate::commands::cli;
use crate::commands::cli::render;
use crate::commands::command_prelude::{anyhow, Cli, Context, DnaCommand, Path, PathBuf, Result};
use crate::commands::validation::{ensure_profile_run_base_dir, load_profile_for_cli};

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
        &serde_json::to_value(&plan.planned_artifacts)?,
    )?;

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

fn write_plan_artifacts(
    artifacts_dir: &Path,
    run_id: &str,
    stage_id: &str,
    tool_id: &str,
    reason: &serde_json::Value,
    planned_artifacts: &serde_json::Value,
) -> Result<()> {
    let artifact_manifest = serde_json::json!({
        "schema_version": "bijux.plan_artifacts.v1",
        "run_id": run_id,
        "stage_id": stage_id,
        "tool_id": tool_id,
        "artifacts": planned_artifacts,
    });
    let decision_trace = serde_json::json!({
        "schema_version": "bijux.decision_trace.v1",
        "run_id": run_id,
        "stage_id": stage_id,
        "tool_id": tool_id,
        "reason": reason
    });
    std::fs::write(
        artifacts_dir.join("plan_artifact_manifest.json"),
        serde_json::to_vec_pretty(&artifact_manifest)?,
    )?;
    std::fs::write(
        artifacts_dir.join("decision_trace.json"),
        serde_json::to_vec_pretty(&decision_trace)?,
    )?;
    Ok(())
}
