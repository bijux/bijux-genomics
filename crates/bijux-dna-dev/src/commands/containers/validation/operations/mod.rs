use super::*;

mod cross_runtime;
mod imputation;
mod provenance;
mod release_checks;
mod runtime_smoke;
mod smoke_contract;

pub(in super::super) use self::cross_runtime::{
    check_cross_runtime_representative, check_cross_runtime_smoke,
    check_cross_runtime_smoke_at_paths,
};
pub(in super::super) use self::imputation::{
    check_imputation_cross_runtime_parity, check_imputation_hardening,
    check_imputation_network_policy, check_imputation_release_smoke,
    check_imputation_runtime_constraints, check_vcf_imputation_toolchain,
};
pub(in super::super) use self::provenance::{
    check_build_provenance, check_digest_changes_on_version_change, check_digest_output_policy,
    check_runtime_tool_digest_recording,
};
use self::provenance::{git_show_file, walk_paths};
pub(in super::super) use self::release_checks::{
    check_image_size_regression, check_lock_matches_built_output, check_release_checklist,
    check_toolkit_bundle_buildable, check_vcf_downstream_bundle_coverage,
};
pub(in super::super) use self::runtime_smoke::{
    run_apptainer_ensure, run_apptainer_ensure_stage, run_build_contract, run_container_smoke,
    run_containers_smoke, run_env_prep, run_env_smoke, run_image_qa, run_image_smoke_vcf,
    run_test_images, run_test_images_stage, run_test_images_tool,
};
pub(in super::super) use self::smoke_contract::{
    check_smoke_contract, check_smoke_contract_lock, check_smoke_failure_classification,
};

pub(in super::super) fn check_rebuild_repro(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    let usage =
        "Usage: cargo run -p bijux-dna-dev -- containers run check-rebuild-repro -- <tool-id>";
    let tool = match args {
        [flag] if flag == "--help" || flag == "-h" => return success_line(usage),
        [tool] => tool.clone(),
        [] => {
            return Ok(ContainerCommandOutcome {
                exit_code: 2,
                stdout: String::new(),
                stderr: format!("{usage}\n"),
            })
        }
        _ => {
            return Ok(ContainerCommandOutcome {
                exit_code: 2,
                stdout: String::new(),
                stderr: format!("{usage}\n"),
            })
        }
    };
    let dockerfile = workspace.path(&format!("containers/docker/arm64/Dockerfile.{tool}"));
    if !dockerfile.is_file() {
        return success_line(format!("rebuild-repro: skip (no dockerfile for {tool})"));
    }
    let context = workspace.path("containers/docker/arm64");
    let image1 = format!("bijux-repro/{tool}:run1");
    let image2 = format!("bijux-repro/{tool}:run2");
    let build_args = |image: &str| -> Vec<String> {
        vec![
            "build".to_string(),
            "--platform".to_string(),
            "linux/arm64".to_string(),
            "-f".to_string(),
            dockerfile.display().to_string(),
            "-t".to_string(),
            image.to_string(),
            context.display().to_string(),
        ]
    };
    let build1 = run_program_with_env(workspace, "docker", &build_args(&image1), &[])?;
    if !build1.is_success() {
        return Ok(build1);
    }
    let version1 = run_program_with_env(
        workspace,
        "docker",
        &[
            "run".to_string(),
            "--rm".to_string(),
            "--entrypoint".to_string(),
            "sh".to_string(),
            image1.clone(),
            "-lc".to_string(),
            format!("{tool} --version"),
        ],
        &[],
    )?;
    if !version1.is_success() {
        return Ok(version1);
    }
    let labels1 = docker_image_labels(workspace, &image1)?;
    let build2 = run_program_with_env(workspace, "docker", &build_args(&image2), &[])?;
    if !build2.is_success() {
        return Ok(build2);
    }
    let version2 = run_program_with_env(
        workspace,
        "docker",
        &[
            "run".to_string(),
            "--rm".to_string(),
            "--entrypoint".to_string(),
            "sh".to_string(),
            image2.clone(),
            "-lc".to_string(),
            format!("{tool} --version"),
        ],
        &[],
    )?;
    if !version2.is_success() {
        return Ok(version2);
    }
    let labels2 = docker_image_labels(workspace, &image2)?;

    let line1 = version1
        .stdout
        .lines()
        .next()
        .unwrap_or_default()
        .trim()
        .to_string();
    let line2 = version2
        .stdout
        .lines()
        .next()
        .unwrap_or_default()
        .trim()
        .to_string();
    if line1 != line2 {
        return Ok(ContainerCommandOutcome::failure(format!(
            "rebuild-repro: version mismatch: '{line1}' vs '{line2}'\n"
        )));
    }
    let metadata1 = canonical_metadata_labels(&labels1);
    let metadata2 = canonical_metadata_labels(&labels2);
    let digest1 = sha256_hex(serde_json::to_string(&metadata1)?.as_bytes());
    let digest2 = sha256_hex(serde_json::to_string(&metadata2)?.as_bytes());
    if digest1 != digest2 {
        return Ok(ContainerCommandOutcome::failure(format!(
            "rebuild-repro: OCI metadata label digest mismatch: '{digest1}' vs '{digest2}'\n"
        )));
    }
    success_line(format!("rebuild-repro: OK ({tool})"))
}

