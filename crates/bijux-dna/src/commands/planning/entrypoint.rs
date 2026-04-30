use bijux_dna_api::v1::api::run::{
    init_logging, new_run_id, write_plan_support_artifacts, DryRunExecutor, Executor, PathSpec,
    RunSpec, ToolRegistry,
};
use tracing::{info, warn};

use crate::commands::cli;
use crate::commands::cli::render;
use crate::commands::support::prelude::{anyhow, Cli, Context, DnaCommand, Path, PathBuf, Result};
use crate::commands::support::run_profile::{ensure_profile_run_base_dir, load_profile_for_cli};
use std::collections::BTreeMap;

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
        paths: PathSpec { input: Vec::new(), output: Vec::new(), work: PathBuf::new() },
        params: BTreeMap::new(),
    };

    let mut profile = load_profile_for_cli(cli)?;
    ensure_profile_run_base_dir(&stage, &tool, &mut profile);
    let plan = bijux_dna_api::v1::api::plan::plan_run(
        bijux_dna_api::v1::api::plan::PlanRunRequest { run_spec, profile, run_id: run_id.clone() },
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

    write_plan_support_artifacts(&plan)?;

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
