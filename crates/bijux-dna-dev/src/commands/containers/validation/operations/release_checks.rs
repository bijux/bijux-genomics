use super::{
    apptainer_def_paths, dockerfile_paths, env_or_empty, failure_lines, git_show_file, load_toml,
    read_json, read_utf8, registry_tool_id, success_line, table_bool, table_string, walk_paths,
    BTreeMap, BTreeSet, ContainerCommandOutcome, PathBuf, Regex, Result, Workspace,
};

pub(in super::super::super) fn check_image_size_regression(
    workspace: &Workspace,
) -> Result<ContainerCommandOutcome> {
    let policy_path = workspace.path("configs/ci/tools/image_size_policy.toml");
    let lock_path = workspace.path("containers/versions/lock.json");
    if !policy_path.exists() || !lock_path.exists() {
        return success_line("image size regression: SKIP (missing policy/lock)");
    }
    let policy = load_toml(&policy_path)?;
    let default_limit = policy
        .get("max_growth_percent_for_promoted")
        .and_then(toml::Value::as_float)
        .unwrap_or(20.0);
    let mut acknowledgements = BTreeMap::new();
    for row in
        policy.get("acknowledgement").and_then(toml::Value::as_array).cloned().unwrap_or_default()
    {
        let Some(row) = row.as_table() else {
            continue;
        };
        let tool = table_string(row, "tool_id");
        let from_version = table_string(row, "from_version");
        let to_version = table_string(row, "to_version");
        let limit =
            row.get("max_growth_percent").and_then(toml::Value::as_float).unwrap_or(default_limit);
        if !tool.is_empty() && !from_version.is_empty() && !to_version.is_empty() {
            acknowledgements.insert((tool, from_version, to_version), limit);
        }
    }
    let current = read_json(&lock_path)?;
    let current_items = current
        .get("items")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|row| {
            let tool = row.get("tool").and_then(serde_json::Value::as_str)?.to_string();
            Some((tool, row))
        })
        .collect::<BTreeMap<_, _>>();
    let previous_lock_text = git_show_file(workspace, "HEAD~1", "containers/versions/lock.json")?;
    if previous_lock_text.trim().is_empty() {
        return success_line("image size regression: SKIP (no previous lock available)");
    }
    let previous = serde_json::from_str::<serde_json::Value>(&previous_lock_text)?;
    let previous_items = previous
        .get("items")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|row| {
            let tool = row.get("tool").and_then(serde_json::Value::as_str)?.to_string();
            Some((tool, row))
        })
        .collect::<BTreeMap<_, _>>();
    let mut checked = 0usize;
    let mut errors = Vec::new();
    for (tool, current_row) in current_items {
        if current_row.get("status").and_then(serde_json::Value::as_str) != Some("production") {
            continue;
        }
        let Some(previous_row) = previous_items.get(&tool) else {
            continue;
        };
        let old_size =
            previous_row.get("image_size_bytes").and_then(serde_json::Value::as_i64).unwrap_or(0);
        let new_size =
            current_row.get("image_size_bytes").and_then(serde_json::Value::as_i64).unwrap_or(0);
        if old_size <= 0 || new_size <= 0 {
            continue;
        }
        checked += 1;
        let growth = ((new_size - old_size) as f64 / old_size as f64) * 100.0;
        let from_version = previous_row
            .get("version")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_string();
        let to_version = current_row
            .get("version")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_string();
        let limit = acknowledgements
            .get(&(tool.clone(), from_version.clone(), to_version.clone()))
            .copied()
            .unwrap_or(default_limit);
        if growth > limit {
            errors.push(format!(
                "{tool}: image grew {growth:.2}% ({old_size} -> {new_size}) over allowed {limit:.2}% (version {from_version} -> {to_version}); add acknowledgement if intentional"
            ));
        }
    }
    if errors.is_empty() {
        return success_line(format!(
            "image size regression: OK ({checked} promoted tools compared)"
        ));
    }
    failure_lines("image size regression: FAILED", &errors)
}