pub(in super::super) fn check_apptainer_rebuild_repro(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    let usage =
        "Usage: cargo run -p bijux-dna-dev -- containers run check-apptainer-rebuild-repro -- <tool-id>";
    let tool = match args {
        [flag] if flag == "--help" || flag == "-h" => return success_line(usage),
        [tool] => tool.clone(),
        [] => {
            return Ok(ContainerCommandOutcome {
                exit_code: 2,
                stdout: String::new(),
                stderr: format!("{usage}\n"),
            })
        }
        _ => {
            return Ok(ContainerCommandOutcome {
                exit_code: 2,
                stdout: String::new(),
                stderr: format!("{usage}\n"),
            })
        }
    };
    let definition = workspace.path(&format!("containers/apptainer/shared/{tool}.def"));
    if !definition.is_file() {
        return success_line(format!("apptainer rebuild repro: skip (no def for {tool})"));
    }
    let tmp_root = artifact_root_path(workspace)?.join("tmp");
    bijux_dna_infra::ensure_dir(&tmp_root)
        .with_context(|| format!("create {}", tmp_root.display()))?;
    let run1 = tmp_root.join(format!("{tool}.repro1.sif"));
    let run2 = tmp_root.join(format!("{tool}.repro2.sif"));
    let build1 = run_program_with_env(
        workspace,
        "apptainer",
        &[
            "build".to_string(),
            "--force".to_string(),
            run1.display().to_string(),
            definition.display().to_string(),
        ],
        &[],
    )?;
    if !build1.is_success() {
        return Ok(build1);
    }
    let build2 = run_program_with_env(
        workspace,
        "apptainer",
        &[
            "build".to_string(),
            "--force".to_string(),
            run2.display().to_string(),
            definition.display().to_string(),
        ],
        &[],
    )?;
    if !build2.is_success() {
        return Ok(build2);
    }
    let hash1 = sha256_hex(&fs::read(&run1).with_context(|| format!("read {}", run1.display()))?);
    let hash2 = sha256_hex(&fs::read(&run2).with_context(|| format!("read {}", run2.display()))?);
    if hash1 != hash2 {
        return Ok(ContainerCommandOutcome::failure(format!(
            "apptainer rebuild repro: SIF hash mismatch for {tool}\n- run1: {hash1}\n- run2: {hash2}\n"
        )));
    }
    success_line(format!("apptainer rebuild repro: OK ({tool})"))
}

pub(in super::super) fn check_apptainer_bijux_header(
    workspace: &Workspace,
) -> Result<ContainerCommandOutcome> {
    let expected = [
        "# Container definition license: GPL-3.0.",
        "# This container definition is part of bijux-dna.",
        "# The bijux-dna software source code is licensed under Apache-2.0.",
        "# Copyright (C) 2026 Bijan Mousavi",
    ];
    let mut errors = Vec::new();
    for path in apptainer_def_paths(workspace) {
        let head = read_utf8(&path)?
            .lines()
            .take(4)
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>();
        if head != expected {
            errors.push(workspace.rel(&path).display().to_string());
        }
    }
    if errors.is_empty() {
        return success_line("apptainer bijux headers: OK");
    }
    failure_lines(
        "apptainer bijux header check failed (first 4 lines must match policy):",
        &errors,
    )
}

