use crate::commands::support::prelude::{
    anyhow, env_doctor, load_image_catalog, load_platform, print_env_export_json, print_env_images,
    print_env_info, print_env_registry_list, render, run_env_prep, run_env_smoke,
    run_env_smoke_for_stage, Cli, EnvCommand, Result,
};

#[allow(clippy::too_many_lines)]
pub(crate) fn handle_environment_command(
    cli: &Cli,
    args: &crate::cli::EnvRootArgs,
) -> Result<bool> {
    match &args.command {
        EnvCommand::List => {
            let cwd = std::env::current_dir()?;
            let registry_path =
                bijux_dna_infra::configs_file(&cwd, "ci/registry/tool_registry.toml");
            print_env_registry_list(&registry_path)?;
        }
        EnvCommand::ExportJson => {
            let cwd = std::env::current_dir()?;
            let registry_path =
                bijux_dna_infra::configs_file(&cwd, "ci/registry/tool_registry.toml");
            print_env_export_json(&registry_path)?;
        }
        EnvCommand::ExportContainers { json } => {
            if !json {
                return Err(anyhow!("environment export-containers requires --json"));
            }
            let cwd = std::env::current_dir()?;
            let registry_path =
                bijux_dna_infra::configs_file(&cwd, "ci/registry/tool_registry.toml");
            crate::commands::cli::env::print_registry_export_containers_json(&registry_path)?;
        }
        EnvCommand::ExportHpc { json, hpc_root } => {
            let root = hpc_root.clone().map_or_else(
                || crate::commands::hpc::load_hpc_config().map(|cfg| cfg.resolve_paths().root),
                Ok,
            )?;
            let layout = crate::commands::hpc::HpcLayout::from_root(&root);
            let export = crate::commands::hpc::export_hpc_env_json(&layout)?;
            if *json {
                render::json::print_pretty(&export)?;
            } else {
                println!("containers_dir={}", export.containers_dir);
                println!("sif_count={}", export.sifs.len());
            }
        }
        EnvCommand::EnsureImages(args) => {
            let cwd = std::env::current_dir()?;
            let registry_path =
                bijux_dna_infra::configs_file(&cwd, "ci/registry/tool_registry.toml");
            let hpc_root = args.hpc_root.clone().map_or_else(
                || crate::commands::hpc::load_hpc_config().map(|cfg| cfg.resolve_paths().root),
                Ok,
            )?;
            let stages = match (&args.stage, &args.stages) {
                (Some(stage), None) => stage.clone(),
                (None, Some(stages)) => stages.clone(),
                _ => {
                    return Err(anyhow!(
                        "environment ensure-images requires exactly one of --stage or --stages"
                    ));
                }
            };
            let report = crate::commands::cli::env::ensure_apptainer_images(
                &registry_path,
                &hpc_root,
                &args.domain,
                &stages,
                args.force_smoke,
                args.repair_mismatch,
            )?;
            if args.json {
                render::json::print_pretty(&report)?;
            } else {
                println!("schema_version={}", report.schema_version);
                println!("requested_tools={}", report.tools.len());
                println!("built={}", report.built);
                println!("reused={}", report.reused);
                println!("quick_smoked={}", report.quick_smoked);
                println!("failed={}", report.failed);
            }
        }
        EnvCommand::LintApptainerDefs => {
            let cwd = std::env::current_dir()?;
            crate::commands::cli::env::lint_apptainer_defs(&cwd)?;
        }
        EnvCommand::SifInventory { hpc_root, json } => {
            let root = hpc_root.clone().map_or_else(
                || crate::commands::hpc::load_hpc_config().map(|cfg| cfg.resolve_paths().root),
                Ok,
            )?;
            let report = crate::commands::cli::env::sif_inventory(&root)?;
            if *json {
                render::json::print_pretty(&report)?;
            } else {
                println!("containers_dir={}", report.containers_dir);
                println!("sif_count={}", report.entries.len());
            }
        }
        EnvCommand::Ensure(args) => {
            let cwd = std::env::current_dir()?;
            let registry_path =
                bijux_dna_infra::configs_file(&cwd, "ci/registry/tool_registry.toml");
            let domain = crate::commands::cli::env::parse_stage_domain(&args.stage)?;
            let hpc_root = args.hpc_root.clone().map_or_else(
                || crate::commands::hpc::load_hpc_config().map(|cfg| cfg.resolve_paths().root),
                Ok,
            )?;
            let report = crate::commands::cli::env::ensure_apptainer_images(
                &registry_path,
                &hpc_root,
                &domain,
                &args.stage,
                args.force_smoke,
                args.repair_mismatch,
            )?;
            if args.json {
                render::json::print_pretty(&report)?;
            } else {
                println!("schema_version={}", report.schema_version);
                println!("requested_tools={}", report.tools.len());
                println!("built={}", report.built);
                println!("reused={}", report.reused);
                println!("quick_smoked={}", report.quick_smoked);
                println!("failed={}", report.failed);
            }
        }
        EnvCommand::ApptainerQaMatrix { hpc_root, out } => {
            let root = hpc_root.clone().map_or_else(
                || crate::commands::hpc::load_hpc_config().map(|cfg| cfg.resolve_paths().root),
                Ok,
            )?;
            let markdown = crate::commands::cli::env::generate_apptainer_qa_matrix_markdown(&root)?;
            if let Some(parent) = out.parent() {
                bijux_dna_infra::ensure_dir(parent)?;
            }
            bijux_dna_api::v1::api::run::atomic_write_bytes(out, markdown.as_bytes())?;
            println!("qa_matrix={}", out.display());
        }
        EnvCommand::Smoke(args) => {
            let cwd = std::env::current_dir()?;
            let registry_path =
                bijux_dna_infra::configs_file(&cwd, "ci/registry/tool_registry.toml");
            if let Some(stage) = args.stage.as_deref() {
                run_env_smoke_for_stage(&registry_path, &args.runtime, stage)?;
            } else if let Some(tool) = args.tool.as_deref() {
                run_env_smoke(&args.runtime, tool)?;
            } else {
                return Err(anyhow!("environment smoke requires either <tool> or --stage"));
            }
        }
        EnvCommand::Prep(args) => {
            let cwd = std::env::current_dir()?;
            let registry_path =
                bijux_dna_infra::configs_file(&cwd, "ci/registry/tool_registry.toml");
            run_env_prep(
                &registry_path,
                &args.runtime,
                args.tool.as_deref(),
                args.stage.as_deref(),
            )?;
        }
        EnvCommand::Images => {
            let platform = load_platform(cli.platform.as_deref())
                .map_err(|err| anyhow!("failed to load platform: {err}"))?;
            let catalog =
                load_image_catalog().map_err(|err| anyhow!("failed to load images: {err}"))?;
            print_env_images(&catalog, &platform)?;
        }
        EnvCommand::Info => {
            let platform = load_platform(cli.platform.as_deref())
                .map_err(|err| anyhow!("failed to load platform: {err}"))?;
            let catalog =
                load_image_catalog().map_err(|err| anyhow!("failed to load images: {err}"))?;
            print_env_info(&catalog, &platform);
        }
        EnvCommand::Doctor => {
            let platform = load_platform(cli.platform.as_deref())
                .map_err(|err| anyhow!("failed to load platform: {err}"))?;
            let catalog =
                load_image_catalog().map_err(|err| anyhow!("failed to load images: {err}"))?;
            env_doctor(&catalog, &platform);
        }
    }
    Ok(true)
}