pub(in super::super::super) fn check_lock_matches_built_output(
    workspace: &Workspace,
) -> Result<ContainerCommandOutcome> {
    let lock_path = workspace.path("containers/versions/lock.json");
    let summary_path = workspace.path("artifacts/containers/summary.json");
    if !lock_path.exists() {
        return Ok(ContainerCommandOutcome::failure(
            "lock-vs-built: missing containers/versions/lock.json\n",
        ));
    }
    if !summary_path.exists() {
        if !env_or_empty("CI").is_empty() {
            return Ok(ContainerCommandOutcome::failure(
                "lock-vs-built: missing artifacts/containers/summary.json\n",
            ));
        }
        return success_line("lock-vs-built: SKIP (no artifacts/containers/summary.json)");
    }

    let lock_data = read_json(&lock_path)?;
    let lock_items =
        lock_data.get("items").and_then(serde_json::Value::as_array).cloned().unwrap_or_default();
    let lock_tools = lock_items
        .iter()
        .filter_map(|item| item.get("tool").and_then(serde_json::Value::as_str))
        .map(ToOwned::to_owned)
        .collect::<BTreeSet<_>>();
    let lock_rows = lock_items
        .into_iter()
        .filter_map(|item| {
            let tool = item.get("tool").and_then(serde_json::Value::as_str)?.to_string();
            Some((tool, item))
        })
        .collect::<BTreeMap<_, _>>();

    let mut production = BTreeMap::new();
    for rel in [
        "configs/ci/registry/tool_registry.toml",
        "configs/ci/registry/tool_registry_vcf.toml",
        "configs/ci/registry/tool_registry_experimental.toml",
        "configs/ci/registry/tool_registry_vcf_downstream.toml",
    ] {
        let data = load_toml(&workspace.path(rel))?;
        for row in data.get("tools").and_then(toml::Value::as_array).cloned().unwrap_or_default() {
            let Some(row) = row.as_table() else {
                continue;
            };
            if table_string(row, "status") != "production" || !table_bool(row, "container") {
                continue;
            }
            let tool = registry_tool_id(row);
            if !tool.is_empty() {
                production.insert(tool, table_string(row, "version"));
            }
        }
    }

    let summary = read_json(&summary_path)?;
    let mut docker_manifest_by_tool = BTreeMap::new();
    let mut apptainer_manifest_by_tool = BTreeMap::new();
    for item in
        summary.get("items").and_then(serde_json::Value::as_array).cloned().unwrap_or_default()
    {
        let tool = item
            .get("tool")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_string();
        let runtime = item
            .get("runtime")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_string();
        let manifest = item.get("manifest").and_then(serde_json::Value::as_str).unwrap_or_default();
        if tool.is_empty() || manifest.is_empty() {
            continue;
        }
        let path = PathBuf::from(manifest);
        if !path.exists() {
            continue;
        }
        let Ok(manifest_json) = read_json(&path) else {
            continue;
        };
        match runtime.as_str() {
            "docker-arm64" => {
                docker_manifest_by_tool.insert(tool, manifest_json);
            }
            "apptainer" => {
                apptainer_manifest_by_tool.insert(tool, manifest_json);
            }
            _ => {}
        }
    }

    let strict_missing = !env_or_empty("CI").is_empty();
    let mut errors = Vec::new();
    for (tool, expected_version) in production {
        if !lock_tools.contains(&tool) {
            errors.push(format!("{tool}: missing from containers/versions/lock.json"));
        }
        let Some(docker_manifest) = docker_manifest_by_tool.get(&tool) else {
            if strict_missing {
                errors.push(format!(
                    "{tool}: missing docker-arm64 manifest in artifacts/containers/summary.json"
                ));
            }
            continue;
        };
        if docker_manifest.get("status").and_then(serde_json::Value::as_str) != Some("ok") {
            errors.push(format!("{tool}: docker manifest status is not ok"));
        }
        let declared_version = docker_manifest
            .get("declared_version")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_string();
        if !declared_version.is_empty()
            && !expected_version.is_empty()
            && declared_version != expected_version
        {
            errors.push(format!(
                "{tool}: declared_version '{declared_version}' != registry version '{expected_version}'"
            ));
        }
        let lock_version = lock_rows
            .get(&tool)
            .and_then(|row| row.get("version"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_string();
        if !lock_version.is_empty()
            && !declared_version.is_empty()
            && lock_version != declared_version
        {
            errors.push(format!(
                "{tool}: lock version '{lock_version}' != declared_version '{declared_version}'"
            ));
        }
        let version_output = docker_manifest
            .get("version_output")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_string();
        if !lock_version.is_empty()
            && !matches!(lock_version.as_str(), "0.0.0" | "planned" | "unknown")
        {
            if version_output.is_empty() {
                errors.push(format!("{tool}: missing version_output for lock/version comparison"));
            } else if !version_output
                .to_ascii_lowercase()
                .contains(&lock_version.to_ascii_lowercase())
            {
                errors.push(format!(
                    "{tool}: version_output '{version_output}' does not contain lock version '{lock_version}'"
                ));
            }
        }
        let digest = docker_manifest
            .get("resolved_image_digest")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_string();
        if digest.is_empty() {
            errors.push(format!("{tool}: missing resolved_image_digest in docker manifest"));
        }
        let lock_digest = lock_rows
            .get(&tool)
            .and_then(|row| row.get("resolved_image_digest"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_string();
        if !lock_digest.is_empty() && !digest.is_empty() && lock_digest != digest {
            errors.push(format!(
                "{tool}: built docker digest '{digest}' does not match lock resolved_image_digest '{lock_digest}'"
            ));
        }
        let lock_sif = lock_rows
            .get(&tool)
            .and_then(|row| row.get("sif_digest_sha256"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_string();
        if let Some(apptainer_manifest) = apptainer_manifest_by_tool.get(&tool) {
            let apptainer_digest = apptainer_manifest
                .get("resolved_image_digest")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .trim()
                .to_string();
            if !lock_sif.is_empty() && !apptainer_digest.is_empty() && lock_sif != apptainer_digest
            {
                errors.push(format!(
                    "{tool}: built apptainer sif digest '{apptainer_digest}' does not match lock sif_digest_sha256 '{lock_sif}'"
                ));
            }
        }
    }

    if errors.is_empty() {
        return success_line("lock-vs-built: OK");
    }
    failure_lines("lock-vs-built: failed", &errors)
}

pub(in super::super::super) fn check_release_checklist(
    workspace: &Workspace,
) -> Result<ContainerCommandOutcome> {
    let checklist_path = workspace.path("containers/docs/RELEASE_CHECKLIST.md");
    if !checklist_path.exists() {
        return Ok(ContainerCommandOutcome::failure(
            "release checklist check: missing containers/docs/RELEASE_CHECKLIST.md\n",
        ));
    }
    let checklist = read_utf8(&checklist_path)?;
    let registry = crate::catalog::containers::container_registry(workspace)?;
    let command_regex = Regex::new(r"cargo run -p bijux-dna-dev -- containers run ([a-z0-9-]+)")?;
    let missing = command_regex
        .captures_iter(&checklist)
        .filter_map(|capture| capture.get(1).map(|value| value.as_str().to_string()))
        .filter(|command| !registry.iter().any(|row| row.id == *command))
        .collect::<Vec<_>>();
    if missing.is_empty() {
        return success_line("release checklist mapping: OK");
    }
    failure_lines("release checklist check: missing native checklist commands:", &missing)
}

pub(in super::super::super) fn check_toolkit_bundle_buildable(
    workspace: &Workspace,
) -> Result<ContainerCommandOutcome> {
    let bundles = load_toml(&workspace.path("configs/ci/tools/toolkit_bundles.toml"))?;
    let images = load_toml(&workspace.path("configs/ci/tools/images.toml"))?;
    let bundle_table =
        bundles.get("bundles").and_then(toml::Value::as_table).cloned().unwrap_or_default();
    let image_table = images.as_table().cloned().unwrap_or_default();
    let apptainer = apptainer_def_paths(workspace)
        .into_iter()
        .filter_map(|path| path.file_stem().and_then(|value| value.to_str()).map(ToOwned::to_owned))
        .collect::<BTreeSet<_>>();
    let docker = dockerfile_paths(workspace)?
        .into_iter()
        .filter_map(|path| {
            path.file_name()
                .and_then(|value| value.to_str())
                .and_then(|value| value.split_once("Dockerfile.").map(|(_, tool)| tool.to_string()))
        })
        .collect::<BTreeSet<_>>();
    let mut errors = Vec::new();
    for (bundle_id, spec) in bundle_table {
        let Some(spec) = spec.as_table() else {
            continue;
        };
        let tools = spec
            .get("tools")
            .and_then(toml::Value::as_array)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|value| value.as_str().map(ToOwned::to_owned))
            .collect::<Vec<_>>();
        if tools.is_empty() {
            errors.push(format!("{bundle_id}: empty tools list"));
            continue;
        }
        let mut any_buildable = false;
        for tool in tools {
            let status = image_table
                .get(&tool)
                .and_then(toml::Value::as_table)
                .map(|row| table_string(row, "status"))
                .unwrap_or_default();
            if apptainer.contains(&tool) || docker.contains(&tool) {
                any_buildable = true;
            } else if status != "planned" {
                errors.push(format!(
                    "{bundle_id}: tool '{tool}' is not buildable (no docker/apptainer def)"
                ));
            }
        }
        if !any_buildable {
            errors.push(format!("{bundle_id}: no buildable tools in bundle"));
        }
    }
    if errors.is_empty() {
        return success_line("toolkit bundle buildable: OK");
    }
    failure_lines("toolkit bundle buildable: FAILED", &errors)
}

pub(in super::super::super) fn check_vcf_downstream_bundle_coverage(
    workspace: &Workspace,
) -> Result<ContainerCommandOutcome> {
    let bundles = load_toml(&workspace.path("configs/ci/tools/toolkit_bundles.toml"))?;
    let tools = bundles
        .get("bundles")
        .and_then(toml::Value::as_table)
        .and_then(|table| table.get("vcf_downstream"))
        .and_then(toml::Value::as_table)
        .and_then(|table| table.get("tools"))
        .and_then(toml::Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|value| value.as_str().map(ToOwned::to_owned))
        .collect::<BTreeSet<_>>();
    let domain_stages = walk_paths(&workspace.path("domain/vcf/stages"))?
        .into_iter()
        .filter_map(|path| {
            (path.extension().and_then(|ext| ext.to_str()) == Some("yaml"))
                .then(|| path.file_stem().and_then(|value| value.to_str()).map(ToOwned::to_owned))
                .flatten()
        })
        .collect::<BTreeSet<_>>();
    let vcf_downstream_enabled =
        domain_stages.contains("phasing") || domain_stages.contains("imputation");
    if !vcf_downstream_enabled {
        return success_line(
            "vcf downstream bundle coverage: SKIP (no downstream phasing/imputation stages)",
        );
    }
    let phasing_required =
        BTreeSet::from(["beagle".to_string(), "eagle".to_string(), "shapeit5".to_string()]);
    let imputation_required = BTreeSet::from([
        "beagle".to_string(),
        "impute5".to_string(),
        "minimac4".to_string(),
        "glimpse".to_string(),
    ]);
    let mut errors = Vec::new();
    if tools.is_disjoint(&phasing_required) {
        errors.push(format!(
            "vcf_downstream bundle requires at least one phasing tool from {phasing_required:?}"
        ));
    }
    if tools.is_disjoint(&imputation_required) {
        errors.push(format!(
            "vcf_downstream bundle requires at least one imputation tool from {imputation_required:?}"
        ));
    }
    if errors.is_empty() {
        return success_line("vcf downstream bundle coverage: OK");
    }
    failure_lines("vcf downstream bundle coverage: FAILED", &errors)
}
