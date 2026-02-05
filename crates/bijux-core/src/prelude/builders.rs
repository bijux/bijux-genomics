pub struct DryRunExecutor;

impl Executor for DryRunExecutor {
    fn run(&self, plan: &RunExecutionPlan) -> Result<RunReport, BijuxError> {
        ensure_run_dirs(plan)?;
        let rendered = plan.tool.command_template.join(" ");
        info!(
            run_id = %plan.run_id.0,
            stage = %plan.stage.stage_id,
            tool = %plan.tool.tool_id,
            command = %rendered,
            "dry-run command"
        );

        let report = RunReport::new(
            RunId(plan.run_id.0.clone()),
            StageId(plan.stage.stage_id.clone()),
            ToolId(plan.tool.tool_id.clone()),
            RunStatus::Skipped,
        );
        let report_path = plan.run_dir.join("report.json");
        let payload = serde_json::to_string_pretty(&report)?;
        bijux_infra::atomic_write_bytes(&report_path, payload.as_bytes())
            .map_err(std::io::Error::other)?;
        Ok(report)
    }
}
