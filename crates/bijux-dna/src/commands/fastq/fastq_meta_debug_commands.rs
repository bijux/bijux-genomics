#[cfg(debug_assertions)]
fn handle_debug_command(cli: &Cli, dna_command: &DnaCommand, registry_path: &Path) -> Result<Option<bool>> {
    match dna_command {
        #[cfg(debug_assertions)]
        DnaCommand::ValidateManifests => {
            let registry = load_manifests(registry_path)
                .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
            println!(
                "validated {} stages and {} tools",
                registry.stages().len(),
                registry
                    .stages()
                    .keys()
                    .map(|stage| registry.tools_for_stage(stage).len())
                    .sum::<usize>()
            );
            Ok(Some(true))
        }
        #[cfg(debug_assertions)]
        DnaCommand::Platform => {
            let platform = load_platform(cli.platform.as_deref())
                .map_err(|err| anyhow!("failed to load platform: {err}"))?;
            render::json::print_pretty(&platform)?;
            Ok(Some(true))
        }
        #[cfg(debug_assertions)]
        DnaCommand::ImageQa => {
            run_image_qa(cli.platform.as_deref())?;
            Ok(Some(true))
        }
        #[cfg(debug_assertions)]
        DnaCommand::Replay(args) => {
            if let Some(manifest_path) = args.manifest.as_ref() {
                bijux_dna_api::v1::api::run::replay_manifest(manifest_path, args.validate_only)?;
                return Ok(Some(true));
            }
            let manifest_path = args
                .search_root
                .join(&args.run_id)
                .join("run_manifest.json");
            bijux_dna_api::v1::api::run::replay_manifest(&manifest_path, args.validate_only)?;
            Ok(Some(true))
        }
        #[cfg(debug_assertions)]
        DnaCommand::Compare(args) => {
            let objective = objective_spec(Objective::Balanced);
            let run_a = args.search_root.join(&args.run_a);
            let run_b = args.search_root.join(&args.run_b);
            let result = if let Some(baseline) = args.baseline.as_ref() {
                let baseline_dir = args.search_root.join(baseline);
                compare_runs_with_baseline(&run_a, &run_b, &baseline_dir, &objective)?
            } else {
                compare_runs(&run_a, &run_b, &objective)?
            };
            let output_dir = args.output_dir.as_ref().unwrap_or(&args.search_root);
            bijux_dna_api::v1::api::run::ensure_dir(output_dir)?;
            let path = output_dir.join("compare.json");
            atomic_write_bytes(&path, &serde_json::to_vec_pretty(&result)?)
                .map_err(anyhow::Error::from)?;
            render::json::print_pretty(&result)?;
            Ok(Some(true))
        }
        #[cfg(debug_assertions)]
        DnaCommand::Policies { command } => {
            match command {
                PoliciesCommand::Audit { out } => {
                    workspace_audit(out)?;
                }
            }
            Ok(Some(true))
        }
        _ => Ok(None),
    }
}

#[cfg(not(debug_assertions))]
fn handle_debug_command(_cli: &Cli, _dna_command: &DnaCommand, _registry_path: &Path) -> Result<Option<bool>> {
    Ok(None)
}
