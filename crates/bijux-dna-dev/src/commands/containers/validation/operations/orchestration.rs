#![allow(clippy::too_many_lines)]

use super::{
    anyhow, append_named_outcome, artifact_root_path, check_apptainer_hardening,
    check_apptainer_post_pins, check_apptainer_version_label_sync, check_docker_context,
    check_docker_hardening, check_docker_labels, check_docker_unpinned_apt,
    check_docker_version_sync, check_hpc_frontend_policy_enforcement, check_hpc_image_naming,
    check_lock_matches_built_output, check_missing_images, check_no_secrets, check_owners,
    check_registry_vs_defs, check_release_checklist, check_runtime_downloads,
    check_smoke_contract_lock, check_time_locale_determinism, check_tool_container_coverage,
    check_tool_id_contract, check_tool_invocation_normalization, check_tool_name_collision,
    container_artifact_dir, ensure_no_args, env_or_default, env_or_empty, lock_items_by_tool,
    metadata, path_from_arg, primary_tools_csv, read_json, read_utf8, registry_tool_rows,
    resolved_smoke_tools, run_bijux_with_env, run_runtime_smoke_contract, success_line, summary,
    versioning, write_ensure_images_plan_report, write_utf8, write_vuln_hook_report,
    ContainerCommandOutcome, Result, Workspace,
};

pub(in super::super::super) fn run_registry_tools(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    if matches!(args, [single] if single == "--help" || single == "-h") {
        return success_line(
            "Usage: cargo run -p bijux-dna-dev -- containers run registry-tools -- <registry-subcommand> [args...]",
        );
    }
    if args.is_empty() {
        return Ok(ContainerCommandOutcome::failure(
            "registry-tools: missing registry subcommand\n",
        ));
    }
    let mut command_args = vec!["registry".to_string()];
    command_args.extend(args.iter().cloned());
    run_bijux_with_env(workspace, &command_args, &[])
}

