pub(crate) fn run_plan(
    cli: &Cli,
    registry: &bijux_api::v1::api::run::ToolRegistry,
    domain_dir: &Path,
) -> Result<()> {
    let (stage, tool, common) = cli::resolve_stage_tool(&cli.command);

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
    let plan = bijux_api::v1::api::plan::plan_run(
        bijux_api::v1::api::plan::PlanRunRequest {
            run_spec,
            profile,
            run_id: run_id.clone(),
        },
        registry,
    )
    .map_err(|err| anyhow!("failed to build plan: {err}"))?
    .plan;

    bijux_api::v1::api::run::ensure_dir(&plan.logs_dir).context("create logs directory")?;
    bijux_api::v1::api::run::ensure_dir(&plan.artifacts_dir).context("create artifacts directory")?;
    let log_path = plan.logs_dir.join("bijux.log");
    let _log_guard = init_logging(&log_path)?;

    render::json::print_pretty(&plan)?;
    println!("manifests: {}", domain_dir.display());

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