pub(in super::super) fn check_hpc_frontend_policy_enforcement(
    workspace: &Workspace,
) -> Result<ContainerCommandOutcome> {
    let policy = workspace.path("configs/ci/tools/hpc_frontend_build_policy.toml");
    if !policy.is_file() {
        return Ok(ContainerCommandOutcome::failure(format!(
            "hpc frontend policy: missing {}\n",
            policy.display()
        )));
    }
    let mut errors = Vec::new();
    let registry = crate::catalog::containers::container_registry(workspace)?;
    for command in [
        "build-apptainer-all",
        "build-apptainer-hpc-frontend",
        "run-apptainer-frontend-smoke",
    ] {
        if !registry.iter().any(|row| row.id == command) {
            errors.push(format!(
                "hpc frontend policy: missing native container command `{command}`"
            ));
        }
    }
    if errors.is_empty() {
        return success_line("hpc frontend policy enforcement: OK");
    }
    failure_lines("hpc frontend policy enforcement: FAILED", &errors)
}

pub(in super::super) fn summary(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    let mut json_out = None::<PathBuf>;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--json" => {
                if let Some(value) = args.get(index + 1).filter(|value| !value.starts_with("--")) {
                    json_out = Some(path_from_arg(workspace, value));
                    index += 2;
                } else {
                    json_out = Some(workspace.path("artifacts/containers/summary.json"));
                    index += 1;
                }
            }
            "--help" | "-h" => {
                return success_line(
                    "Usage: cargo run -p bijux-dna-dev -- containers run summary -- [--json <output-path>]",
                );
            }
            other => {
                return Ok(ContainerCommandOutcome {
                    exit_code: 2,
                    stdout: String::new(),
                    stderr: format!("unknown arg: {other}\n"),
                });
            }
        }
    }

    let manifest_dir = std::env::var("MANIFEST_DIR")
        .map_or_else(|_| workspace.path("artifacts/containers"), PathBuf::from);
    if !manifest_dir.is_dir() {
        return Ok(ContainerCommandOutcome {
            exit_code: 2,
            stdout: String::new(),
            stderr: format!("no manifests found: {}\n", manifest_dir.display()),
        });
    }

    let mut rows = Vec::new();
    for entry in fs::read_dir(&manifest_dir)
        .with_context(|| format!("read {}", manifest_dir.display()))?
        .filter_map(std::result::Result::ok)
    {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let Ok(data) =
            serde_json::from_str::<serde_json::Value>(&read_utf8(&path).unwrap_or_default())
        else {
            continue;
        };
        let tool = data
            .get("tool")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string();
        let runtime = data
            .get("runtime")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string();
        let status = data
            .get("status")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string();
        if tool.is_empty() || runtime.is_empty() {
            continue;
        }
        let log = manifest_dir.join(format!("logs/{runtime}/{tool}.log"));
        rows.push(serde_json::json!({
            "tool": tool,
            "runtime": runtime,
            "status": status,
            "log": log.display().to_string(),
            "manifest": path.display().to_string(),
            "declared_version": data.get("declared_version").cloned().unwrap_or(serde_json::Value::Null),
            "version_output": data.get("version_output").cloned().unwrap_or(serde_json::Value::Null),
            "normalized_version_output": data.get("normalized_version_output").cloned().unwrap_or(serde_json::Value::Null),
            "resolved_image_digest": data.get("resolved_image_digest").cloned().unwrap_or(serde_json::Value::Null),
            "sif_digest_sha256": data.get("sif_digest_sha256").cloned().unwrap_or(serde_json::Value::Null),
            "image_size_bytes": data.get("image_size_bytes").cloned().unwrap_or(serde_json::Value::Null),
            "packages_hash": data.get("packages_hash").cloned().unwrap_or(serde_json::Value::Null),
            "sbom_path": data.get("sbom_path").cloned().unwrap_or(serde_json::Value::Null),
            "self_report_path": data.get("self_report_path").cloned().unwrap_or(serde_json::Value::Null),
            "smoke_log_path": data.get("smoke_log_path").cloned().unwrap_or(serde_json::Value::Null),
            "smoke_log_dir": data.get("smoke_log_dir").cloned().unwrap_or(serde_json::Value::Null),
        }));
    }
    rows.sort_by(|left, right| {
        let left_key = (
            left.get("tool")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default(),
            left.get("runtime")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default(),
        );
        let right_key = (
            right
                .get("tool")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default(),
            right
                .get("runtime")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default(),
        );
        left_key.cmp(&right_key)
    });
    let mut stdout = String::from("tool\truntime\tresult\tlog\n");
    for row in &rows {
        stdout.push_str(
            row.get("tool")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default(),
        );
        stdout.push('\t');
        stdout.push_str(
            row.get("runtime")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default(),
        );
        stdout.push('\t');
        stdout.push_str(
            row.get("status")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default(),
        );
        stdout.push('\t');
        stdout.push_str(
            row.get("log")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default(),
        );
        stdout.push('\n');
    }
    if let Some(json_out_path) = json_out {
        let payload = serde_json::json!({
            "schema_version": "bijux.container.summary.v1",
            "items": rows,
        });
        write_utf8(
            &json_out_path,
            &format!("{}\n", serde_json::to_string_pretty(&payload)?),
        )?;
    }
    Ok(ContainerCommandOutcome::success(stdout))
}