pub(in super::super::super) fn run_container_lint(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    ensure_no_args("lint", args)?;
    let mut aggregate = ContainerCommandOutcome::success(String::new());

    append_named_outcome(
        &mut aggregate,
        "check-tool-id-manifest",
        metadata::check_tool_id_manifest(workspace)?,
    );
    append_named_outcome(
        &mut aggregate,
        "check-tool-name-map-generated",
        metadata::check_tool_name_map_generated(workspace)?,
    );
    append_named_outcome(
        &mut aggregate,
        "check-index",
        metadata::check_container_index(workspace)?,
    );
    append_named_outcome(
        &mut aggregate,
        "check-license-metadata",
        metadata::check_license_metadata(workspace)?,
    );
    append_named_outcome(
        &mut aggregate,
        "check-license-index-generated",
        metadata::check_license_index_generated(workspace)?,
    );
    append_named_outcome(
        &mut aggregate,
        "check-qa-matrix-generated",
        metadata::check_qa_matrix_generated(workspace)?,
    );
    append_named_outcome(
        &mut aggregate,
        "check-tool-docs-generated",
        metadata::check_tool_docs_generated(workspace)?,
    );
    append_named_outcome(
        &mut aggregate,
        "check-network-disclosure",
        metadata::check_network_disclosure(workspace, &[])?,
    );
    append_named_outcome(
        &mut aggregate,
        "check-version-lock",
        versioning::check_version_lock(workspace)?,
    );
    append_named_outcome(
        &mut aggregate,
        "check-version-authority",
        versioning::check_version_authority(workspace)?,
    );
    append_named_outcome(
        &mut aggregate,
        "check-lock-schema",
        versioning::check_lock_schema(workspace)?,
    );
    append_named_outcome(
        &mut aggregate,
        "check-version-completeness",
        versioning::check_version_completeness(workspace)?,
    );
    append_named_outcome(
        &mut aggregate,
        "check-version-hash-pin",
        versioning::check_version_hash_pin(workspace)?,
    );
    append_named_outcome(&mut aggregate, "check-owners", check_owners(workspace)?);
    append_named_outcome(
        &mut aggregate,
        "check-tool-name-collision",
        check_tool_name_collision(workspace)?,
    );
    append_named_outcome(
        &mut aggregate,
        "check-tool-id-contract",
        check_tool_id_contract(workspace)?,
    );
    append_named_outcome(&mut aggregate, "check-docker-context", check_docker_context(workspace)?);
    append_named_outcome(
        &mut aggregate,
        "check-docker-hardening",
        check_docker_hardening(workspace)?,
    );
    append_named_outcome(&mut aggregate, "check-docker-labels", check_docker_labels(workspace)?);
    append_named_outcome(
        &mut aggregate,
        "check-docker-unpinned-apt",
        check_docker_unpinned_apt(workspace)?,
    );
    append_named_outcome(
        &mut aggregate,
        "check-docker-version-sync",
        check_docker_version_sync(workspace)?,
    );
    append_named_outcome(
        &mut aggregate,
        "check-apptainer-hardening",
        check_apptainer_hardening(workspace)?,
    );
    append_named_outcome(
        &mut aggregate,
        "check-apptainer-post-pins",
        check_apptainer_post_pins(workspace)?,
    );
    append_named_outcome(
        &mut aggregate,
        "check-apptainer-version-label-sync",
        check_apptainer_version_label_sync(workspace)?,
    );
    append_named_outcome(&mut aggregate, "check-no-secrets", check_no_secrets(workspace)?);
    append_named_outcome(
        &mut aggregate,
        "check-runtime-downloads",
        check_runtime_downloads(workspace)?,
    );
    append_named_outcome(
        &mut aggregate,
        "check-time-locale-determinism",
        check_time_locale_determinism(workspace)?,
    );
    append_named_outcome(
        &mut aggregate,
        "check-tool-invocation-normalization",
        check_tool_invocation_normalization(workspace)?,
    );
    append_named_outcome(
        &mut aggregate,
        "check-hpc-image-naming",
        check_hpc_image_naming(workspace, &[])?,
    );
    append_named_outcome(
        &mut aggregate,
        "check-hpc-frontend-policy-enforcement",
        check_hpc_frontend_policy_enforcement(workspace)?,
    );

    Ok(aggregate)
}

pub(in super::super::super) fn run_ensure_images(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    let usage =
        "Usage: cargo run -p bijux-dna-dev -- containers run ensure-images -- [--plan] [--only <tool-id>] [--changed]";
    if matches!(args, [single] if single == "--help" || single == "-h") {
        return success_line(usage);
    }
    let mut plan_only = false;
    let mut changed_only = false;
    let mut only_tool = None::<String>;
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--plan" => {
                plan_only = true;
                index += 1;
            }
            "--changed" => {
                changed_only = true;
                index += 1;
            }
            "--only" => {
                let value =
                    args.get(index + 1).ok_or_else(|| anyhow!("--only requires <tool-id>"))?;
                only_tool = Some(value.clone());
                index += 2;
            }
            other => return Err(anyhow!("unknown arg for ensure-images: {other}\n{usage}")),
        }
    }
    if only_tool.is_some() && changed_only {
        return Ok(ContainerCommandOutcome::failure(
            "ensure-images: --only and --changed are mutually exclusive\n",
        ));
    }

    write_ensure_images_plan_report(workspace)?;
    let report = workspace.path("artifacts/containers/ensure-images/report.json");
    if plan_only {
        return success_line(format!("ensure-images: wrote {}", report.display()));
    }

    let tools = if let Some(tool) = only_tool { tool } else { primary_tools_csv(workspace)? };
    let smoke = run_runtime_smoke_contract(workspace, "apptainer", tools)?;
    let mut aggregate = ContainerCommandOutcome::success(String::new());
    append_named_outcome(&mut aggregate, "smoke-containers-apptainer", smoke);
    append_named_outcome(
        &mut aggregate,
        "generate-version-lock",
        versioning::generate_version_lock(workspace, &[])?,
    );
    append_named_outcome(
        &mut aggregate,
        "check-hpc-image-naming",
        check_hpc_image_naming(workspace, &[])?,
    );

    let lock_sha_path = workspace.path("configs/ci/registry/tool_registry_lock.sha256");
    let snapshot = workspace.path("artifacts/containers/ensure-images/last_lock.sha256");
    if lock_sha_path.is_file() {
        let sha = read_utf8(&lock_sha_path)?;
        write_utf8(&snapshot, sha.trim())?;
    }
    if changed_only && aggregate.is_success() {
        aggregate.stdout.push_str(
            "ensure-images: changed selection resolved through the governed primary tool set\n",
        );
    }
    Ok(aggregate)
}

