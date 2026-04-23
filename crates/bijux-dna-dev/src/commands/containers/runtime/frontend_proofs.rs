use super::{
    anyhow, apptainer_def_paths, command_exists, env_or_default, env_or_empty, failure_lines, fs,
    iso_root_path, iso_run_id, load_toml, lock_items_by_tool, lock_json_path, path_from_arg,
    policy_path, read_json, run_program_with_env, sha256_hex, success_line, table_string,
    tool_versions, validation, write_utf8, BTreeMap, BTreeSet, ContainerCommandOutcome, Context,
    Path, PathBuf, Result, WalkDir, Workspace,
};

pub(crate) fn check_apptainer_frontend_reproducibility(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    let usage = "Usage: cargo run -p bijux-dna-dev -- containers run check-apptainer-frontend-reproducibility -- [<summary-path>]";
    if matches!(args, [single] if single == "--help" || single == "-h") {
        return success_line(usage);
    }
    let summary_path = match args {
        [] => iso_root_path(workspace)
            .join("containers/hpc/frontend-reproducibility")
            .join(iso_run_id())
            .join("summary.json"),
        [path] => path_from_arg(workspace, path),
        _ => return Err(anyhow!(usage.to_string())),
    };
    if !summary_path.is_file() {
        if env_or_default("CI", "0") == "1" {
            return Ok(ContainerCommandOutcome::failure(format!(
                "frontend reproducibility check: missing summary in CI: {}\n",
                summary_path.display()
            )));
        }
        return success_line(format!(
            "frontend reproducibility check: SKIP (no summary at {})",
            summary_path.display()
        ));
    }
    let summary = read_json(&summary_path)?;
    let policy = load_toml(&policy_path(
        workspace,
        "POLICY_TOML",
        "configs/ci/tools/apptainer_reproducibility_policy.toml",
    ))?;
    let threshold = policy
        .get("confidence_min")
        .and_then(toml::Value::as_float)
        .unwrap_or(1.0);
    let require_all = policy
        .get("require_all_tools_deterministic")
        .and_then(toml::Value::as_bool)
        .unwrap_or(true);
    let mut errors = Vec::new();
    let confidence = summary
        .get("confidence")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(-1.0);
    if confidence < threshold {
        errors.push(format!(
            "confidence below threshold: got {confidence:.4}, need {threshold:.4}"
        ));
    }
    if require_all {
        let bad = summary
            .get("items")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter(|row| {
                !row.get("deterministic")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false)
            })
            .filter_map(|row| {
                row.get("tool")
                    .and_then(serde_json::Value::as_str)
                    .map(ToOwned::to_owned)
            })
            .collect::<Vec<_>>();
        if !bad.is_empty() {
            errors.push(format!("non-deterministic tools: {}", bad.join(", ")));
        }
    }
    if errors.is_empty() {
        return success_line("frontend reproducibility check: OK");
    }
    failure_lines("frontend reproducibility check: FAILED", &errors)
}

pub(crate) fn check_apptainer_frontend_security(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    let usage =
        "Usage: cargo run -p bijux-dna-dev -- containers run check-apptainer-frontend-security -- [<summary-path>]";
    if matches!(args, [single] if single == "--help" || single == "-h") {
        return success_line(usage);
    }
    let summary_path = match args {
        [] => iso_root_path(workspace)
            .join("containers/hpc/frontend-security")
            .join(iso_run_id())
            .join("security_summary.json"),
        [path] => path_from_arg(workspace, path),
        _ => return Err(anyhow!(usage.to_string())),
    };
    if !summary_path.is_file() {
        if env_or_default("CI", "0") == "1" {
            return Ok(ContainerCommandOutcome::failure(format!(
                "frontend security check: missing summary in CI: {}\n",
                summary_path.display()
            )));
        }
        return success_line(format!(
            "frontend security check: SKIP (no summary at {})",
            summary_path.display()
        ));
    }
    let summary = read_json(&summary_path)?;
    let policy = load_toml(&policy_path(
        workspace,
        "POLICY_TOML",
        "configs/ci/tools/apptainer_security_policy.toml",
    ))?;
    let fail_on_critical = policy
        .get("fail_on_unallowlisted_critical")
        .and_then(toml::Value::as_bool)
        .unwrap_or(true);
    let mut errors = Vec::new();
    if summary
        .get("items")
        .and_then(serde_json::Value::as_array)
        .is_none_or(std::vec::Vec::is_empty)
    {
        errors.push("no SBOM/SIF items recorded".to_string());
    }
    if summary
        .get("license_mismatches")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|items| !items.is_empty())
    {
        errors.push("license mismatches present".to_string());
    }
    if fail_on_critical
        && summary
            .get("critical_unallowlisted")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|items| !items.is_empty())
    {
        errors.push("unallowlisted critical CVEs present".to_string());
    }
    if !summary
        .get("ok")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
    {
        errors.push("summary status is fail".to_string());
    }
    if errors.is_empty() {
        return success_line("frontend security check: OK");
    }
    failure_lines("frontend security check: FAILED", &errors)
}

