use super::{
    env_or_empty, failure_lines, fs, load_toml, read_json, registry_tool_id, success_line,
    table_bool, table_string, BTreeMap, BTreeSet, ContainerCommandOutcome, Context, PathBuf,
    Result, Workspace,
};

pub(in super::super::super) fn check_smoke_failure_classification(
    workspace: &Workspace,
) -> Result<ContainerCommandOutcome> {
    let manifests = workspace.path("artifacts/containers/manifests");
    if !manifests.exists() {
        return success_line("smoke failure classification: SKIP (no manifests)");
    }
    let allowed =
        BTreeSet::from(["build".to_string(), "runtime".to_string(), "smoke_mismatch".to_string()]);
    let mut errors = Vec::new();
    for entry in fs::read_dir(&manifests)
        .with_context(|| format!("read {}", manifests.display()))?
        .filter_map(std::result::Result::ok)
    {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        match read_json(&path) {
            Ok(data) => {
                if data.get("status").and_then(serde_json::Value::as_str) == Some("fail") {
                    let fail_class = data
                        .get("fail_class")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or_default()
                        .trim()
                        .to_string();
                    if !allowed.contains(&fail_class) {
                        errors.push(format!(
                            "{}: missing/invalid fail_class '{}'",
                            workspace.rel(&path).display(),
                            fail_class
                        ));
                    }
                }
            }
            Err(_) => errors.push(format!("{}: invalid JSON", workspace.rel(&path).display())),
        }
    }
    if errors.is_empty() {
        return success_line("smoke failure classification: OK");
    }
    failure_lines("smoke failure classification: failed", &errors)
}

pub(in super::super::super) fn check_smoke_contract(
    workspace: &Workspace,
) -> Result<ContainerCommandOutcome> {
    let contract_doc = workspace.path("containers/docs/SMOKE_CONTRACT.md");
    if !contract_doc.exists() {
        return Ok(ContainerCommandOutcome::failure(format!(
            "smoke contract check failed: missing {}\n",
            contract_doc.display()
        )));
    }
    let images_path = workspace.path("configs/ci/tools/images.toml");
    let mut exempt = BTreeSet::new();
    if images_path.exists() {
        let images = load_toml(&images_path)?;
        if let Some(table) = images.get("smoke_exemptions").and_then(toml::Value::as_table) {
            for (tool, value) in table {
                if value.as_bool() == Some(true) {
                    exempt.insert(tool.clone());
                }
            }
        }
    }

    let allowed_statuses = BTreeSet::from(["production".to_string(), "supported".to_string()]);
    let mut errors = Vec::new();
    for rel in [
        "configs/ci/registry/tool_registry.toml",
        "configs/ci/registry/tool_registry_vcf.toml",
        "configs/ci/registry/tool_registry_vcf_downstream.toml",
    ] {
        let value = load_toml(&workspace.path(rel))?;
        for row in value.get("tools").and_then(toml::Value::as_array).cloned().unwrap_or_default() {
            let Some(row) = row.as_table() else {
                continue;
            };
            let status = table_string(row, "status");
            let status_allowed = allowed_statuses.contains(&status)
                || (rel.ends_with("tool_registry_vcf_downstream.toml") && status == "planned");
            if !status_allowed || !table_bool(row, "container") {
                continue;
            }
            let tool_id = registry_tool_id(row);
            if tool_id.is_empty() || exempt.contains(&tool_id) {
                continue;
            }
            let version_cmd = table_string(row, "smoke_version_cmd");
            let help_cmd = table_string(row, "smoke_help_cmd");
            let minimal_cmd = {
                let value = table_string(row, "smoke_minimal_cmd");
                if value.is_empty() {
                    format!("{tool_id} --help")
                } else {
                    value
                }
            };
            let negative_cmd = {
                let value = table_string(row, "smoke_negative_cmd");
                if value.is_empty() {
                    format!("{tool_id} --__bijux_invalid_flag__")
                } else {
                    value
                }
            };
            let negative_pattern = {
                let value = table_string(row, "smoke_negative_expected_pattern");
                if value.is_empty() {
                    "invalid|unknown|error|usage".to_string()
                } else {
                    value
                }
            };
            let expected_bin = table_string(row, "expected_bin");
            let help_exit =
                row.get("smoke_help_exit_code").map_or(Some(0), toml::Value::as_integer);
            let minimal_exit =
                row.get("smoke_minimal_exit_code").map_or(Some(0), toml::Value::as_integer);
            let negative_exit =
                row.get("smoke_negative_exit_code").map_or(Some(2), toml::Value::as_integer);

            if version_cmd.is_empty() {
                errors.push(format!("{rel}: {tool_id} missing smoke_version_cmd"));
            }
            if help_cmd.is_empty() {
                errors.push(format!("{rel}: {tool_id} missing smoke_help_cmd"));
            }
            if help_exit != Some(0) {
                errors.push(format!("{rel}: {tool_id} smoke_help_exit_code must be 0"));
            }
            if expected_bin.is_empty() {
                errors.push(format!("{rel}: {tool_id} missing expected_bin tool binary contract"));
            }
            if minimal_cmd.is_empty() {
                errors.push(format!("{rel}: {tool_id} resolved smoke_minimal_cmd is empty"));
            }
            if minimal_exit.is_none() {
                errors.push(format!("{rel}: {tool_id} smoke_minimal_exit_code must be integer"));
            }
            if negative_cmd.is_empty() {
                errors.push(format!("{rel}: {tool_id} resolved smoke_negative_cmd is empty"));
            }
            if negative_exit.is_none() {
                errors.push(format!("{rel}: {tool_id} smoke_negative_exit_code must be integer"));
            }
            if negative_pattern.is_empty() {
                errors.push(format!(
                    "{rel}: {tool_id} resolved smoke_negative_expected_pattern is empty"
                ));
            }
        }
    }

    if errors.is_empty() {
        return success_line("smoke contract: OK");
    }
    failure_lines("smoke contract check failed:", &errors)
}