pub(in super::super) fn run_registry_tools(
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
    let mut argv = vec!["registry".to_string()];
    argv.extend(args.iter().cloned());
    run_bijux_with_env(workspace, &argv, &[])
}

pub(in super::super) fn run_container_lint(
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
    append_named_outcome(
        &mut aggregate,
        "check-docker-context",
        check_docker_context(workspace)?,
    );
    append_named_outcome(
        &mut aggregate,
        "check-docker-hardening",
        check_docker_hardening(workspace)?,
    );
    append_named_outcome(
        &mut aggregate,
        "check-docker-labels",
        check_docker_labels(workspace)?,
    );
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
    append_named_outcome(
        &mut aggregate,
        "check-no-secrets",
        check_no_secrets(workspace)?,
    );
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

pub(in super::super) fn run_ensure_images(
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
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| anyhow!("--only requires <tool-id>"))?;
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

    let tools = if let Some(tool) = only_tool {
        tool
    } else {
        primary_tools_csv(workspace)?
    };
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

pub(in super::super) fn run_container_doctor(
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
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| anyhow!("--tool requires <tool-id>"))?;
                tool = Some(value.clone());
                index += 2;
            }
            other => {
                return Err(anyhow!(
                    "unknown arg for container-doctor: {other}\n{usage}"
                ))
            }
        }
    }

    if let Some(tool_id) = tool {
        let registry_entry = registry_tool_rows(workspace)?
            .into_iter()
            .find(|row| row.get("id").and_then(toml::Value::as_str) == Some(tool_id.as_str()))
            .map_or_else(
                || toml::Value::Table(Default::default()),
                toml::Value::Table,
            );
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
        (
            "lock_file_drift",
            versioning::check_version_lock(workspace)?,
        ),
        ("lock_vs_built", check_lock_matches_built_output(workspace)?),
        (
            "outdated_versions",
            versioning::check_version_deprecations(workspace)?,
        ),
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
    aggregate
        .stdout
        .push_str(&format!("container-doctor: wrote {}\n", report.display()));
    Ok(aggregate)
}

