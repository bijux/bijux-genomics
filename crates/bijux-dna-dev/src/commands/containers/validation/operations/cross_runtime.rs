use super::{
    env_or_default, env_or_empty, failure_lines, load_runtime_manifest_rows,
    normalized_version_output, registry_tool_id, registry_tool_rows, success_line, table_string,
    BTreeMap, ContainerCommandOutcome, PathBuf, Regex, Result, Workspace,
};

pub(in super::super::super) fn check_cross_runtime_representative(
    workspace: &Workspace,
) -> Result<ContainerCommandOutcome> {
    let docker_dir = PathBuf::from(env_or_default(
        "DOCKER_DIR",
        &workspace
            .path("artifacts/containers/docker-arm64")
            .display()
            .to_string(),
    ));
    let apptainer_dir = PathBuf::from(env_or_default(
        "APPTAINER_DIR",
        &workspace
            .path("artifacts/containers/apptainer")
            .display()
            .to_string(),
    ));
    check_cross_runtime_representative_at_paths(workspace, docker_dir, apptainer_dir)
}

pub(in super::super::super) fn check_cross_runtime_representative_at_paths(
    _workspace: &Workspace,
    docker_dir: PathBuf,
    apptainer_dir: PathBuf,
) -> Result<ContainerCommandOutcome> {
    if !docker_dir.exists() || !apptainer_dir.exists() {
        if env_or_empty("CI").is_empty() {
            return success_line(format!(
                "cross-runtime representative: SKIP (missing runtime dirs docker='{}' apptainer='{}')",
                docker_dir.display(),
                apptainer_dir.display()
            ));
        }
        return Ok(ContainerCommandOutcome::failure(
            "cross-runtime representative: missing runtime dirs\n",
        ));
    }

    let docker_rows = load_runtime_manifest_rows(&docker_dir)?;
    let apptainer_rows = load_runtime_manifest_rows(&apptainer_dir)?;
    let shared = docker_rows
        .keys()
        .filter(|tool| apptainer_rows.contains_key(*tool))
        .cloned()
        .collect::<Vec<_>>();
    if shared.len() < 5 {
        if env_or_empty("CI").is_empty() {
            return success_line(format!(
                "cross-runtime representative: SKIP (<5 shared tools, found {})",
                shared.len()
            ));
        }
        return Ok(ContainerCommandOutcome::failure(format!(
            "cross-runtime representative: need >=5 shared tools, found {}\n",
            shared.len()
        )));
    }

    let mut errors = Vec::new();
    let representative = shared.into_iter().take(5).collect::<Vec<_>>();
    for tool in &representative {
        let docker_row = &docker_rows[tool];
        let apptainer_row = &apptainer_rows[tool];
        if docker_row.get("status").and_then(serde_json::Value::as_str) != Some("ok")
            || apptainer_row
                .get("status")
                .and_then(serde_json::Value::as_str)
                != Some("ok")
        {
            errors.push(format!(
                "{tool}: non-ok status docker={} apptainer={}",
                docker_row
                    .get("status")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default(),
                apptainer_row
                    .get("status")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default()
            ));
            continue;
        }
        let docker_version = normalized_version_output(docker_row);
        let apptainer_version = normalized_version_output(apptainer_row);
        if docker_version.is_empty()
            || apptainer_version.is_empty()
            || docker_version != apptainer_version
        {
            errors.push(format!(
                "{tool}: version_output mismatch docker='{docker_version}' apptainer='{apptainer_version}'"
            ));
        }
    }

    if errors.is_empty() {
        return success_line(format!(
            "cross-runtime representative: OK ({})",
            representative.join(", ")
        ));
    }
    failure_lines("cross-runtime representative: FAILED", &errors)
}

pub(in super::super::super) fn check_cross_runtime_smoke(
    workspace: &Workspace,
) -> Result<ContainerCommandOutcome> {
    let docker_dir = PathBuf::from(env_or_default(
        "DOCKER_DIR",
        &workspace
            .path("artifacts/containers/docker-arm64")
            .display()
            .to_string(),
    ));
    let apptainer_dir = PathBuf::from(env_or_default(
        "APPTAINER_DIR",
        &workspace
            .path("artifacts/containers/apptainer")
            .display()
            .to_string(),
    ));
    check_cross_runtime_smoke_at_paths(workspace, docker_dir, apptainer_dir)
}

