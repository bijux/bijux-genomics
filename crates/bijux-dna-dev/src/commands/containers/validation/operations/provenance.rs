use super::{
    env_or_empty, failure_lines, fs, load_toml, missing_container_label_markers, read_json,
    read_utf8, registry_tool_id, success_line, table_bool, table_string, BTreeMap, BTreeSet,
    ContainerCommandOutcome, Context, PathBuf, ProcessRunner, Regex, Result, Workspace,
};

pub(super) fn git_show_file(workspace: &Workspace, revision: &str, path: &str) -> Result<String> {
    let output = ProcessRunner::new(workspace).run_owned(
        "git",
        &[
            "-C".to_string(),
            workspace.root.display().to_string(),
            "show".to_string(),
            format!("{revision}:{path}"),
        ],
    )?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        Ok(String::new())
    }
}

pub(super) fn walk_paths(root: &std::path::Path) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    if !root.exists() {
        return Ok(out);
    }
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir)
            .with_context(|| format!("read {}", dir.display()))?
            .filter_map(std::result::Result::ok)
        {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                out.push(path);
            }
        }
    }
    Ok(out)
}

pub(in super::super::super) fn check_build_provenance(
    workspace: &Workspace,
) -> Result<ContainerCommandOutcome> {
    let registry_path = workspace.path("configs/ci/registry/tool_registry_vcf_downstream.toml");
    if !registry_path.exists() {
        return success_line("build-provenance: OK (no downstream registry)");
    }
    let data = load_toml(&registry_path)?;
    let rows = data
        .get("tools")
        .and_then(toml::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut promoted = BTreeSet::new();
    for row in &rows {
        let Some(row) = row.as_table() else {
            continue;
        };
        if table_string(row, "status") == "production" {
            let tool = registry_tool_id(row);
            if !tool.is_empty() {
                promoted.insert(tool);
            }
        }
    }

    let hex64 = Regex::new(r"^[0-9a-f]{64}$").expect("regex");
    let hex40 = Regex::new(r"^[0-9a-f]{40}$").expect("regex");
    let mut errors = Vec::new();
    for row in rows {
        let Some(row) = row.as_table() else {
            continue;
        };
        if !table_bool(row, "container") {
            continue;
        }
        let tool = registry_tool_id(row);
        let dockerfile = table_string(row, "dockerfile");
        let apptainer_def = table_string(row, "apptainer_def");
        for (kind, rel_path) in [("dockerfile", dockerfile), ("apptainer def", apptainer_def)] {
            if rel_path.is_empty() {
                continue;
            }
            let path = workspace.path(&rel_path);
            if !path.exists() {
                errors.push(format!("{tool}: missing {kind} {rel_path}"));
                continue;
            }
            let text = read_utf8(&path)?;
            let missing_labels = missing_container_label_markers(&text);
            if !missing_labels.is_empty() {
                errors.push(format!(
                    "{tool}: {kind} missing OCI metadata labels {}",
                    missing_labels.join(", ")
                ));
            }
            if text.contains("/opt/bijux/VERSION.json") || text.contains("bijux-tool-info") {
                errors.push(format!(
                    "{tool}: {kind} still embeds duplicated self-report metadata; use OCI labels as the canonical metadata surface"
                ));
            }
        }
    }

    let artifacts = workspace.path("artifacts/containers");
    if artifacts.exists() && !promoted.is_empty() {
        for tool in promoted {
            let manifest_path = artifacts.join(format!("{tool}.json"));
            if !manifest_path.exists() {
                errors.push(format!(
                    "{tool}: missing manifest artifact {}",
                    manifest_path.display()
                ));
                continue;
            }
            let payload = if let Ok(payload) = read_json(&manifest_path) {
                payload
            } else {
                errors.push(format!(
                    "{tool}: invalid json in {}",
                    manifest_path.display()
                ));
                continue;
            };
            if payload.get("status").and_then(serde_json::Value::as_str) != Some("ok") {
                errors.push(format!("{tool}: manifest status is not ok"));
                continue;
            }
            for key in [
                "builder",
                "built_at_utc",
                "git_sha",
                "versions_toml_sha256",
                "tool_source_url",
                "tool_source_hash",
                "build_script_sha256",
            ] {
                if payload
                    .get(key)
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default()
                    .trim()
                    .is_empty()
                {
                    errors.push(format!("{tool}: manifest missing provenance key '{key}'"));
                }
            }
            let versions_sha = payload
                .get("versions_toml_sha256")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .trim()
                .to_string();
            if !versions_sha.is_empty() && !hex64.is_match(&versions_sha) {
                errors.push(format!("{tool}: versions_toml_sha256 must be 64 hex chars"));
            }
            let git_sha = payload
                .get("git_sha")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .trim()
                .to_string();
            if !git_sha.is_empty() && git_sha != "unknown" && !hex40.is_match(&git_sha) {
                errors.push(format!("{tool}: git_sha must be 40 hex chars or 'unknown'"));
            }
            let source_hash = payload
                .get("tool_source_hash")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .trim()
                .to_string();
            if !source_hash.is_empty() && source_hash != "unknown" && !hex64.is_match(&source_hash)
            {
                errors.push(format!("{tool}: tool_source_hash must be 64 hex chars"));
            }
            let script_hash = payload
                .get("build_script_sha256")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .trim()
                .to_string();
            if !script_hash.is_empty() && !hex64.is_match(&script_hash) {
                errors.push(format!("{tool}: build_script_sha256 must be 64 hex chars"));
            }
        }
    }

    if errors.is_empty() {
        return success_line("build-provenance: OK");
    }
    failure_lines("build-provenance: failed", &errors)
}

pub(in super::super::super) fn check_digest_changes_on_version_change(
    workspace: &Workspace,
) -> Result<ContainerCommandOutcome> {
    let head_versions = load_toml(&workspace.path("containers/versions/versions.toml"))?;
    let head_lock = read_json(&workspace.path("containers/versions/lock.json"))?;
    let head_digest = head_lock
        .get("items")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|row| {
            let tool = row
                .get("tool")
                .and_then(serde_json::Value::as_str)?
                .trim()
                .to_string();
            let digest = row
                .get("resolved_image_digest")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .trim()
                .to_string();
            Some((tool, digest))
        })
        .collect::<BTreeMap<_, _>>();

    let prev_rev_output = ProcessRunner::new(workspace).run_owned(
        "git",
        &[
            "-C".to_string(),
            workspace.root.display().to_string(),
            "rev-parse".to_string(),
            "--verify".to_string(),
            "HEAD^".to_string(),
        ],
    )?;
    if !prev_rev_output.status.success() {
        return success_line("digest/version coupling: SKIP (no previous commit)");
    }
    let prev_rev = String::from_utf8_lossy(&prev_rev_output.stdout)
        .trim()
        .to_string();
    let prev_versions_text =
        git_show_file(workspace, &prev_rev, "containers/versions/versions.toml")?;
    let prev_lock_text = git_show_file(workspace, &prev_rev, "containers/versions/lock.json")?;
    if prev_versions_text.is_empty() || prev_lock_text.is_empty() {
        return success_line("digest/version coupling: SKIP (previous lock/version file missing)");
    }
    let prev_versions: toml::Value = toml::from_str(&prev_versions_text)?;
    let prev_lock: serde_json::Value = serde_json::from_str(&prev_lock_text)?;
    let prev_digest = prev_lock
        .get("items")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|row| {
            let tool = row
                .get("tool")
                .and_then(serde_json::Value::as_str)?
                .trim()
                .to_string();
            let digest = row
                .get("resolved_image_digest")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .trim()
                .to_string();
            Some((tool, digest))
        })
        .collect::<BTreeMap<_, _>>();

    let in_ci = !env_or_empty("CI").is_empty();
    let Some(head_tables) = head_versions.as_table() else {
        return success_line("digest/version coupling: OK");
    };
    let prev_tables = prev_versions.as_table().cloned().unwrap_or_default();
    let mut errors = Vec::new();
    for (tool, row) in head_tables {
        let Some(row) = row.as_table() else {
            continue;
        };
        let now_version = table_string(row, "version");
        let prev_version = prev_tables
            .get(tool)
            .and_then(toml::Value::as_table)
            .map(|table| table_string(table, "version"))
            .unwrap_or_default();
        if prev_version.is_empty() || now_version == prev_version {
            continue;
        }
        let previous_digest = prev_digest.get(tool).cloned().unwrap_or_default();
        let current_digest = head_digest.get(tool).cloned().unwrap_or_default();
        if current_digest.is_empty() {
            if in_ci {
                errors.push(format!(
                    "{tool}: version changed {prev_version} -> {now_version} but current lock digest is empty"
                ));
            }
        } else if !previous_digest.is_empty() && previous_digest == current_digest {
            errors.push(format!(
                "{tool}: version changed {prev_version} -> {now_version} but digest did not change ({current_digest})"
            ));
        }
    }

    if errors.is_empty() {
        return success_line("digest/version coupling: OK");
    }
    failure_lines("digest/version coupling: failed", &errors)
}