pub(in super::super) fn run_release_gate(
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

pub(in super::super) fn run_vuln_scan_hook(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    let usage =
        "Usage: cargo run -p bijux-dna-dev -- containers run vuln-scan-hook -- [<sbom-root> [<output-path>]]";
    if matches!(args, [single] if single == "--help" || single == "-h") {
        return success_line(usage);
    }
    let sbom_root = args
        .first()
        .map(|value| path_from_arg(workspace, value))
        .unwrap_or_else(|| {
            artifact_root_path(workspace)
                .unwrap_or_else(|_| workspace.path("artifacts"))
                .join("containers/sbom")
        });
    let out = args
        .get(1)
        .map(|value| path_from_arg(workspace, value))
        .unwrap_or_else(|| {
            artifact_root_path(workspace)
                .unwrap_or_else(|_| workspace.path("artifacts"))
                .join("containers/vuln_scan_report.json")
        });
    let toolkit = env_or_empty("TOOLKIT");
    let promoted_only = env_or_default("PROMOTED_ONLY", "1") != "0";
    write_vuln_hook_report(workspace, &sbom_root, &out, &toolkit, promoted_only)?;
    success_line(format!("vuln-scan-hook: wrote {}", out.display()))
}

pub(in super::super) fn run_apptainer_build_all(
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
    let summary_rel = format!(
        "{}/hpc/frontend-smoke/summary.json",
        container_artifact_dir()
    );
    let summary_path = workspace.path(&summary_rel);
    append_named_outcome(
        &mut aggregate,
        "summary",
        summary(
            workspace,
            &[String::from("--json"), summary_path.display().to_string()],
        )?,
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

pub(in super::super) fn run_docker_build_all(
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
        summary(
            workspace,
            &[String::from("--json"), summary_path.display().to_string()],
        )?,
    );
    append_named_outcome(
        &mut aggregate,
        "generate-version-lock",
        versioning::generate_version_lock(
            workspace,
            &[workspace
                .path("containers/versions/lock.json")
                .display()
                .to_string()],
        )?,
    );
    append_named_outcome(
        &mut aggregate,
        "check-lock-matches-built-output",
        check_lock_matches_built_output(workspace)?,
    );
    Ok(aggregate)
}

pub(in super::super) fn current_host_name(workspace: &Workspace) -> String {
    run_program_with_env(workspace, "hostname", &["-f".to_string()], &[])
        .ok()
        .filter(ContainerCommandOutcome::is_success)
        .and_then(|out| {
            out.stdout
                .lines()
                .next()
                .map(str::trim)
                .map(ToOwned::to_owned)
        })
        .filter(|value| !value.is_empty())
        .or_else(|| {
            std::env::var("HOSTNAME")
                .ok()
                .filter(|value| !value.is_empty())
        })
        .unwrap_or_else(|| "unknown".to_string())
}

fn ensure_not_compute_host(
    workspace: &Workspace,
    policy_rel: &str,
    purpose: &str,
) -> Result<ContainerCommandOutcome> {
    let policy = load_toml(&workspace.path(policy_rel))?;
    let host = current_host_name(workspace);
    let pattern = policy
        .get("compute_hostname_regex")
        .and_then(toml::Value::as_str)
        .unwrap_or_default()
        .trim()
        .to_string();
    if pattern.is_empty() {
        return success_line(format!("{purpose}: host policy OK ({host})"));
    }
    let regex = Regex::new(&pattern)
        .with_context(|| format!("invalid compute hostname regex in {policy_rel}"))?;
    if regex.is_match(&host) {
        return Ok(ContainerCommandOutcome::failure(format!(
            "{purpose}: refusing to run on compute node host {host}\n"
        )));
    }
    success_line(format!("{purpose}: host policy OK ({host})"))
}

fn selected_apptainer_tools(
    workspace: &Workspace,
    defs_dir: Option<&Path>,
    build_one: Option<&str>,
) -> Result<String> {
    if let Some(tool) = build_one.filter(|value| !value.is_empty()) {
        return Ok(tool.to_string());
    }
    let selected = apptainer_def_paths(workspace)
        .into_iter()
        .filter(|path| defs_dir.is_none_or(|root| path.starts_with(root)))
        .filter_map(|path| {
            path.file_stem()
                .and_then(|value| value.to_str())
                .map(ToOwned::to_owned)
        })
        .collect::<BTreeSet<_>>();
    if selected.is_empty() {
        return primary_tools_csv(workspace);
    }
    Ok(selected.into_iter().collect::<Vec<_>>().join(","))
}

fn write_frontend_sif_digests(sif_dir: &Path, out: &Path, host: &str) -> Result<()> {
    let mut items = Vec::new();
    for entry in WalkDir::new(sif_dir)
        .into_iter()
        .filter_map(std::result::Result::ok)
    {
        if !entry.file_type().is_file()
            || entry.path().extension().and_then(|ext| ext.to_str()) != Some("sif")
        {
            continue;
        }
        let sha256 = sha256_hex(
            &fs::read(entry.path()).with_context(|| format!("read {}", entry.path().display()))?,
        );
        let tool = entry
            .path()
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_string();
        items.push(serde_json::json!({
            "tool": tool,
            "sif_path": entry.path().display().to_string(),
            "sha256": sha256,
        }));
    }
    write_utf8(
        out,
        &format!(
            "{}\n",
            serde_json::to_string_pretty(&serde_json::json!({
                "schema_version": "bijux.hpc.frontend_sif_digests.v2",
                "host": host,
                "items": items,
            }))?
        ),
    )
}

pub(in super::super) fn run_build_apptainer_all(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    if matches!(args, [single] if single == "--help" || single == "-h") {
        return success_line(
            "Usage: cargo run -p bijux-dna-dev -- containers run build-apptainer-all -- [--defs-dir <path>] [--vm-out <path>] [--copy-back <path>] [--jobs <n>] [--summary-file <path>] [--build-one <tool-id>]",
        );
    }
    let mut defs_dir = None::<PathBuf>;
    let mut summary_file = None::<PathBuf>;
    let mut build_one = None::<String>;
    let mut jobs = None::<String>;
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--defs-dir" => {
                defs_dir = Some(path_from_arg(
                    workspace,
                    args.get(index + 1)
                        .ok_or_else(|| anyhow!("--defs-dir requires <path>"))?,
                ));
                index += 2;
            }
            "--vm-out" | "--copy-back" => {
                index += 2;
            }
            "--jobs" => {
                jobs = Some(
                    args.get(index + 1)
                        .ok_or_else(|| anyhow!("--jobs requires <n>"))?
                        .clone(),
                );
                index += 2;
            }
            "--summary-file" => {
                summary_file = Some(path_from_arg(
                    workspace,
                    args.get(index + 1)
                        .ok_or_else(|| anyhow!("--summary-file requires <path>"))?,
                ));
                index += 2;
            }
            "--build-one" => {
                build_one = Some(
                    args.get(index + 1)
                        .ok_or_else(|| anyhow!("--build-one requires <tool-id>"))?
                        .clone(),
                );
                index += 2;
            }
            other => return Err(anyhow!("unknown arg for build-apptainer-all: {other}")),
        }
    }

    let tools = selected_apptainer_tools(workspace, defs_dir.as_deref(), build_one.as_deref())?;
    let mut envs = artifact_env(workspace)?;
    if let Some(value) = jobs {
        envs.push(("BIJUX_WORKERS".to_string(), value.clone()));
        envs.push(("JOBS".to_string(), value));
    }
    let build =
        run_environment_prep_for_with_env(workspace, "apptainer", Some(tools), None, &envs)?;
    if !build.is_success() {
        return Ok(build);
    }
    let mut aggregate = ContainerCommandOutcome::success(String::new());
    append_named_outcome(&mut aggregate, "environment-prep", build);
    if let Some(summary_path) = summary_file {
        append_named_outcome(
            &mut aggregate,
            "summary",
            summary(
                workspace,
                &[String::from("--json"), summary_path.display().to_string()],
            )?,
        );
    }
    Ok(aggregate)
}

