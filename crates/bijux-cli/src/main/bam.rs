use bijux_core::ToolRegistry;
use bijux_engine::api::{build_tool_execution_spec, execute_stage_plan};
use bijux_env_runtime::api::RunnerKind;
use bijux_pipelines::registry;
use bijux_pipelines::Domain;

use crate::cli::parse::{BamCommand, BamRunArgs};
// imports provided by entry.rs

#[allow(clippy::missing_errors_doc)]
pub fn handle_bam_commands(
    cli: &Cli,
    registry: &ToolRegistry,
    domain_dir: &Path,
) -> Result<bool> {
    let Commands::Bam { command } = &cli.command else {
        return Ok(false);
    };

    match command {
        BamCommand::ListStages => {
            for stage in bijux_domain_bam::BamStage::all() {
                println!("{}", stage.as_str());
            }
            Ok(true)
        }
        BamCommand::Explain { stage } => {
            let stage_id = stage.stage().as_str();
            let manifest = registry
                .stages()
                .get(stage_id)
                .ok_or_else(|| anyhow!("stage {stage_id} missing from manifests"))?;
            println!("{}", serde_json::to_string_pretty(manifest)?);
            Ok(true)
        }
        BamCommand::Run(args) => {
            run_bam_stage(cli, registry, domain_dir, args)?;
            Ok(true)
        }
    }
}

fn run_bam_stage(
    cli: &Cli,
    registry: &ToolRegistry,
    domain_dir: &Path,
    args: &BamRunArgs,
) -> Result<()> {
    let platform = load_platform(cli.platform.as_deref())
        .map_err(|err| anyhow!("failed to load platform: {err}"))?;
    let catalog =
        load_image_catalog().map_err(|err| anyhow!("failed to load image catalog: {err}"))?;
    let stage = args.stage.stage();
    let profile = registry::profile_by_id(Domain::Bam, &args.profile)?;
    let tool_id = args.tool.clone().unwrap_or_else(|| {
        profile
            .defaults
            .tools
            .get(stage.as_str())
            .cloned()
            .unwrap_or_else(|| "samtools".to_string())
    });
    let spec =
        build_tool_execution_spec(stage.as_str(), &tool_id, registry, &catalog, &platform)?;

    let out_dir = args.out.clone();
    std::fs::create_dir_all(&out_dir).context("create bam out dir")?;
    let log_path = out_dir.join("bijux_bam.log");
    let _log_guard = init_logging(&log_path)?;

    let plan =
        crate::plan_for_bam_stage_with_profile(stage, &spec, args, &profile, out_dir.as_path())?;
    println!("{}", serde_json::to_string_pretty(&plan)?);
    println!("manifests: {}", domain_dir.display());

    if args.dry_run {
        return Ok(());
    }
    execute_stage_plan(&plan, RunnerKind::Docker, None)?;
    Ok(())
}

pub(crate) fn downstream_enabled() -> bool {
    cfg!(feature = "bam_downstream")
}