pub(in super::super::super) fn run_container_doctor(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    let usage =
        "Usage: cargo run -p bijux-dna-dev -- containers run container-doctor -- [--strict] [--tool <tool-id>]";
    if matches!(args, [single] if single == "--help" || single == "-h") {
        return success_line(usage);
    }
    let mut strict = false;
    let mut tool = None::<String>;
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--strict" => {
                strict = true;
                index += 1;
            }
            "--tool" => {
                let value =
                    args.get(index + 1).ok_or_else(|| anyhow!("--tool requires <tool-id>"))?;
                tool = Some(value.clone());
                index += 2;
            }
            other => return Err(anyhow!("unknown arg for container-doctor: {other}\n{usage}")),
        }
    }

    if let Some(tool_id) = tool {
        let registry_entry = registry_tool_rows(workspace)?
            .into_iter()
            .find(|row| row.get("id").and_then(toml::Value::as_str) == Some(tool_id.as_str()))
            .map_or_else(|| toml::Value::Table(Default::default()), toml::Value::Table);
        let version_lock = lock_items_by_tool(workspace)?
            .remove(&tool_id)
            .unwrap_or_else(|| serde_json::json!({}));
        let smoke_summary_path =
            workspace.path("artifacts/containers/hpc/frontend-smoke/summary.json");
        let smoke = if smoke_summary_path.is_file() {
            read_json(&smoke_summary_path)?
                .get("items")
                .and_then(serde_json::Value::as_array)
                .and_then(|items| {
                    items.iter().find(|row| {
                        row.get("tool").and_then(serde_json::Value::as_str)
                            == Some(tool_id.as_str())
                    })
                })
                .cloned()
                .unwrap_or_else(|| serde_json::json!({}))
        } else {
            serde_json::json!({})
        };
        return Ok(ContainerCommandOutcome::success(format!(
            "{}\n",
            serde_json::to_string_pretty(&serde_json::json!({
                "schema_version": "bijux.container.doctor.tool.v2",
                "tool": tool_id,
                "registry": registry_entry,
                "version_lock": version_lock,
                "smoke": smoke,
            }))?
        )));
    }

    let mut aggregate = ContainerCommandOutcome::success(String::new());
    let mut items = Vec::new();
    for (name, outcome) in [
        ("missing_images", check_missing_images(workspace)?),
        ("lock_file_drift", versioning::check_version_lock(workspace)?),
        ("lock_vs_built", check_lock_matches_built_output(workspace)?),
        ("outdated_versions", versioning::check_version_deprecations(workspace)?),
        ("domain_parity", check_tool_container_coverage(workspace)?),
        ("registry_orphans", check_registry_vs_defs(workspace)?),
    ] {
        items.push(serde_json::json!({
            "id": name,
            "status": if outcome.is_success() { "ok" } else { "fail" },
            "detail": if outcome.is_success() {
                outcome.stdout.trim()
            } else {
                outcome.stderr.trim()
            },
        }));
        append_named_outcome(&mut aggregate, name, outcome);
    }
    let report = workspace.path("artifacts/containers/doctor/report.json");
    write_utf8(
        &report,
        &format!(
            "{}\n",
            serde_json::to_string_pretty(&serde_json::json!({
                "schema_version": "bijux.container.doctor.v2",
                "strict": strict,
                "items": items,
            }))?
        ),
    )?;
    if strict && !aggregate.is_success() {
        return Ok(aggregate);
    }
    aggregate.stdout.push_str(&format!("container-doctor: wrote {}\n", report.display()));
    Ok(aggregate)
}