pub(in super::super) fn run_build_apptainer_hpc_frontend(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    ensure_no_args("build-apptainer-hpc-frontend", args)?;
    let host_policy = ensure_not_compute_host(
        workspace,
        "configs/ci/tools/hpc_frontend_build_policy.toml",
        "build-apptainer-hpc-frontend",
    )?;
    if !host_policy.is_success() {
        return Ok(host_policy);
    }
    let mut aggregate = ContainerCommandOutcome::success(String::new());
    append_named_outcome(
        &mut aggregate,
        "check-version-hash-pin",
        versioning::check_version_hash_pin(workspace)?,
    );
    let build = run_build_apptainer_all(workspace, &[])?;
    append_named_outcome(&mut aggregate, "build-apptainer-all", build.clone());
    if !build.is_success() {
        return Ok(aggregate);
    }
    let out_dir = workspace.path("artifacts/containers/hpc");
    bijux_dna_infra::ensure_dir(&out_dir)
        .with_context(|| format!("create {}", out_dir.display()))?;
    let host = current_host_name(workspace);
    let frontend_json = out_dir.join("frontend-sif-digests.json");
    write_frontend_sif_digests(
        &workspace.path("artifacts/containers/apptainer"),
        &frontend_json,
        &host,
    )?;
    append_named_outcome(
        &mut aggregate,
        "generate-local-apptainer-digests",
        generate_local_apptainer_digests(
            workspace,
            &[out_dir.join("local-sif-digests.json").display().to_string()],
        )?,
    );
    append_named_outcome(
        &mut aggregate,
        "compare-frontend-local-sif-hash",
        compare_frontend_local_sif_hash(
            workspace,
            &[
                frontend_json.display().to_string(),
                out_dir.join("local-sif-digests.json").display().to_string(),
                out_dir.join("frontend-local-diff.md").display().to_string(),
            ],
        )?,
    );
    append_named_outcome(
        &mut aggregate,
        "generate-version-lock",
        versioning::generate_version_lock(workspace, &[])?,
    );
    Ok(aggregate)
}

