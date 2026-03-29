use std::path::Path;

use anyhow::{anyhow, Result};

use crate::commands::{cli, hpc};

pub(crate) fn handle_environment_root(
    command: &cli::EnvCommand,
    cwd: &Path,
    platform_name: Option<&str>,
) -> Result<()> {
    use crate::commands::cli::env::{
        ensure_apptainer_images, env_doctor, generate_apptainer_qa_matrix_markdown,
        lint_apptainer_defs, parse_stage_domain, print_env_export_json, print_env_images,
        print_env_info, print_env_registry_list, run_env_prep, run_env_smoke,
        run_env_smoke_for_stage, sif_inventory,
    };
    use bijux_dna_api::v1::api::env::{load_image_catalog, load_platform};
    match command {
        cli::EnvCommand::List => {
            let registry_path =
                bijux_dna_infra::configs_file(cwd, "ci/registry/tool_registry.toml");
            print_env_registry_list(&registry_path)?;
        }
        cli::EnvCommand::ExportJson => {
            let registry_path =
                bijux_dna_infra::configs_file(cwd, "ci/registry/tool_registry.toml");
            print_env_export_json(&registry_path)?;
        }
        cli::EnvCommand::ExportContainers { .. } => {
            let registry_path =
                bijux_dna_infra::configs_file(cwd, "ci/registry/tool_registry.toml");
            crate::commands::cli::env::print_registry_export_containers_json(&registry_path)?;
        }
        cli::EnvCommand::ExportHpc { json, hpc_root } => {
            let root = hpc_root.clone().map_or_else(
                || hpc::load_hpc_config().map(|cfg| cfg.resolve_paths().root),
                Ok,
            )?;
            let layout = hpc::HpcLayout::from_root(&root);
            let export = hpc::export_hpc_env_json(&layout)?;
            if *json {
                cli::render::json::print_pretty(&export)?;
            } else {
                println!("containers_dir={}", export.containers_dir);
                println!("sif_count={}", export.sifs.len());
            }
        }
        cli::EnvCommand::SifInventory { hpc_root, json } => {
            let root = hpc_root.clone().map_or_else(
                || hpc::load_hpc_config().map(|cfg| cfg.resolve_paths().root),
                Ok,
            )?;
            let report = sif_inventory(&root)?;
            if *json {
                cli::render::json::print_pretty(&report)?;
            } else {
                println!("containers_dir={}", report.containers_dir);
                println!("sif_count={}", report.entries.len());
            }
        }
        cli::EnvCommand::Ensure(args) => {
            let domain = parse_stage_domain(&args.stage)?;
            let hpc_root = args.hpc_root.clone().map_or_else(
                || hpc::load_hpc_config().map(|cfg| cfg.resolve_paths().root),
                Ok,
            )?;
            let report = ensure_apptainer_images(
                &bijux_dna_infra::configs_file(cwd, "ci/registry/tool_registry.toml"),
                &hpc_root,
                &domain,
                &args.stage,
                args.force_smoke,
                args.repair_mismatch,
            )?;
            if args.json {
                cli::render::json::print_pretty(&report)?;
            } else {
                println!("schema_version={}", report.schema_version);
                println!("requested_tools={}", report.tools.len());
                println!("built={}", report.built);
                println!("reused={}", report.reused);
                println!("quick_smoked={}", report.quick_smoked);
                println!("failed={}", report.failed);
            }
        }
        cli::EnvCommand::ApptainerQaMatrix { hpc_root, out } => {
            let root = hpc_root.clone().map_or_else(
                || hpc::load_hpc_config().map(|cfg| cfg.resolve_paths().root),
                Ok,
            )?;
            let markdown = generate_apptainer_qa_matrix_markdown(&root)?;
            if let Some(parent) = out.parent() {
                bijux_dna_infra::ensure_dir(parent)?;
            }
            bijux_dna_api::v1::api::run::atomic_write_bytes(out, markdown.as_bytes())?;
            println!("qa_matrix={}", out.display());
        }
        cli::EnvCommand::EnsureImages(args) => {
            let registry_path =
                bijux_dna_infra::configs_file(cwd, "ci/registry/tool_registry.toml");
            let hpc_root = args.hpc_root.clone().map_or_else(
                || hpc::load_hpc_config().map(|cfg| cfg.resolve_paths().root),
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
            let report = ensure_apptainer_images(
                &registry_path,
                &hpc_root,
                &args.domain,
                &stages,
                args.force_smoke,
                args.repair_mismatch,
            )?;
            if args.json {
                cli::render::json::print_pretty(&report)?;
            } else {
                println!("schema_version={}", report.schema_version);
                println!("requested_tools={}", report.tools.len());
                println!("built={}", report.built);
                println!("reused={}", report.reused);
                println!("quick_smoked={}", report.quick_smoked);
                println!("failed={}", report.failed);
            }
        }
        cli::EnvCommand::LintApptainerDefs => {
            lint_apptainer_defs(cwd)?;
        }
        cli::EnvCommand::Smoke(args) => {
            let registry_path =
                bijux_dna_infra::configs_file(cwd, "ci/registry/tool_registry.toml");
            if let Some(stage) = args.stage.as_deref() {
                run_env_smoke_for_stage(&registry_path, &args.runtime, stage)?;
            } else if let Some(tool) = args.tool.as_deref() {
                run_env_smoke(&args.runtime, tool)?;
            } else {
                return Err(anyhow!(
                    "environment smoke requires either <tool> or --stage"
                ));
            }
        }
        cli::EnvCommand::Prep(args) => {
            let registry_path =
                bijux_dna_infra::configs_file(cwd, "ci/registry/tool_registry.toml");
            run_env_prep(
                &registry_path,
                &args.runtime,
                args.tool.as_deref(),
                args.stage.as_deref(),
            )?;
        }
        cli::EnvCommand::Images | cli::EnvCommand::Info | cli::EnvCommand::Doctor => {
            let platform = load_platform(platform_name)
                .map_err(|err| anyhow!("failed to load platform: {err}"))?;
            let catalog =
                load_image_catalog().map_err(|err| anyhow!("failed to load images: {err}"))?;
            match command {
                cli::EnvCommand::Images => print_env_images(&catalog, &platform)?,
                cli::EnvCommand::Info => print_env_info(&catalog, &platform),
                cli::EnvCommand::Doctor => env_doctor(&catalog, &platform),
                cli::EnvCommand::List
                | cli::EnvCommand::ExportJson
                | cli::EnvCommand::ExportContainers { .. }
                | cli::EnvCommand::ExportHpc { .. }
                | cli::EnvCommand::SifInventory { .. }
                | cli::EnvCommand::Ensure(_)
                | cli::EnvCommand::ApptainerQaMatrix { .. }
                | cli::EnvCommand::EnsureImages(_)
                | cli::EnvCommand::LintApptainerDefs
                | cli::EnvCommand::Smoke(_)
                | cli::EnvCommand::Prep(_) => {}
            }
        }
    }
    Ok(())
}