pub(in super::super::super) fn check_digest_output_policy(
    workspace: &Workspace,
) -> Result<ContainerCommandOutcome> {
    let containers_root = workspace.path("containers");
    let versions_root = workspace.path("containers/versions");
    let mut errors = Vec::new();
    for path in walk_paths(&containers_root)? {
        let rel = workspace.rel(&path).display().to_string();
        let name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default();
        let forbidden_name = path.extension().and_then(|ext| ext.to_str()) == Some("digest")
            || path.extension().and_then(|ext| ext.to_str()) == Some("sha256")
            || name.contains("digests") && name.ends_with(".json");
        if forbidden_name && !path.starts_with(&versions_root) {
            errors.push(format!(
                "generated digest artifacts must not live under containers/ tree: {rel}"
            ));
        }
    }

    let latest_regex = Regex::new(r":[Ll][Aa][Tt][Ee][Ss][Tt]\b").expect("regex");
    for base in [
        workspace.path("containers/docs"),
        workspace.path("containers"),
        workspace.path("docs/30-operations"),
    ] {
        for path in walk_paths(&base)? {
            if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
                continue;
            }
            for (index, line) in read_utf8(&path)?.lines().enumerate() {
                if latest_regex.is_match(line) {
                    errors.push(format!(
                        "{}:{}: floating ':latest' reference is forbidden",
                        workspace.rel(&path).display(),
                        index + 1
                    ));
                }
            }
        }
    }

    let lock_path = workspace.path("containers/versions/lock.json");
    if lock_path.exists() {
        let lock = read_json(&lock_path)?;
        for row in lock
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
            let status = row
                .get("status")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .trim()
                .to_string();
            let digest = row
                .get("resolved_image_digest")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .trim()
                .to_string();
            if status == "production" && !digest.is_empty() && !digest.starts_with("sha256:") {
                errors.push(format!(
                    "lock.json: {tool} resolved_image_digest must be sha256:* when present"
                ));
            }
        }
    }

    if errors.is_empty() {
        return success_line("digest output policy: OK");
    }
    failure_lines("digest output policy failed:", &errors)
}

pub(in super::super::super) fn check_runtime_tool_digest_recording(
    workspace: &Workspace,
) -> Result<ContainerCommandOutcome> {
    let stage_file = workspace.path("crates/bijux-dna-stages-vcf/src/pipeline.rs");
    let stage_text = read_utf8(&stage_file)?;
    let runtime_contract =
        workspace.path("crates/bijux-dna-runtime/tests/contracts/manifest_integrity.rs");
    let runtime_text = read_utf8(&runtime_contract)?;
    let mut errors = Vec::new();
    for marker in [
        "\"tool_digest\": resolve_tool_digest",
        "\"tool_digest\": tool_digest",
    ] {
        if !stage_text.contains(marker) {
            errors.push(format!(
                "{} missing marker `{marker}`",
                workspace.rel(&stage_file).display()
            ));
        }
    }
    if !runtime_text.contains("image_digest") {
        errors.push(format!(
            "{} missing image_digest contract checks",
            workspace.rel(&runtime_contract).display()
        ));
    }
    if errors.is_empty() {
        return success_line("runtime tool digest recording: OK");
    }
    failure_lines("runtime tool digest recording: FAILED", &errors)
}