pub(in super::super::super) fn check_smoke_contract_lock(
    workspace: &Workspace,
) -> Result<ContainerCommandOutcome> {
    let lock_path = std::env::var("LOCK_PATH")
        .map_or_else(|_| workspace.path("containers/versions/lock.json"), PathBuf::from);
    let summary_path = std::env::var("SUMMARY_PATH").map_or_else(
        |_| workspace.path("artifacts/containers/hpc/frontend-smoke/summary.json"),
        PathBuf::from,
    );

    if !lock_path.exists() {
        return Ok(ContainerCommandOutcome::failure(format!(
            "smoke lock gate: missing lock file {}\n",
            lock_path.display()
        )));
    }
    if !summary_path.exists() {
        if !env_or_empty("CI").is_empty() {
            return Ok(ContainerCommandOutcome::failure(format!(
                "smoke lock gate: missing smoke summary {}\n",
                summary_path.display()
            )));
        }
        return success_line(format!(
            "smoke lock gate: SKIP (missing smoke summary {})",
            summary_path.display()
        ));
    }

    let lock = read_json(&lock_path)?;
    let summary = read_json(&summary_path)?;
    let rows = summary
        .get("items")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|row| {
            let tool = row
                .get("tool")
                .and_then(serde_json::Value::as_str)
                .map(|value| value.trim().to_string())?;
            Some((tool, row))
        })
        .collect::<BTreeMap<_, _>>();
    let mut errors = Vec::new();
    let mut total = 0usize;
    for item in lock.get("items").and_then(serde_json::Value::as_array).cloned().unwrap_or_default()
    {
        let tool = item
            .get("tool")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_string();
        if tool.is_empty() {
            continue;
        }
        total += 1;
        let Some(row) = rows.get(&tool) else {
            errors.push(format!("{tool}: missing smoke summary row"));
            continue;
        };
        if row.get("status").and_then(serde_json::Value::as_str) != Some("ok") {
            errors.push(format!("{tool}: smoke status is not ok"));
        }
        let log_dir = row
            .get("smoke_log_dir")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_string();
        if log_dir.is_empty() {
            errors.push(format!("{tool}: missing smoke_log_dir"));
            continue;
        }
        let log_dir_path = PathBuf::from(&log_dir);
        if !log_dir_path.exists() {
            errors.push(format!("{tool}: smoke_log_dir does not exist: {log_dir}"));
        }
        if !log_dir_path
            .display()
            .to_string()
            .replace('\\', "/")
            .contains(&format!("/smoke/{tool}/"))
        {
            errors.push(format!("{tool}: smoke_log_dir not in required layout: {log_dir}"));
        }
    }

    if errors.is_empty() {
        return success_line(format!("smoke lock gate: OK ({total} tools)"));
    }
    failure_lines("smoke lock gate: FAILED", &errors)
}