pub(crate) fn check_apptainer_frontend_smoke_proof(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    let usage =
        "Usage: cargo run -p bijux-dna-dev -- containers run check-apptainer-frontend-smoke-proof -- [<proof-root>]";
    if matches!(args, [single] if single == "--help" || single == "-h") {
        return success_line(usage);
    }
    let proof_root = match args {
        [] => workspace.path("artifacts/containers/hpc/frontend-smoke"),
        [path] => path_from_arg(workspace, path),
        _ => return Err(anyhow!(usage.to_string())),
    };
    let summary_path = proof_root.join("summary.json");
    if !summary_path.exists() {
        if env_or_empty("CI").is_empty() {
            return success_line("frontend smoke proof: SKIP (no summary)");
        }
        return Ok(ContainerCommandOutcome::failure(format!(
            "frontend smoke proof: missing {}\n",
            summary_path.display()
        )));
    }
    let summary = read_json(&summary_path)?;
    let versions = tool_versions(workspace)?;
    let apptainer_tools = apptainer_def_paths(workspace)
        .into_iter()
        .filter_map(|path| {
            path.file_stem()
                .and_then(|name| name.to_str())
                .map(ToOwned::to_owned)
        })
        .collect::<BTreeSet<_>>();
    let items = summary
        .get("items")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|row| {
            let tool = row
                .get("tool")
                .and_then(serde_json::Value::as_str)
                .map(|tool| tool.trim().to_string())?;
            Some((tool, row))
        })
        .collect::<BTreeMap<_, _>>();
    let mut errors = Vec::new();
    for tool in apptainer_tools {
        let Some(row) = items.get(&tool) else {
            errors.push(format!("{tool}: missing smoke proof row"));
            continue;
        };
        if row.get("status").and_then(serde_json::Value::as_str) != Some("ok") {
            errors.push(format!("{tool}: smoke status not ok"));
            continue;
        }
        let output = row
            .get("normalized_version_output")
            .and_then(serde_json::Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .or_else(|| {
                row.get("version_output")
                    .and_then(serde_json::Value::as_str)
            })
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase();
        let expected = versions
            .get(&tool)
            .map(|row| table_string(row, "version").to_ascii_lowercase())
            .unwrap_or_default();
        if !expected.is_empty() && !output.contains(&expected) {
            errors.push(format!(
                "{tool}: version output does not include expected version {expected}"
            ));
        }
        for key in [
            "help_actual_exit_code",
            "minimal_actual_exit_code",
            "negative_actual_exit_code",
        ] {
            if row.get(key).is_none() {
                errors.push(format!("{tool}: missing {key}"));
            }
        }
        if row
            .get("network_runtime_detected")
            .and_then(serde_json::Value::as_bool)
            == Some(true)
        {
            errors.push(format!("{tool}: runtime network access detected"));
        }
        if row
            .get("home_write_detected")
            .and_then(serde_json::Value::as_bool)
            == Some(true)
        {
            errors.push(format!("{tool}: write to HOME detected"));
        }
        for key in ["home_policy_ok", "filesystem_policy_ok", "write_policy_ok"] {
            if row.get(key).and_then(serde_json::Value::as_bool) != Some(true) {
                errors.push(format!("{tool}: {key} is false"));
            }
        }
        let log_dir = row
            .get("smoke_log_dir")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_string();
        if log_dir.is_empty() {
            errors.push(format!("{tool}: missing smoke_log_dir"));
        } else if !PathBuf::from(&log_dir)
            .display()
            .to_string()
            .replace('\\', "/")
            .contains(&format!("/smoke/{tool}/"))
        {
            errors.push(format!("{tool}: smoke_log_dir path layout mismatch"));
        }
    }
    if errors.is_empty() {
        return success_line(format!("frontend smoke proof: OK ({})", items.len()));
    }
    failure_lines("frontend smoke proof: failed", &errors)
}

