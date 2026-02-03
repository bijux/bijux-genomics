fn run_plan(cli: &Cli, registry: &bijux_core::ToolRegistry, domain_dir: &Path) -> Result<()> {
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
    let plan = bijux_api::run::build_stage_plan(run_spec, registry, profile, run_id.clone())
        .map_err(|err| anyhow!("failed to build plan: {err}"))?;

    std::fs::create_dir_all(&plan.logs_dir).context("create logs directory")?;
    std::fs::create_dir_all(&plan.artifacts_dir).context("create artifacts directory")?;
    let log_path = plan.logs_dir.join("bijux.log");
    let _log_guard = init_logging(&log_path)?;

    println!("{}", serde_json::to_string_pretty(&plan)?);
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