pub(in super::super::super) fn run_release_gate(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    ensure_no_args("release-gate", args)?;
    let mut aggregate = ContainerCommandOutcome::success(String::new());
    append_named_outcome(&mut aggregate, "lint", run_container_lint(workspace, &[])?);
    append_named_outcome(
        &mut aggregate,
        "ensure-images",
        run_ensure_images(workspace, &[String::from("--plan")])?,
    );
    append_named_outcome(
        &mut aggregate,
        "container-doctor",
        run_container_doctor(workspace, &[String::from("--strict")])?,
    );
    append_named_outcome(
        &mut aggregate,
        "check-release-checklist",
        check_release_checklist(workspace)?,
    );
    Ok(aggregate)
}

pub(in super::super::super) fn run_vuln_scan_hook(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    let usage =
        "Usage: cargo run -p bijux-dna-dev -- containers run vuln-scan-hook -- [<sbom-root> [<output-path>]]";
    if matches!(args, [single] if single == "--help" || single == "-h") {
        return success_line(usage);
    }
    let sbom_root =
        args.first().map(|value| path_from_arg(workspace, value)).unwrap_or_else(|| {
            artifact_root_path(workspace)
                .unwrap_or_else(|_| workspace.path("artifacts"))
                .join("containers/sbom")
        });
    let out = args.get(1).map(|value| path_from_arg(workspace, value)).unwrap_or_else(|| {
        artifact_root_path(workspace)
            .unwrap_or_else(|_| workspace.path("artifacts"))
            .join("containers/vuln_scan_report.json")
    });
    let toolkit = env_or_empty("TOOLKIT");
    let promoted_only = env_or_default("PROMOTED_ONLY", "1") != "0";
    write_vuln_hook_report(workspace, &sbom_root, &out, &toolkit, promoted_only)?;
    success_line(format!("vuln-scan-hook: wrote {}", out.display()))
}

pub(in super::super::super) fn run_apptainer_build_all(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    ensure_no_args("apptainer-build-all", args)?;
    let mut aggregate = ContainerCommandOutcome::success(String::new());
    append_named_outcome(
        &mut aggregate,
        "smoke-apptainer",
        run_runtime_smoke_contract(workspace, "apptainer", resolved_smoke_tools(workspace)?)?,
    );
    let summary_rel = format!("{}/hpc/frontend-smoke/summary.json", container_artifact_dir());
    let summary_path = workspace.path(&summary_rel);
    append_named_outcome(
        &mut aggregate,
        "summary",
        summary(workspace, &[String::from("--json"), summary_path.display().to_string()])?,
    );
    append_named_outcome(
        &mut aggregate,
        "generate-version-lock",
        versioning::generate_version_lock(workspace, &[])?,
    );
    append_named_outcome(
        &mut aggregate,
        "check-smoke-contract-lock",
        check_smoke_contract_lock(workspace)?,
    );
    Ok(aggregate)
}

pub(in super::super::super) fn run_docker_build_all(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    ensure_no_args("docker-build-all", args)?;
    let mut aggregate = ContainerCommandOutcome::success(String::new());
    append_named_outcome(
        &mut aggregate,
        "smoke-docker-arm64",
        run_runtime_smoke_contract(workspace, "docker-arm64", resolved_smoke_tools(workspace)?)?,
    );
    let summary_rel = format!("{}/summary.json", container_artifact_dir());
    let summary_path = workspace.path(&summary_rel);
    append_named_outcome(
        &mut aggregate,
        "summary",
        summary(workspace, &[String::from("--json"), summary_path.display().to_string()])?,
    );
    append_named_outcome(
        &mut aggregate,
        "generate-version-lock",
        versioning::generate_version_lock(
            workspace,
            &[workspace.path("containers/versions/lock.json").display().to_string()],
        )?,
    );
    append_named_outcome(
        &mut aggregate,
        "check-lock-matches-built-output",
        check_lock_matches_built_output(workspace)?,
    );
    Ok(aggregate)
}