pub(crate) fn check_apptainer_frontend_version_output_lock(
    workspace: &Workspace,
) -> Result<ContainerCommandOutcome> {
    let summary_path = workspace.path("artifacts/containers/hpc/frontend-smoke/summary.json");
    let lock_path = lock_json_path(workspace);
    if !lock_path.exists() {
        return Ok(ContainerCommandOutcome::failure(
            "frontend version-output lock check: missing lock.json\n",
        ));
    }
    if !summary_path.exists() {
        if !env_or_empty("CI").is_empty() {
            return Ok(ContainerCommandOutcome::failure(
                "frontend version-output lock check: missing frontend smoke summary in CI\n",
            ));
        }
        return success_line(
            "frontend version-output lock check: SKIP (no frontend smoke summary)",
        );
    }
    let summary = read_json(&summary_path)?;
    let lock_rows = lock_items_by_tool(workspace)?;
    let mut errors = Vec::new();
    for row in summary
        .get("items")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default()
    {
        let tool = row
            .get("tool")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_string();
        if tool.is_empty() {
            continue;
        }
        if row.get("status").and_then(serde_json::Value::as_str) != Some("ok") {
            errors.push(format!("{tool}: smoke status is not ok"));
            continue;
        }
        let output = row
            .get("normalized_version_output")
            .and_then(serde_json::Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .or_else(|| {
                row.get("version_output")
                    .and_then(serde_json::Value::as_str)
            })
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase();
        if output.is_empty() {
            errors.push(format!(
                "{tool}: empty version output in frontend smoke summary"
            ));
            continue;
        }
        let current = sha256_hex(output.as_bytes());
        let locked = lock_rows
            .get(&tool)
            .and_then(|row| row.get("frontend_smoke_version_output_sha256"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_string();
        if locked.is_empty() {
            errors.push(format!(
                "{tool}: missing frontend_smoke_version_output_sha256 in lock"
            ));
        } else if current != locked {
            errors.push(format!(
                "{tool}: frontend version output drift detected; regenerate lock"
            ));
        }
    }
    if errors.is_empty() {
        return success_line("frontend version-output lock check: OK");
    }
    failure_lines("frontend version-output lock check: failed", &errors)
}

pub(crate) fn compare_frontend_local_sif_hash(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    let usage = "Usage: cargo run -p bijux-dna-dev -- containers run compare-frontend-local-sif-hash -- [<frontend-json>] [<local-json>] [<output-path>]";
    if matches!(args, [single] if single == "--help" || single == "-h") {
        return success_line(usage);
    }
    let (frontend_json, local_json, out_md) = match args {
        [] => (
            workspace.path("artifacts/containers/hpc/frontend-sif-digests.json"),
            workspace.path("artifacts/containers/hpc/local-sif-digests.json"),
            workspace.path("artifacts/containers/hpc/frontend-local-diff.md"),
        ),
        [frontend, local, out] => (
            path_from_arg(workspace, frontend),
            path_from_arg(workspace, local),
            path_from_arg(workspace, out),
        ),
        _ => return Err(anyhow!(usage.to_string())),
    };
    let frontend = if frontend_json.exists() {
        read_json(&frontend_json)?
    } else {
        serde_json::json!({ "items": [] })
    };
    let local = if local_json.exists() {
        read_json(&local_json)?
    } else {
        serde_json::json!({ "items": [] })
    };
    let frontend_map = frontend
        .get("items")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|row| {
            Some((
                row.get("tool")?.as_str()?.trim().to_string(),
                row.get("sha256")?.as_str()?.trim().to_string(),
            ))
        })
        .collect::<BTreeMap<_, _>>();
    let local_map = local
        .get("items")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|row| {
            Some((
                row.get("tool")?.as_str()?.trim().to_string(),
                row.get("sha256")?.as_str()?.trim().to_string(),
            ))
        })
        .collect::<BTreeMap<_, _>>();
    let shared = frontend_map
        .keys()
        .filter(|tool| local_map.contains_key(*tool))
        .cloned()
        .collect::<Vec<_>>();
    let mut lines = vec![
        "# Frontend vs Local SIF Hash Diff".to_string(),
        String::new(),
        "| tool | frontend_sha256 | local_sha256 | match |".to_string(),
        "|---|---|---|---|".to_string(),
    ];
    for tool in &shared {
        let frontend = frontend_map.get(tool).cloned().unwrap_or_default();
        let local = local_map.get(tool).cloned().unwrap_or_default();
        lines.push(format!(
            "| `{tool}` | `{frontend}` | `{local}` | `{}` |",
            if frontend == local { "yes" } else { "no" }
        ));
    }
    let missing_frontend = local_map
        .keys()
        .filter(|tool| !frontend_map.contains_key(*tool))
        .cloned()
        .collect::<Vec<_>>();
    let missing_local = frontend_map
        .keys()
        .filter(|tool| !local_map.contains_key(*tool))
        .cloned()
        .collect::<Vec<_>>();
    if !missing_frontend.is_empty() {
        lines.extend([
            String::new(),
            "## Missing On Frontend".to_string(),
            String::new(),
        ]);
        lines.extend(missing_frontend.iter().map(|tool| format!("- `{tool}`")));
    }
    if !missing_local.is_empty() {
        lines.extend([
            String::new(),
            "## Missing Locally".to_string(),
            String::new(),
        ]);
        lines.extend(missing_local.iter().map(|tool| format!("- `{tool}`")));
    }
    let mismatch = shared
        .iter()
        .filter(|tool| frontend_map.get(*tool) != local_map.get(*tool))
        .cloned()
        .collect::<Vec<_>>();
    if !mismatch.is_empty() {
        lines.extend([
            String::new(),
            "## Deterministic Causes To Document".to_string(),
            String::new(),
            "- base image digest drift".to_string(),
            "- build timestamp embedded in image".to_string(),
            "- tool download source changed".to_string(),
            "- Apptainer/host version differences".to_string(),
        ]);
    }
    write_utf8(&out_md, &format!("{}\n", lines.join("\n")))?;
    if mismatch.is_empty() {
        return success_line(out_md.display().to_string());
    }
    Ok(ContainerCommandOutcome {
        exit_code: 1,
        stdout: format!("{}\n", out_md.display()),
        stderr: String::new(),
    })
}

pub(crate) fn write_frontend_repro_summary(
    workspace: &Workspace,
    policy: &toml::Value,
    seed: &str,
    items: &[serde_json::Value],
    summary_path: &Path,
    doc_path: &Path,
) -> Result<()> {
    let threshold = policy
        .get("confidence_min")
        .and_then(toml::Value::as_float)
        .unwrap_or(1.0);
    let require_all = policy
        .get("require_all_tools_deterministic")
        .and_then(toml::Value::as_bool)
        .unwrap_or(true);
    let total_checks = items.len() * 3;
    let passed_checks = items
        .iter()
        .map(|row| {
            row.get("checks")
                .and_then(serde_json::Value::as_object)
                .map_or(0, |checks| {
                    ["same_cache_twice", "clean_cache_match", "purge_cache_match"]
                        .into_iter()
                        .filter(|key| {
                            checks
                                .get(*key)
                                .and_then(serde_json::Value::as_bool)
                                .unwrap_or(false)
                        })
                        .count()
                })
        })
        .sum::<usize>();
    let confidence = if total_checks == 0 {
        1.0
    } else {
        passed_checks as f64 / total_checks as f64
    };
    let all_ok = items.iter().all(|row| {
        row.get("deterministic")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false)
    });
    let ok = confidence >= threshold && (!require_all || all_ok);
    write_utf8(
        summary_path,
        &format!(
            "{}\n",
            serde_json::to_string_pretty(&serde_json::json!({
                "schema_version": "bijux.apptainer.frontend_reproducibility.v2",
                "host": validation::current_host_name(workspace),
                "seed": seed,
                "confidence_min": threshold,
                "require_all_tools_deterministic": require_all,
                "items": items,
                "confidence": confidence,
                "confidence_total_checks": total_checks,
                "confidence_passed_checks": passed_checks,
                "ok": ok,
            }))?
        ),
    )?;
    let mut lines = vec![
        "<!-- Generated by cargo run -p bijux-dna-dev -- containers run run-apptainer-frontend-reproducibility -->".to_string(),
        String::new(),
        "# Apptainer Frontend Reproducibility Report".to_string(),
        String::new(),
        format!("- host: `{}`", validation::current_host_name(workspace)),
        format!("- seed: `{seed}`"),
        format!("- sampled_tools: `{}`", items.len()),
        format!("- confidence: `{confidence:.4}` (threshold `{threshold:.4}`)"),
        format!("- all_tools_deterministic_required: `{}`", if require_all { "true" } else { "false" }),
        format!("- gate_status: `{}`", if ok { "PASS" } else { "FAIL" }),
        String::new(),
        "| tool | same_cache_twice | clean_cache_match | purge_cache_match | deterministic | cause_if_mismatch |".to_string(),
        "|---|---:|---:|---:|---:|---|".to_string(),
    ];
    for row in items {
        let tool = row
            .get("tool")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        let checks = row.get("checks").and_then(serde_json::Value::as_object);
        let same = checks
            .and_then(|value| value.get("same_cache_twice"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        let clean = checks
            .and_then(|value| value.get("clean_cache_match"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        let purge = checks
            .and_then(|value| value.get("purge_cache_match"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        let deterministic = row
            .get("deterministic")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        let cause = row
            .get("nondeterministic_cause")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        lines.push(format!(
            "| `{tool}` | `{same}` | `{clean}` | `{purge}` | `{deterministic}` | `{cause}` |"
        ));
    }
    write_utf8(doc_path, &format!("{}\n", lines.join("\n")))
}

pub(crate) fn write_frontend_security_summary(
    workspace: &Workspace,
    out_dir: &Path,
    summary_path: &Path,
    doc_path: &Path,
) -> Result<()> {
    let policy = load_toml(&workspace.path("configs/ci/tools/apptainer_security_policy.toml"))?;
    let allowlist_path = policy
        .get("vuln_allowlist_path")
        .and_then(toml::Value::as_str)
        .map(|rel| workspace.path(rel))
        .filter(|path| path.is_file());
    let allowlisted = if let Some(path) = allowlist_path {
        load_toml(&path)?
            .get("allowlist")
            .and_then(toml::Value::as_array)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|row| {
                row.get("cve")
                    .and_then(toml::Value::as_str)
                    .map(str::to_ascii_uppercase)
            })
            .collect::<BTreeSet<_>>()
    } else {
        BTreeSet::new()
    };
    let fail_on_critical = policy
        .get("fail_on_unallowlisted_critical")
        .and_then(toml::Value::as_bool)
        .unwrap_or(true);
    let require_scanner_ci = policy
        .get("require_scanner_in_ci")
        .and_then(toml::Value::as_bool)
        .unwrap_or(true);
    let require_scanner_local = policy
        .get("require_scanner_local")
        .and_then(toml::Value::as_bool)
        .unwrap_or(false);
    let is_ci = !env_or_empty("CI").is_empty();
    let scanner = if command_exists("grype") {
        Some("grype")
    } else if command_exists("trivy") {
        Some("trivy")
    } else {
        None
    };
    if scanner.is_none() && ((is_ci && require_scanner_ci) || (!is_ci && require_scanner_local)) {
        return Err(anyhow!(
            "frontend security summary requires grype or trivy per policy"
        ));
    }
    let manifests = WalkDir::new(out_dir)
        .max_depth(1)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| entry.path().extension().and_then(|ext| ext.to_str()) == Some("json"))
        .filter(|entry| {
            !matches!(
                entry
                    .path()
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or_default(),
                "summary.json"
                    | "security_summary.json"
                    | "vuln_scan_report.json"
                    | "sbom_index.json"
            )
        })
        .collect::<Vec<_>>();
    let mut sbom_rows = Vec::new();
    let mut vuln_items = Vec::new();
    let mut critical_total = 0usize;
    let mut critical_unallowlisted = Vec::new();
    let mut license_mismatches = Vec::new();
    let vuln_dir = out_dir.join("vuln");
    bijux_dna_infra::ensure_dir(&vuln_dir)
        .with_context(|| format!("create {}", vuln_dir.display()))?;

    for entry in manifests {
        let row = read_json(entry.path())?;
        let tool = row
            .get("tool")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string();
        if tool.is_empty() {
            continue;
        }
        let sbom_path = PathBuf::from(
            row.get("sbom_path")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default(),
        );
        let sif_path = PathBuf::from(
            row.get("image")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default(),
        );
        if !sbom_path.is_file() {
            continue;
        }
        let sbom_sha256 = sha256_hex(
            &fs::read(&sbom_path).with_context(|| format!("read {}", sbom_path.display()))?,
        );
        let sif_sha256 = if sif_path.is_file() {
            sha256_hex(
                &fs::read(&sif_path).with_context(|| format!("read {}", sif_path.display()))?,
            )
        } else {
            String::new()
        };
        sbom_rows.push(serde_json::json!({
            "tool": tool,
            "sbom_path": sbom_path.display().to_string(),
            "sbom_sha256": sbom_sha256,
            "sif_path": sif_path.display().to_string(),
            "sif_sha256": sif_sha256,
        }));
    }

    for row in &sbom_rows {
        let tool = row
            .get("tool")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        let sbom = row
            .get("sbom_path")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        let mut counts = BTreeMap::from([
            ("critical".to_string(), 0usize),
            ("high".to_string(), 0usize),
            ("medium".to_string(), 0usize),
            ("low".to_string(), 0usize),
            ("unknown".to_string(), 0usize),
        ]);
        if let Some(scanner_name) = scanner {
            let output = if scanner_name == "grype" {
                run_program_with_env(
                    workspace,
                    "grype",
                    &[format!("sbom:{sbom}"), "-o".to_string(), "json".to_string()],
                    &[],
                )?
            } else {
                run_program_with_env(
                    workspace,
                    "trivy",
                    &[
                        "sbom".to_string(),
                        "--format".to_string(),
                        "json".to_string(),
                        sbom.to_string(),
                    ],
                    &[],
                )?
            };
            let raw = if output.stdout.trim().is_empty() {
                "{}".to_string()
            } else {
                output.stdout.clone()
            };
            let suffix = if scanner_name == "grype" {
                "grype"
            } else {
                "trivy"
            };
            write_utf8(&vuln_dir.join(format!("{tool}.{suffix}.json")), &raw)?;
            let payload = serde_json::from_str::<serde_json::Value>(&raw)
                .unwrap_or_else(|_| serde_json::json!({}));
            let mut parsed = Vec::new();
            if scanner_name == "grype" {
                if let Some(matches) = payload.get("matches").and_then(serde_json::Value::as_array)
                {
                    for item in matches {
                        let vuln = item.get("vulnerability").unwrap_or(item);
                        let cve = vuln
                            .get("id")
                            .and_then(serde_json::Value::as_str)
                            .unwrap_or_default()
                            .to_ascii_uppercase();
                        let sev = vuln
                            .get("severity")
                            .and_then(serde_json::Value::as_str)
                            .unwrap_or("UNKNOWN")
                            .to_ascii_uppercase();
                        if !cve.is_empty() {
                            parsed.push((cve, sev));
                        }
                    }
                }
            } else if let Some(results) =
                payload.get("Results").and_then(serde_json::Value::as_array)
            {
                for result in results {
                    if let Some(vulns) = result
                        .get("Vulnerabilities")
                        .and_then(serde_json::Value::as_array)
                    {
                        for vuln in vulns {
                            let cve = vuln
                                .get("VulnerabilityID")
                                .and_then(serde_json::Value::as_str)
                                .unwrap_or_default()
                                .to_ascii_uppercase();
                            let sev = vuln
                                .get("Severity")
                                .and_then(serde_json::Value::as_str)
                                .unwrap_or("UNKNOWN")
                                .to_ascii_uppercase();
                            if !cve.is_empty() {
                                parsed.push((cve, sev));
                            }
                        }
                    }
                }
            }
            for (cve, severity) in parsed {
                let key = severity.to_ascii_lowercase();
                *counts.entry(key.clone()).or_insert(0) += 1;
                if severity == "CRITICAL" {
                    critical_total += 1;
                    if !allowlisted.contains(&cve) {
                        critical_unallowlisted.push(serde_json::json!({
                            "tool": tool,
                            "cve": cve,
                        }));
                    }
                }
            }
        }
        vuln_items.push(serde_json::json!({
            "tool": tool,
            "scanner": scanner.unwrap_or("none"),
            "critical": counts.get("critical").copied().unwrap_or(0),
            "high": counts.get("high").copied().unwrap_or(0),
            "medium": counts.get("medium").copied().unwrap_or(0),
            "low": counts.get("low").copied().unwrap_or(0),
            "unknown": counts.get("unknown").copied().unwrap_or(0),
        }));
        let license_file = workspace.path(&format!("containers/licenses/{tool}.license.toml"));
        if license_file.is_file() {
            let license = load_toml(&license_file)?;
            if license
                .get("spdx")
                .and_then(toml::Value::as_str)
                .unwrap_or_default()
                .trim()
                .is_empty()
            {
                license_mismatches.push(format!(
                    "{tool}: empty spdx in {}",
                    workspace.rel(&license_file).display()
                ));
            }
        } else {
            license_mismatches.push(format!(
                "{tool}: missing {}",
                workspace.rel(&license_file).display()
            ));
        }
    }

    let ok = if fail_on_critical {
        critical_unallowlisted.is_empty() && license_mismatches.is_empty()
    } else {
        license_mismatches.is_empty()
    };
    write_utf8(
        summary_path,
        &format!(
            "{}\n",
            serde_json::to_string_pretty(&serde_json::json!({
                "schema_version": "bijux.apptainer.frontend.security.v2",
                "host": validation::current_host_name(workspace),
                "scanner": scanner.unwrap_or("none"),
                "items": sbom_rows,
                "vulnerabilities": vuln_items,
                "critical_total": critical_total,
                "critical_unallowlisted": critical_unallowlisted,
                "license_mismatches": license_mismatches,
                "ok": ok,
            }))?
        ),
    )?;
    let summary_json = read_json(summary_path)?;
    let mut lines = vec![
        "<!-- Generated by cargo run -p bijux-dna-dev -- containers run run-apptainer-frontend-security -->".to_string(),
        String::new(),
        "# Apptainer Frontend Security Summary".to_string(),
        String::new(),
        format!("- host: `{}`", validation::current_host_name(workspace)),
        format!("- scanner: `{}`", scanner.unwrap_or("none")),
        format!("- sif_count: `{}`", summary_json.get("items").and_then(serde_json::Value::as_array).map_or(0, Vec::len)),
        format!("- critical_total: `{}`", critical_total),
        format!("- critical_unallowlisted: `{}`", summary_json.get("critical_unallowlisted").and_then(serde_json::Value::as_array).map_or(0, Vec::len)),
        format!("- license_mismatches: `{}`", summary_json.get("license_mismatches").and_then(serde_json::Value::as_array).map_or(0, Vec::len)),
        format!("- gate_status: `{}`", if ok { "PASS" } else { "FAIL" }),
        String::new(),
        "## SBOM Index".to_string(),
        String::new(),
        "| tool | sif_sha256 | sbom_sha256 | sbom_path |".to_string(),
        "|---|---|---|---|".to_string(),
    ];
    if let Some(items) = summary_json
        .get("items")
        .and_then(serde_json::Value::as_array)
    {
        for row in items {
            lines.push(format!(
                "| `{}` | `{}` | `{}` | `{}` |",
                row.get("tool")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default(),
                row.get("sif_sha256")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default(),
                row.get("sbom_sha256")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default(),
                row.get("sbom_path")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default(),
            ));
        }
    }
    lines.extend([
        String::new(),
        "## Vulnerability Summary".to_string(),
        String::new(),
        "| tool | critical | high | medium | low | unknown |".to_string(),
        "|---|---:|---:|---:|---:|---:|".to_string(),
    ]);
    if let Some(items) = summary_json
        .get("vulnerabilities")
        .and_then(serde_json::Value::as_array)
    {
        for row in items {
            lines.push(format!(
                "| `{}` | `{}` | `{}` | `{}` | `{}` | `{}` |",
                row.get("tool")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default(),
                row.get("critical")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or(0),
                row.get("high")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or(0),
                row.get("medium")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or(0),
                row.get("low")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or(0),
                row.get("unknown")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or(0),
            ));
        }
    }
    write_utf8(doc_path, &format!("{}\n", lines.join("\n")))
}