pub(in super::super::super) fn check_cross_runtime_smoke_at_paths(
    workspace: &Workspace,
    docker_dir: PathBuf,
    apptainer_dir: PathBuf,
) -> Result<ContainerCommandOutcome> {
    if !docker_dir.exists() || !apptainer_dir.exists() {
        if env_or_empty("CI").is_empty() {
            return success_line("cross-runtime smoke: SKIP (missing runtime dirs)");
        }
        return Ok(ContainerCommandOutcome::failure(format!(
            "cross-runtime smoke: missing runtime dirs docker='{}' apptainer='{}'\n",
            docker_dir.display(),
            apptainer_dir.display()
        )));
    }

    let docker_rows = load_runtime_manifest_rows(&docker_dir)?;
    let apptainer_rows = load_runtime_manifest_rows(&apptainer_dir)?;
    let mut expected_regexes = BTreeMap::new();
    for row in registry_tool_rows(workspace)? {
        let tool = registry_tool_id(&row);
        let regex = table_string(&row, "expected_version_regex");
        if !tool.is_empty() && !regex.is_empty() {
            expected_regexes.insert(tool, regex);
        }
    }

    let shared = docker_rows
        .keys()
        .filter(|tool| apptainer_rows.contains_key(*tool))
        .cloned()
        .collect::<Vec<_>>();
    if shared.is_empty() {
        return Ok(ContainerCommandOutcome::failure(
            "cross-runtime smoke: no shared tool manifests to compare\n",
        ));
    }

    let mut errors = Vec::new();
    for tool in shared {
        let docker_row = &docker_rows[&tool];
        let apptainer_row = &apptainer_rows[&tool];
        if docker_row.get("status").and_then(serde_json::Value::as_str) != Some("ok")
            || apptainer_row
                .get("status")
                .and_then(serde_json::Value::as_str)
                != Some("ok")
        {
            errors.push(format!(
                "{tool}: non-ok status docker={} apptainer={}",
                docker_row
                    .get("status")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default(),
                apptainer_row
                    .get("status")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default()
            ));
            continue;
        }
        let docker_version = normalized_version_output(docker_row);
        let apptainer_version = normalized_version_output(apptainer_row);
        if docker_version.is_empty() || apptainer_version.is_empty() {
            errors.push(format!("{tool}: missing version_output in one runtime"));
        } else if docker_version != apptainer_version {
            errors.push(format!(
                "{tool}: version_output mismatch docker='{docker_version}' apptainer='{apptainer_version}'"
            ));
        }

        let regex_text = expected_regexes
            .get(&tool)
            .cloned()
            .unwrap_or_else(|| r"v?[0-9]+\.[0-9]+([.-][0-9A-Za-z]+)?".to_string());
        match Regex::new(&regex_text) {
            Ok(regex) => {
                if !docker_version.is_empty() && !regex.is_match(&docker_version) {
                    errors.push(format!(
                        "{tool}: docker version_output does not match expected pattern '{regex_text}'"
                    ));
                }
                if !apptainer_version.is_empty() && !regex.is_match(&apptainer_version) {
                    errors.push(format!(
                        "{tool}: apptainer version_output does not match expected pattern '{regex_text}'"
                    ));
                }
            }
            Err(error) => errors.push(format!(
                "{tool}: invalid expected_version_regex '{regex_text}': {error}"
            )),
        }

        for key in [
            "help_actual_exit_code",
            "minimal_actual_exit_code",
            "negative_actual_exit_code",
        ] {
            let docker_value = docker_row
                .get(key)
                .map(serde_json::Value::to_string)
                .unwrap_or_default();
            let apptainer_value = apptainer_row
                .get(key)
                .map(serde_json::Value::to_string)
                .unwrap_or_default();
            if docker_value != apptainer_value {
                errors.push(format!(
                    "{tool}: {key} mismatch docker={} apptainer={}",
                    docker_row.get(key).unwrap_or(&serde_json::Value::Null),
                    apptainer_row.get(key).unwrap_or(&serde_json::Value::Null)
                ));
            }
        }
    }

    if errors.is_empty() {
        return success_line(format!(
            "container runtime parity: OK ({}) shared tools",
            docker_rows
                .keys()
                .filter(|tool| apptainer_rows.contains_key(*tool))
                .count()
        ));
    }
    failure_lines("container runtime parity: FAILED", &errors)
}
