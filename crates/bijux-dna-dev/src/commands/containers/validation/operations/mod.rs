use super::{
    anyhow, append_named_outcome, apptainer_def_paths, artifact_env, artifact_root_path,
    bijux_command_prefix, canonical_metadata_labels, check_apptainer_frontend_reproducibility,
    check_apptainer_frontend_security, check_apptainer_frontend_smoke_proof,
    check_apptainer_hardening, check_apptainer_post_pins, check_apptainer_version_label_sync,
    check_docker_context, check_docker_hardening, check_docker_labels, check_docker_unpinned_apt,
    check_docker_version_sync, check_hpc_image_naming, check_missing_images, check_no_secrets,
    check_owners, check_registry_vs_defs, check_runtime_downloads, check_time_locale_determinism,
    check_tool_container_coverage, check_tool_id_contract, check_tool_invocation_normalization,
    check_tool_name_collision, checked_container_type, compare_frontend_local_sif_hash,
    container_artifact_dir, docker_image_labels, dockerfile_paths, ensure_no_args, env_or_default,
    env_or_empty, failure_lines, fs, generate_local_apptainer_digests, list_tools_for_stage,
    load_runtime_manifest_rows, load_toml, lock_items_by_tool, merge_outcomes, metadata,
    missing_container_label_markers, normalized_version_output, path_from_arg, primary_tools_csv,
    read_json, read_utf8, registry_tool_id, registry_tool_rows, require_tools_or_stage,
    resolved_smoke_tools, run_argv, run_bijux_with_env, run_environment_prep_for,
    run_environment_prep_for_with_env, run_environment_smoke_for,
    run_environment_smoke_for_with_env, run_program_with_env, run_runtime_smoke_contract,
    sampled_apptainer_defs, sha256_hex, success_line, table_array_strings, table_bool,
    table_string, validation, versioning, write_ensure_images_plan_report,
    write_frontend_repro_summary, write_frontend_security_summary, write_utf8,
    write_vuln_hook_report, BTreeMap, BTreeSet, ContainerCommandOutcome, Context, Path, PathBuf,
    ProcessRunner, Regex, Result, WalkDir, Workspace,
};

mod cross_runtime;
mod frontend_support;
mod frontend_workflows;
mod imputation;
mod orchestration;
mod provenance;
mod release_checks;
mod runtime_smoke;
mod smoke_contract;

pub(in super::super) use self::cross_runtime::{
    check_cross_runtime_representative, check_cross_runtime_smoke,
    check_cross_runtime_smoke_at_paths,
};
pub(in super::super) use self::frontend_support::current_host_name;
pub(in super::super) use self::frontend_workflows::{
    run_apptainer_frontend_reproducibility, run_apptainer_frontend_security,
    run_apptainer_frontend_smoke, run_build_apptainer_all, run_build_apptainer_hpc_frontend,
};
pub(in super::super) use self::imputation::{
    check_imputation_cross_runtime_parity, check_imputation_hardening,
    check_imputation_network_policy, check_imputation_release_smoke,
    check_imputation_runtime_constraints, check_vcf_imputation_toolchain,
};
pub(in super::super) use self::orchestration::{
    run_apptainer_build_all, run_container_doctor, run_container_lint, run_docker_build_all,
    run_ensure_images, run_registry_tools, run_release_gate, run_vuln_scan_hook,
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

    let line1 = version1.stdout.lines().next().unwrap_or_default().trim().to_string();
    let line2 = version2.stdout.lines().next().unwrap_or_default().trim().to_string();
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
        let head = read_utf8(&path)?.lines().take(4).map(ToOwned::to_owned).collect::<Vec<_>>();
        if head != expected {
            errors.push(workspace.rel(&path).display().to_string());
        }
    }
    if errors.is_empty() {
        return success_line("apptainer bijux headers: OK");
    }
    failure_lines("apptainer bijux header check failed (first 4 lines must match policy):", &errors)
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
    for command in
        ["build-apptainer-all", "build-apptainer-hpc-frontend", "run-apptainer-frontend-smoke"]
    {
        if !registry.iter().any(|row| row.id == command) {
            errors
                .push(format!("hpc frontend policy: missing native container command `{command}`"));
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
        let tool =
            data.get("tool").and_then(serde_json::Value::as_str).unwrap_or_default().to_string();
        let runtime =
            data.get("runtime").and_then(serde_json::Value::as_str).unwrap_or_default().to_string();
        let status =
            data.get("status").and_then(serde_json::Value::as_str).unwrap_or_default().to_string();
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
            left.get("tool").and_then(serde_json::Value::as_str).unwrap_or_default(),
            left.get("runtime").and_then(serde_json::Value::as_str).unwrap_or_default(),
        );
        let right_key = (
            right.get("tool").and_then(serde_json::Value::as_str).unwrap_or_default(),
            right.get("runtime").and_then(serde_json::Value::as_str).unwrap_or_default(),
        );
        left_key.cmp(&right_key)
    });
    let mut stdout = String::from("tool\truntime\tresult\tlog\n");
    for row in &rows {
        stdout.push_str(row.get("tool").and_then(serde_json::Value::as_str).unwrap_or_default());
        stdout.push('\t');
        stdout.push_str(row.get("runtime").and_then(serde_json::Value::as_str).unwrap_or_default());
        stdout.push('\t');
        stdout.push_str(row.get("status").and_then(serde_json::Value::as_str).unwrap_or_default());
        stdout.push('\t');
        stdout.push_str(row.get("log").and_then(serde_json::Value::as_str).unwrap_or_default());
        stdout.push('\n');
    }
    if let Some(json_out_path) = json_out {
        let payload = serde_json::json!({
            "schema_version": "bijux.container.summary.v1",
            "items": rows,
        });
        write_utf8(&json_out_path, &format!("{}\n", serde_json::to_string_pretty(&payload)?))?;
    }
    Ok(ContainerCommandOutcome::success(stdout))
}