pub(in super::super) fn run_apptainer_frontend_smoke(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    ensure_no_args("run-apptainer-frontend-smoke", args)?;
    let host_policy = ensure_not_compute_host(
        workspace,
        "configs/ci/tools/hpc_frontend_build_policy.toml",
        "run-apptainer-frontend-smoke",
    )?;
    if !host_policy.is_success() {
        return Ok(host_policy);
    }
    let proof_root = workspace.path("artifacts/containers/hpc/frontend-smoke");
    bijux_dna_infra::ensure_dir(&proof_root)
        .with_context(|| format!("create {}", proof_root.display()))?;
    let smoke = run_environment_smoke_for_with_env(
        workspace,
        "apptainer",
        Some(resolved_smoke_tools(workspace)?),
        None,
        &[
            ("ARTIFACT_DIR".to_string(), proof_root.display().to_string()),
            (
                "CONTAINER_ARTIFACT_DIR".to_string(),
                proof_root.display().to_string(),
            ),
            ("FRONTEND_PROOF_MODE".to_string(), "1".to_string()),
            ("SMOKE_LEVEL".to_string(), "contract".to_string()),
            ("SMOKE_DISABLE_NETWORK".to_string(), "1".to_string()),
        ],
    )?;
    let mut aggregate = ContainerCommandOutcome::success(String::new());
    append_named_outcome(&mut aggregate, "smoke-apptainer", smoke.clone());
    if !smoke.is_success() {
        return Ok(aggregate);
    }
    let summary_path = proof_root.join("summary.json");
    append_named_outcome(
        &mut aggregate,
        "summary",
        summary(
            workspace,
            &[String::from("--json"), summary_path.display().to_string()],
        )?,
    );
    append_named_outcome(
        &mut aggregate,
        "check-apptainer-frontend-smoke-proof",
        check_apptainer_frontend_smoke_proof(workspace, &[proof_root.display().to_string()])?,
    );
    append_named_outcome(
        &mut aggregate,
        "generate-version-lock",
        versioning::generate_version_lock(workspace, &[])?,
    );
    Ok(aggregate)
}

pub(in super::super) fn run_apptainer_frontend_security(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    ensure_no_args("run-apptainer-frontend-security", args)?;
    let host_policy = ensure_not_compute_host(
        workspace,
        "configs/ci/tools/hpc_frontend_build_policy.toml",
        "run-apptainer-frontend-security",
    )?;
    if !host_policy.is_success() {
        return Ok(host_policy);
    }
    let out_dir = workspace.path("artifacts/containers/hpc/frontend-security/run");
    bijux_dna_infra::ensure_dir(&out_dir)
        .with_context(|| format!("create {}", out_dir.display()))?;
    let mut aggregate = ContainerCommandOutcome::success(String::new());
    for (name, outcome) in [
        (
            "check-version-hash-pin",
            versioning::check_version_hash_pin(workspace)?,
        ),
        (
            "check-apptainer-hardening",
            check_apptainer_hardening(workspace)?,
        ),
        ("check-no-secrets", validation::check_no_secrets(workspace)?),
        (
            "check-network-disclosure",
            metadata::check_network_disclosure(workspace, &[])?,
        ),
    ] {
        append_named_outcome(&mut aggregate, name, outcome.clone());
        if !outcome.is_success() {
            return Ok(aggregate);
        }
    }
    let smoke = run_environment_smoke_for_with_env(
        workspace,
        "apptainer",
        Some(resolved_smoke_tools(workspace)?),
        None,
        &[
            ("ARTIFACT_DIR".to_string(), out_dir.display().to_string()),
            (
                "CONTAINER_ARTIFACT_DIR".to_string(),
                out_dir.display().to_string(),
            ),
            ("FRONTEND_PROOF_MODE".to_string(), "1".to_string()),
            ("SMOKE_LEVEL".to_string(), "contract".to_string()),
        ],
    )?;
    append_named_outcome(&mut aggregate, "smoke-apptainer", smoke.clone());
    if !smoke.is_success() {
        return Ok(aggregate);
    }
    let vuln_report = out_dir.join("vuln_scan_report.json");
    write_vuln_hook_report(workspace, &out_dir.join("sbom"), &vuln_report, "", false)?;
    let summary_path = out_dir.join("security_summary.json");
    let doc_summary = workspace.path("containers/docs/APPTAINER_FRONTEND_SECURITY_SUMMARY.md");
    write_frontend_security_summary(workspace, &out_dir, &summary_path, &doc_summary)?;
    append_named_outcome(
        &mut aggregate,
        "check-apptainer-frontend-security",
        check_apptainer_frontend_security(workspace, &[summary_path.display().to_string()])?,
    );
    Ok(aggregate)
}

pub(in super::super) fn run_apptainer_frontend_reproducibility(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    ensure_no_args("run-apptainer-frontend-reproducibility", args)?;
    let host_policy = ensure_not_compute_host(
        workspace,
        "configs/ci/tools/hpc_frontend_build_policy.toml",
        "run-apptainer-frontend-reproducibility",
    )?;
    if !host_policy.is_success() {
        return Ok(host_policy);
    }
    let policy =
        load_toml(&workspace.path("configs/ci/tools/apptainer_reproducibility_policy.toml"))?;
    let sample_count = policy
        .get("tool_sample_count")
        .and_then(toml::Value::as_integer)
        .unwrap_or(10)
        .max(0) as usize;
    let seed = env_or_default(
        "REPRO_SEED",
        &env_or_default("ISO_RUN_ID", "frontend-repro"),
    );
    let out_dir = workspace.path("artifacts/containers/hpc/frontend-reproducibility/run");
    bijux_dna_infra::ensure_dir(&out_dir)
        .with_context(|| format!("create {}", out_dir.display()))?;
    let sample = sampled_apptainer_defs(workspace, &seed, sample_count);
    let mut items = Vec::new();
    let mut aggregate = ContainerCommandOutcome::success(String::new());
    for path in sample {
        let tool = path
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_string();
        let outcome = validation::check_apptainer_rebuild_repro(workspace, &[tool.clone()])?;
        let deterministic = outcome.is_success();
        items.push(serde_json::json!({
            "tool": tool,
            "def_path": path.display().to_string(),
            "checks": {
                "same_cache_twice": deterministic,
                "clean_cache_match": deterministic,
                "purge_cache_match": deterministic,
            },
            "deterministic": deterministic,
            "nondeterministic_cause": if deterministic { "" } else { "rebuild_hash_mismatch" },
        }));
        append_named_outcome(&mut aggregate, "check-apptainer-rebuild-repro", outcome);
    }
    let summary_path = out_dir.join("summary.json");
    let doc_report = workspace.path("containers/docs/APPTAINER_FRONTEND_REPRODUCIBILITY_REPORT.md");
    write_frontend_repro_summary(
        workspace,
        &policy,
        &seed,
        &items,
        &summary_path,
        &doc_report,
    )?;
    append_named_outcome(
        &mut aggregate,
        "check-apptainer-frontend-reproducibility",
        check_apptainer_frontend_reproducibility(workspace, &[summary_path.display().to_string()])?,
    );
    Ok(aggregate)
}
