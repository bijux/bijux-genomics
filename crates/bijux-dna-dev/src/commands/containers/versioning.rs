#![allow(clippy::too_many_lines)]

use super::{
    all_registry_paths, anyhow, append_toml_table, container_version_deprecations_path,
    env_or_empty, failure_lines, fs, git_is_shallow_repository, git_last_modified_timestamp,
    governed_container_file_ids, load_toml, lock_items_by_tool, out_path_arg, parse_date,
    production_registry_paths, read_lock_json, read_utf8, registry_deprecations_path, run_argv,
    run_argv_with_env, set_registry_status, set_versions_status, sha256_hex, success_line,
    table_string, tool_versions, write_utf8, BTreeMap, BTreeSet, ContainerCommandOutcome, Context,
    Local, PathBuf, Result, Utc, VersionMapItem, Workspace,
};

fn choose_lock_build_date_utc(
    git_timestamp: &str,
    is_shallow_repository: bool,
    existing_lock: Option<&serde_json::Value>,
    current_source_sha256: &str,
) -> String {
    if !is_shallow_repository {
        return git_timestamp.to_string();
    }
    let Some(lock) = existing_lock else {
        return git_timestamp.to_string();
    };
    let existing_source_sha256 =
        lock.get("source_sha256").and_then(serde_json::Value::as_str).unwrap_or_default().trim();
    let existing_build_date_utc =
        lock.get("build_date_utc").and_then(serde_json::Value::as_str).unwrap_or_default().trim();
    if existing_source_sha256 == current_source_sha256 && !existing_build_date_utc.is_empty() {
        return existing_build_date_utc.to_string();
    }
    git_timestamp.to_string()
}

fn lock_build_date_utc(workspace: &Workspace, versions_path: &std::path::Path) -> Result<String> {
    let source_sha256 = sha256_hex(
        &fs::read(versions_path).with_context(|| format!("read {}", versions_path.display()))?,
    );
    let existing_lock = read_utf8(&workspace.path("containers/versions/lock.json"))
        .ok()
        .and_then(|payload| serde_json::from_str::<serde_json::Value>(&payload).ok());
    Ok(choose_lock_build_date_utc(
        &git_last_modified_timestamp(workspace, "containers/versions/versions.toml"),
        git_is_shallow_repository(workspace),
        existing_lock.as_ref(),
        &source_sha256,
    ))
}

pub(super) fn extract_version_map_content(workspace: &Workspace) -> Result<String> {
    let versions = tool_versions(workspace)?;
    let items = versions
        .into_iter()
        .map(|(tool, row)| VersionMapItem {
            tool,
            version: row
                .get("version")
                .and_then(toml::Value::as_str)
                .unwrap_or_default()
                .to_string(),
            status: row
                .get("status")
                .and_then(toml::Value::as_str)
                .unwrap_or("production")
                .to_string(),
            source: row.get("source").and_then(toml::Value::as_str).unwrap_or_default().to_string(),
            source_sha256: row
                .get("source_sha256")
                .and_then(toml::Value::as_str)
                .unwrap_or_default()
                .to_string(),
            pinned_commit: row
                .get("pinned_commit")
                .and_then(toml::Value::as_str)
                .unwrap_or_default()
                .to_string(),
            date_pinned: row
                .get("date_pinned")
                .and_then(toml::Value::as_str)
                .unwrap_or_default()
                .to_string(),
        })
        .collect::<Vec<_>>();
    Ok(format!(
        "{}\n",
        serde_json::to_string_pretty(&serde_json::json!({
            "schema_version": "bijux.container.version_map.v1",
            "source": "containers/versions/versions.toml",
            "items": items,
        }))?
    ))
}

pub(super) fn extract_version_map(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    let usage =
        "Usage: cargo run -p bijux-dna-dev -- containers run extract-version-map -- [<output-path>]";
    if matches!(args, [single] if single == "--help" || single == "-h") {
        return success_line(usage);
    }
    let out = out_path_arg(workspace, args, "artifacts/containers/version_map.json", usage)?;
    write_utf8(&out, &extract_version_map_content(workspace)?)?;
    success_line(format!("generated {}", out.display()))
}

pub(super) fn generate_versions_index_sha_content(workspace: &Workspace) -> Result<String> {
    let versions_dir = workspace.path("containers/versions");
    let mut rows = Vec::new();
    for entry in fs::read_dir(&versions_dir)
        .with_context(|| format!("read {}", versions_dir.display()))?
        .filter_map(std::result::Result::ok)
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = path.file_name().and_then(|name| name.to_str()).unwrap_or_default();
        if name == "index.sha256" {
            continue;
        }
        let digest =
            sha256_hex(&fs::read(&path).with_context(|| format!("read {}", path.display()))?);
        rows.push((name.to_string(), digest));
    }
    rows.sort();
    let payload = rows
        .into_iter()
        .map(|(name, digest)| format!("{digest}  {name}"))
        .collect::<Vec<_>>()
        .join("\n");
    Ok(format!("{payload}\n"))
}

pub(super) fn generate_versions_index_sha(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    let usage =
        "Usage: cargo run -p bijux-dna-dev -- containers run generate-versions-index-sha -- [<output-path>]";
    if matches!(args, [single] if single == "--help" || single == "-h") {
        return success_line(usage);
    }
    let out = out_path_arg(workspace, args, "containers/versions/index.sha256", usage)?;
    write_utf8(&out, &generate_versions_index_sha_content(workspace)?)?;
    success_line(format!("generated {}", out.display()))
}

pub(super) fn check_versions_index_sha(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let expected = workspace.path("containers/versions/index.sha256");
    if read_utf8(&expected)? != generate_versions_index_sha_content(workspace)? {
        return Ok(ContainerCommandOutcome::failure(
            "versions index sha drift: regenerate with cargo run -p bijux-dna-dev -- containers run generate-versions-index-sha\n",
        ));
    }
    success_line("versions index sha: OK")
}

pub(super) fn check_lock_change_discipline(
    workspace: &Workspace,
) -> Result<ContainerCommandOutcome> {
    if env_or_empty("CI").is_empty() {
        return success_line("lock change discipline: SKIP (CI-only gate)");
    }
    let previous = std::process::Command::new("git")
        .arg("-C")
        .arg(&workspace.root)
        .args(["rev-parse", "--verify", "HEAD^"])
        .output()
        .with_context(|| "resolve previous commit".to_string())?;
    if !previous.status.success() {
        return success_line("lock change discipline: SKIP (no previous commit)");
    }
    let diff = std::process::Command::new("git")
        .arg("-C")
        .arg(&workspace.root)
        .args([
            "diff",
            "--name-only",
            "HEAD^..HEAD",
            "--",
            "containers/versions/versions.toml",
            "containers/versions/lock.json",
        ])
        .output()
        .with_context(|| "inspect lock discipline diff".to_string())?;
    let changed = String::from_utf8_lossy(&diff.stdout);
    let has_versions =
        changed.lines().any(|line| line.trim() == "containers/versions/versions.toml");
    let has_lock = changed.lines().any(|line| line.trim() == "containers/versions/lock.json");
    if has_versions && !has_lock {
        return Ok(ContainerCommandOutcome::failure(
            "lock change discipline: versions.toml changed but lock.json did not\n",
        ));
    }
    if !has_versions && has_lock {
        return Ok(ContainerCommandOutcome::failure(
            "lock change discipline: lock.json changed without versions.toml change\n",
        ));
    }
    success_line("lock change discipline: OK")
}

pub(super) fn check_lock_schema(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let lock = read_lock_json(workspace)?;
    let mut errors = Vec::new();
    for key in [
        "schema_version",
        "source",
        "source_sha256",
        "build_date_utc",
        "builder_platform",
        "generator_script",
        "generator_sha256",
        "items",
    ] {
        if lock.get(key).is_none() {
            errors.push(format!("missing top-level key: {key}"));
        }
    }
    if lock.get("schema_version").and_then(serde_json::Value::as_str)
        != Some("bijux.container.version_lock.v3")
    {
        errors.push("schema_version must be bijux.container.version_lock.v3".to_string());
    }
    match lock.get("items").and_then(serde_json::Value::as_array) {
        Some(items) if !items.is_empty() => {
            let mut seen = BTreeSet::new();
            for (index, row) in items.iter().enumerate() {
                let Some(row_obj) = row.as_object() else {
                    errors.push(format!("items[{index}] must be object"));
                    continue;
                };
                for key in [
                    "tool",
                    "version",
                    "status",
                    "source",
                    "entry_sha256",
                    "resolved_image_digest",
                    "resolved_sif_sha256",
                    "sif_digest_sha256",
                    "frontend_resolved_sif_sha256",
                    "frontend_sif_digest_sha256",
                    "frontend_smoke_version_output_sha256",
                ] {
                    if !row_obj.contains_key(key) {
                        errors.push(format!("items[{index}] missing key: {key}"));
                    }
                }
                let tool =
                    row.get("tool").and_then(serde_json::Value::as_str).unwrap_or_default().trim();
                if tool.is_empty() {
                    errors.push(format!("items[{index}] has empty tool"));
                } else if !seen.insert(tool.to_string()) {
                    errors.push(format!("duplicate tool in lock items: {tool}"));
                }
            }
        }
        _ => errors.push("items must be non-empty list".to_string()),
    }
    if errors.is_empty() {
        return success_line("lock schema: OK");
    }
    failure_lines("lock schema: failed", &errors)
}

pub(super) fn check_version_completeness(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let known = tool_versions(workspace)?.into_keys().collect::<BTreeSet<_>>();
    let missing =
        governed_container_file_ids(workspace)?.difference(&known).cloned().collect::<Vec<_>>();
    if missing.is_empty() {
        return success_line("container versions completeness: OK");
    }
    let mut errors = Vec::new();
    for tool in missing {
        errors.push(format!("missing {tool} in containers/versions/versions.toml"));
    }
    failure_lines("container versions completeness check failed:", &errors)
}

pub(super) fn check_version_hash_pin(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let mut errors = Vec::new();
    for (tool, row) in tool_versions(workspace)? {
        let source = table_string(&row, "source");
        if source.is_empty() {
            errors.push(format!("{tool}: missing source URL"));
            continue;
        }
        if !source.starts_with("http://") && !source.starts_with("https://") {
            errors.push(format!("{tool}: source must be explicit http(s) URL"));
        }
        let version = table_string(&row, "version");
        if version.is_empty() || matches!(version.as_str(), "0.0.0" | "planned" | "unknown") {
            errors.push(format!(
                "{tool}: version must be concrete and must not be placeholder ({version})"
            ));
        }
        let source_sha = table_string(&row, "source_sha256");
        let pin = table_string(&row, "pinned_commit");
        if source_sha.is_empty() && pin.is_empty() {
            errors.push(format!("{tool}: missing source_sha256 or pinned_commit"));
        }
        if !source_sha.is_empty()
            && (source_sha.len() != 64 || !source_sha.chars().all(|ch| ch.is_ascii_hexdigit()))
        {
            errors.push(format!("{tool}: source_sha256 must be 64 hex chars"));
        }
        if !pin.is_empty() {
            if matches!(pin.to_ascii_lowercase().as_str(), "pending" | "unknown") {
                errors.push(format!("{tool}: pinned_commit must not be pending/unknown"));
            } else if !matches!(pin.len(), 7 | 40) {
                errors.push(format!("{tool}: pinned_commit must be short(7) or full(40) git hash"));
            }
        }
    }
    if errors.is_empty() {
        return success_line("version hash pin: OK");
    }
    failure_lines("version hash pin check failed:", &errors)
}

pub(super) fn check_version_immutability(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    if env_or_empty("CI").is_empty() {
        return success_line("version immutability: SKIP (CI-only gate)");
    }
    let previous = std::process::Command::new("git")
        .arg("-C")
        .arg(&workspace.root)
        .args(["rev-parse", "--verify", "HEAD^"])
        .output()
        .with_context(|| "resolve previous commit".to_string())?;
    if !previous.status.success() {
        return success_line("version immutability: SKIP (no previous commit)");
    }
    let previous_ref = String::from_utf8_lossy(&previous.stdout).trim().to_string();
    let show = std::process::Command::new("git")
        .arg("-C")
        .arg(&workspace.root)
        .args(["show", &format!("{previous_ref}:containers/versions/versions.toml")])
        .output()
        .with_context(|| "read previous versions.toml".to_string())?;
    if !show.status.success() {
        return success_line("version immutability: SKIP (no previous versions.toml)");
    }
    let previous_value: toml::Value =
        toml::from_str(String::from_utf8_lossy(&show.stdout).as_ref())
            .with_context(|| "parse previous containers/versions/versions.toml".to_string())?;
    let mut previous_rows = BTreeMap::new();
    if let Some(table) = previous_value.as_table() {
        for (tool, row) in table {
            if let Some(row) = row.as_table() {
                previous_rows.insert(tool.clone(), row.clone());
            }
        }
    }
    let current_rows = tool_versions(workspace)?;
    let mut errors = Vec::new();
    for (tool, previous_row) in previous_rows {
        let Some(current_row) = current_rows.get(&tool) else {
            continue;
        };
        let previous_status = table_string(&previous_row, "status");
        let current_status = {
            let value = table_string(current_row, "status");
            if value.is_empty() {
                previous_status.clone()
            } else {
                value
            }
        };
        let previous_version = table_string(&previous_row, "version");
        let current_version = table_string(current_row, "version");
        if previous_status == "production"
            && current_status == "production"
            && !previous_version.is_empty()
            && !current_version.is_empty()
            && previous_version != current_version
        {
            errors.push(format!(
                "{tool}: production version is immutable ({previous_version} -> {current_version})"
            ));
        }
    }
    if errors.is_empty() {
        return success_line("version immutability: OK");
    }
    failure_lines("version immutability: failed", &errors)
}

pub(super) fn check_version_deprecations(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let versions = tool_versions(workspace)?;
    let deps_path = container_version_deprecations_path(workspace);
    let lock_tools = lock_items_by_tool(workspace)?.into_keys().collect::<BTreeSet<_>>();
    let today = Local::now().date_naive();
    let mut errors = Vec::new();
    if deps_path.exists() {
        let value = load_toml(&deps_path)?;
        for row in
            value.get("deprecation").and_then(toml::Value::as_array).cloned().unwrap_or_default()
        {
            let Some(row) = row.as_table() else {
                continue;
            };
            let tool = table_string(row, "tool_id");
            let version = table_string(row, "version");
            let deprecated_since = table_string(row, "deprecated_since");
            let sunset_date = table_string(row, "sunset_date");
            let replacement_tool = table_string(row, "replacement_tool");
            let replacement_version = table_string(row, "replacement_version");
            let mode = table_string(row, "compatibility_mode");
            if tool.is_empty() || version.is_empty() {
                errors.push("deprecation row missing tool_id/version".to_string());
                continue;
            }
            if sunset_date.is_empty() {
                errors.push(format!("{tool}: missing required sunset_date"));
            }
            if replacement_tool.is_empty() || replacement_version.is_empty() {
                errors
                    .push(format!("{tool}: missing required replacement_tool/replacement_version"));
            }
            match versions.get(&tool) {
                None => errors.push(format!("{tool}: deprecation refers to unknown tool")),
                Some(current) => {
                    let current_version = table_string(current, "version");
                    if current_version != version {
                        errors.push(format!(
                            "{tool}: deprecation version '{version}' does not match versions.toml '{current_version}'"
                        ));
                    }
                }
            }
            if !replacement_tool.is_empty() {
                match versions.get(&replacement_tool) {
                    None => errors.push(format!(
                        "{tool}: replacement_tool '{replacement_tool}' is unknown in versions.toml"
                    )),
                    Some(current) => {
                        let current_version = table_string(current, "version");
                        if !replacement_version.is_empty()
                            && !current_version.is_empty()
                            && current_version != replacement_version
                        {
                            errors.push(format!(
                                "{tool}: replacement_version '{replacement_version}' does not match versions.toml[{replacement_tool}]='{current_version}'"
                            ));
                        }
                    }
                }
            }
            if !lock_tools.contains(&tool) {
                errors.push(format!("{tool}: missing from lock.json, breaks reproducibility"));
            }
            match (
                parse_date(&deprecated_since, "deprecated_since"),
                parse_date(&sunset_date, "sunset_date"),
            ) {
                (Ok(deprecated_since), Ok(sunset_date)) => {
                    if sunset_date <= deprecated_since {
                        errors.push(format!("{tool}: sunset_date must be after deprecated_since"));
                    }
                    if mode == "allowed" && today > sunset_date {
                        errors.push(format!(
                            "{tool}: compatibility_mode=allowed expired after {sunset_date}"
                        ));
                    }
                }
                _ => errors.push(format!("{tool}: invalid dates in deprecations.toml")),
            }
            if mode != "allowed" && mode != "blocked" {
                errors.push(format!("{tool}: compatibility_mode must be allowed|blocked"));
            }
        }
    }
    if errors.is_empty() {
        return success_line("version deprecations: OK");
    }
    failure_lines("version deprecations: failed", &errors)
}

pub(super) fn check_promotion_policy(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let policy = workspace.path("containers/docs/PROMOTION_POLICY.md");
    if !policy.is_file() {
        return Ok(ContainerCommandOutcome::failure(
            "missing containers/docs/PROMOTION_POLICY.md\n",
        ));
    }
    let text = read_utf8(&policy)?;
    let mut errors = Vec::new();
    for marker in [
        "License clarity",
        "Provenance",
        "Reproducibility",
        "Smoke quality",
        "cargo run -p bijux-dna-dev -- containers run tool-lifecycle",
        "cargo run -p bijux-dna-dev -- containers run demote",
    ] {
        if !text.contains(marker) {
            errors.push(format!("promotion policy missing marker: {marker}"));
        }
    }
    if errors.is_empty() {
        return success_line("promotion policy: OK");
    }
    failure_lines("promotion policy: failed", &errors)
}

pub(super) fn check_promotion_lock_integrity(
    workspace: &Workspace,
) -> Result<ContainerCommandOutcome> {
    if env_or_empty("CI").is_empty() {
        return success_line("promotion lock integrity: SKIP (CI-only gate)");
    }
    let lock_rows = lock_items_by_tool(workspace)?;
    let versions = tool_versions(workspace)?;
    let mut production_tools = BTreeSet::new();
    for path in production_registry_paths(workspace) {
        if !path.exists() {
            continue;
        }
        let value = load_toml(&path)?;
        for row in value.get("tools").and_then(toml::Value::as_array).cloned().unwrap_or_default() {
            let Some(row) = row.as_table() else {
                continue;
            };
            if table_string(row, "status") != "production" {
                continue;
            }
            let tool = table_string(row, "id");
            let tool = if tool.is_empty() { table_string(row, "tool_id") } else { tool };
            if !tool.is_empty() {
                production_tools.insert(tool);
            }
        }
    }
    let mut errors = Vec::new();
    for tool in production_tools {
        let Some(lock_row) = lock_rows.get(&tool) else {
            errors.push(format!("{tool}: production tool missing from lock.json"));
            continue;
        };
        let lock_version = lock_row
            .get("version")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_string();
        let version =
            versions.get(&tool).map(|row| table_string(row, "version")).unwrap_or_default();
        if lock_version != version {
            errors.push(format!(
                "{tool}: lock version '{lock_version}' != versions.toml '{version}'"
            ));
        }
        let docker_digest = lock_row
            .get("resolved_image_digest")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_string();
        let sif_digest = lock_row
            .get("resolved_sif_sha256")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_string();
        if docker_digest.is_empty() && sif_digest.is_empty() {
            errors.push(format!(
                "{tool}: promotion requires at least one locked artifact digest (docker/apptainer)"
            ));
        }
    }
    if errors.is_empty() {
        return success_line("promotion lock integrity: OK");
    }
    failure_lines("promotion lock integrity: failed", &errors)
}

pub(super) fn generate_version_lock_content(workspace: &Workspace) -> Result<String> {
    let version_map: serde_json::Value =
        serde_json::from_str(&extract_version_map_content(workspace)?)?;
    let generator_path = workspace.path("crates/bijux-dna-dev/src/commands/containers/mod.rs");
    let versions_path = workspace.path("containers/versions/versions.toml");
    let build_date_utc = lock_build_date_utc(workspace, &versions_path)?;

    let manifest_candidates =
        [workspace.path("artifacts/containers"), workspace.path("artifacts/containers/manifests")];
    let mut docker_digest_by_tool = BTreeMap::new();
    let mut apptainer_sif_sha256_by_tool = BTreeMap::new();
    let mut frontend_sif_sha256_by_tool = BTreeMap::new();
    let mut frontend_smoke_version_output_sha256_by_tool = BTreeMap::new();
    let mut size_by_tool = BTreeMap::new();
    let mut seen = BTreeSet::new();
    for base in manifest_candidates {
        if !base.exists() {
            continue;
        }
        for entry in fs::read_dir(&base)
            .with_context(|| format!("read {}", base.display()))?
            .filter_map(std::result::Result::ok)
        {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }
            let name = path.file_name().and_then(|value| value.to_str()).unwrap_or_default();
            if matches!(name, "lock.json" | "summary.json" | "report.json")
                || !seen.insert(path.clone())
            {
                continue;
            }
            let Ok(value) =
                serde_json::from_str::<serde_json::Value>(&read_utf8(&path).unwrap_or_default())
            else {
                continue;
            };
            let tool = value
                .get("tool")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .trim()
                .to_string();
            let runtime = value
                .get("runtime")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .trim()
                .to_string();
            let digest = value
                .get("resolved_image_digest")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .trim()
                .to_string();
            let size =
                value.get("image_size_bytes").and_then(serde_json::Value::as_i64).unwrap_or(0);
            if tool.is_empty() {
                continue;
            }
            if runtime.starts_with("docker") {
                docker_digest_by_tool.insert(tool.clone(), digest);
            } else if runtime == "apptainer" {
                apptainer_sif_sha256_by_tool.insert(tool.clone(), digest);
            }
            if size > 0 {
                size_by_tool.insert(tool, size);
            }
        }
    }

    let frontend_digests = workspace.path("artifacts/containers/hpc/frontend-sif-digests.json");
    if frontend_digests.is_file() {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&read_utf8(&frontend_digests)?)
        {
            if let Some(items) = value.get("items").and_then(serde_json::Value::as_array) {
                for row in items {
                    let tool = row
                        .get("tool")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or_default()
                        .trim();
                    let sha = row
                        .get("sha256")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or_default()
                        .trim();
                    if !tool.is_empty() && !sha.is_empty() {
                        frontend_sif_sha256_by_tool.insert(tool.to_string(), sha.to_string());
                    }
                }
            }
        }
    }

    let frontend_smoke_summary =
        workspace.path("artifacts/containers/hpc/frontend-smoke/summary.json");
    if frontend_smoke_summary.is_file() {
        if let Ok(value) =
            serde_json::from_str::<serde_json::Value>(&read_utf8(&frontend_smoke_summary)?)
        {
            if let Some(items) = value.get("items").and_then(serde_json::Value::as_array) {
                for row in items {
                    let tool = row
                        .get("tool")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or_default()
                        .trim();
                    let output = row
                        .get("normalized_version_output")
                        .and_then(serde_json::Value::as_str)
                        .or_else(|| row.get("version_output").and_then(serde_json::Value::as_str))
                        .unwrap_or_default()
                        .trim()
                        .to_lowercase();
                    if !tool.is_empty() && !output.is_empty() {
                        frontend_smoke_version_output_sha256_by_tool
                            .insert(tool.to_string(), sha256_hex(output.as_bytes()));
                    }
                }
            }
        }
    }

    let mut items = Vec::new();
    for row in
        version_map.get("items").and_then(serde_json::Value::as_array).cloned().unwrap_or_default()
    {
        let tool =
            row.get("tool").and_then(serde_json::Value::as_str).unwrap_or_default().to_string();
        let canonical = serde_json::to_string(&row)?;
        items.push(serde_json::json!({
            "tool": tool,
            "version": row.get("version").and_then(serde_json::Value::as_str).unwrap_or_default(),
            "status": row.get("status").and_then(serde_json::Value::as_str).unwrap_or_default(),
            "source": row.get("source").and_then(serde_json::Value::as_str).unwrap_or_default(),
            "source_sha256": row.get("source_sha256").and_then(serde_json::Value::as_str).unwrap_or_default(),
            "pinned_commit": row.get("pinned_commit").and_then(serde_json::Value::as_str).unwrap_or_default(),
            "resolved_image_digest": docker_digest_by_tool.get(&tool).cloned().unwrap_or_default(),
            "resolved_sif_sha256": apptainer_sif_sha256_by_tool.get(&tool).cloned().unwrap_or_default(),
            "sif_digest_sha256": apptainer_sif_sha256_by_tool.get(&tool).cloned().unwrap_or_default(),
            "frontend_resolved_sif_sha256": frontend_sif_sha256_by_tool.get(&tool).cloned().unwrap_or_default(),
            "frontend_sif_digest_sha256": frontend_sif_sha256_by_tool.get(&tool).cloned().unwrap_or_default(),
            "frontend_smoke_version_output_sha256": frontend_smoke_version_output_sha256_by_tool.get(&tool).cloned().unwrap_or_default(),
            "image_size_bytes": size_by_tool.get(&tool).copied().unwrap_or(0),
            "entry_sha256": sha256_hex(canonical.as_bytes()),
        }));
    }

    let output = serde_json::json!({
        "schema_version": "bijux.container.version_lock.v3",
        "source": "containers/versions/versions.toml",
        "version_map_source": "artifacts/containers/version_map.json",
        "build_manifests_source": "artifacts/containers/manifests/*.json",
        "build_date_utc": build_date_utc,
        "builder_platform": "arm64",
        "generator_script": "cargo run -p bijux-dna-dev -- containers run generate-version-lock",
        "generator_sha256": sha256_hex(&fs::read(&generator_path).with_context(|| format!("read {}", generator_path.display()))?),
        "source_sha256": sha256_hex(&fs::read(&versions_path).with_context(|| format!("read {}", versions_path.display()))?),
        "items": items,
    });
    Ok(format!("{}\n", serde_json::to_string_pretty(&output)?))
}

pub(super) fn generate_version_lock(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    let usage =
        "Usage: cargo run -p bijux-dna-dev -- containers run generate-version-lock -- [<output-path>]";
    if matches!(args, [single] if single == "--help" || single == "-h") {
        return success_line(usage);
    }
    let out = out_path_arg(workspace, args, "containers/versions/lock.json", usage)?;
    write_utf8(&out, &generate_version_lock_content(workspace)?)?;
    success_line(format!("generated {}", out.display()))
}

pub(super) fn check_version_lock(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let lock = workspace.path("containers/versions/lock.json");
    if read_utf8(&lock)? != generate_version_lock_content(workspace)? {
        return Ok(ContainerCommandOutcome::failure(
            "version lock drift: regenerate with cargo run -p bijux-dna-dev -- containers run generate-version-lock\n",
        ));
    }
    success_line("version lock: OK")
}

pub(super) fn check_version_authority(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let violations = std::process::Command::new("find")
        .arg(workspace.path("containers"))
        .args(["-type", "f", "(", "-iname", "*version*", "-o", "-iname", "*lock*", ")"])
        .output()
        .with_context(|| "scan container version/lock files".to_string())?;
    let listing = String::from_utf8_lossy(&violations.stdout);
    let forbidden = listing
        .lines()
        .map(|line| workspace.rel(&PathBuf::from(line)).display().to_string())
        .filter(|rel| rel.starts_with("containers/"))
        .filter(|rel| !rel.starts_with("containers/docs/"))
        .filter(|rel| {
            !matches!(
                rel.as_str(),
                "containers/versions/versions.toml"
                    | "containers/versions/lock.json"
                    | "containers/versions/LOCK.md"
                    | "containers/versions/index.md"
            )
        })
        .collect::<Vec<_>>();
    if !forbidden.is_empty() {
        let mut stderr =
            String::from("non-canonical version/lock files found under containers/ (use containers/versions/* only):\n");
        stderr.push_str(&forbidden.join("\n"));
        stderr.push('\n');
        return Ok(ContainerCommandOutcome::failure(stderr));
    }

    let lock: serde_json::Value =
        serde_json::from_str(&read_utf8(&workspace.path("containers/versions/lock.json"))?)?;
    let versions_path = workspace.path("containers/versions/versions.toml");
    let generator_path = workspace.path("crates/bijux-dna-dev/src/commands/containers/mod.rs");
    let mut errors = Vec::new();
    if lock
        .get("schema_version")
        .and_then(serde_json::Value::as_str)
        .is_none_or(|value| value != "bijux.container.version_lock.v3")
    {
        errors
            .push("- lock.json schema_version must be bijux.container.version_lock.v3".to_string());
    }
    if lock.get("source").and_then(serde_json::Value::as_str)
        != Some("containers/versions/versions.toml")
    {
        errors.push("- lock.json source must be containers/versions/versions.toml".to_string());
    }
    if lock
        .get("build_date_utc")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .trim()
        .is_empty()
    {
        errors.push("- lock.json must include build_date_utc".to_string());
    }
    if lock.get("builder_platform").and_then(serde_json::Value::as_str) != Some("arm64") {
        errors.push("- lock.json builder_platform must be arm64".to_string());
    }
    if lock.get("generator_script").and_then(serde_json::Value::as_str)
        != Some("cargo run -p bijux-dna-dev -- containers run generate-version-lock")
    {
        errors.push("- lock.json generator_script must reference bijux-dna-dev".to_string());
    }
    let expected_gen_sha = sha256_hex(
        &fs::read(&generator_path).with_context(|| format!("read {}", generator_path.display()))?,
    );
    if lock.get("generator_sha256").and_then(serde_json::Value::as_str)
        != Some(expected_gen_sha.as_str())
    {
        errors.push(
            "- lock.json generator_sha256 does not match bijux-dna-dev container generator"
                .to_string(),
        );
    }
    let expected_sha = sha256_hex(
        &fs::read(&versions_path).with_context(|| format!("read {}", versions_path.display()))?,
    );
    if lock.get("source_sha256").and_then(serde_json::Value::as_str) != Some(expected_sha.as_str())
    {
        errors.push("- lock.json source_sha256 does not match versions.toml".to_string());
    }
    if lock.get("items").and_then(serde_json::Value::as_array).is_none_or(std::vec::Vec::is_empty) {
        errors.push("- lock.json items must be a non-empty list".to_string());
    }

    let version_source_marker = "VERSION_SOURCE: containers/versions/versions.toml";
    for root in [workspace.path("containers/apptainer"), workspace.path("containers/docker/arm64")]
    {
        for entry in walkdir::WalkDir::new(&root)
            .into_iter()
            .filter_map(std::result::Result::ok)
            .filter(|entry| entry.file_type().is_file())
        {
            let ext = entry.path().extension().and_then(|ext| ext.to_str());
            let file_name =
                entry.path().file_name().and_then(|name| name.to_str()).unwrap_or_default();
            if ext != Some("def") && !file_name.starts_with("Dockerfile.") {
                continue;
            }
            let raw = read_utf8(entry.path()).unwrap_or_default();
            if !raw.contains(version_source_marker) {
                errors.push(format!(
                    "- version authority: missing VERSION_SOURCE marker in {}",
                    workspace.rel(entry.path()).display()
                ));
            }
        }
    }

    if errors.is_empty() {
        return success_line("version authority: OK");
    }
    failure_lines("version authority check failed:", &errors)
}

pub(super) fn parse_required_option(
    command: &str,
    options: &BTreeMap<String, String>,
    key: &str,
) -> Result<String> {
    options
        .get(key)
        .cloned()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| anyhow!("{command}: missing required option --{key}"))
}

pub(super) fn parse_named_options(
    command: &str,
    args: &[String],
) -> Result<BTreeMap<String, String>> {
    let mut options = BTreeMap::new();
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        if arg == "--help" || arg == "-h" {
            return Err(anyhow!("help"));
        }
        let Some(name) = arg.strip_prefix("--") else {
            return Err(anyhow!("{command}: unknown arg: {arg}"));
        };
        let Some(value) = args.get(index + 1) else {
            return Err(anyhow!("{command}: missing value for --{name}"));
        };
        if value.starts_with("--") {
            return Err(anyhow!("{command}: missing value for --{name}"));
        }
        options.insert(name.to_string(), value.clone());
        index += 2;
    }
    Ok(options)
}

pub(super) fn regenerate_lifecycle_outputs(workspace: &Workspace) -> Result<()> {
    let commands = [
        ["containers", "run", "generate-version-lock"].as_slice(),
        ["containers", "run", "generate-index"].as_slice(),
        ["containers", "run", "generate-license-metadata"].as_slice(),
    ];
    for command in commands {
        let argv = [
            vec![
                "cargo".to_string(),
                "run".to_string(),
                "-q".to_string(),
                "-p".to_string(),
                "bijux-dna-dev".to_string(),
                "--".to_string(),
            ],
            command.iter().map(|value| (*value).to_string()).collect::<Vec<_>>(),
        ]
        .concat();
        let outcome = run_argv(workspace, &argv)?;
        if !outcome.is_success() {
            return Err(anyhow!(
                "failed to regenerate lifecycle output with `{}`: {}",
                argv.join(" "),
                outcome.stderr.trim()
            ));
        }
    }
    let domain_lock = run_argv(
        workspace,
        &[
            "cargo".to_string(),
            "run".to_string(),
            "-q".to_string(),
            "-p".to_string(),
            "bijux-dna-dev".to_string(),
            "--".to_string(),
            "domain".to_string(),
            "run".to_string(),
            "lock-registry".to_string(),
        ],
    )?;
    if !domain_lock.is_success() {
        return Err(anyhow!(
            "failed to regenerate domain registry lock: {}",
            domain_lock.stderr.trim()
        ));
    }
    Ok(())
}

pub(super) fn promote_tool(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    let usage =
        "Usage: cargo run -p bijux-dna-dev -- containers run promote -- --tool <id> --to <experimental|production>";
    if matches!(args, [single] if single == "--help" || single == "-h") {
        return success_line(usage);
    }
    let options = match parse_named_options("promote", args) {
        Ok(options) => options,
        Err(error) if error.to_string() == "help" => return success_line(usage),
        Err(error) => {
            return Ok(ContainerCommandOutcome::failure(format!("{error}\n")));
        }
    };
    let tool = parse_required_option("promote", &options, "tool")
        .map_err(|error| anyhow!("{usage}\n{error}"))?;
    let to_status = parse_required_option("promote", &options, "to")
        .map_err(|error| anyhow!("{usage}\n{error}"))?;
    if to_status != "experimental" && to_status != "production" {
        return Ok(ContainerCommandOutcome::failure(
            "--to must be experimental|production\n".to_string(),
        ));
    }
    let lock_rows = lock_items_by_tool(workspace)?;
    let Some(lock_row) = lock_rows.get(&tool) else {
        return Ok(ContainerCommandOutcome::failure(format!(
            "tool '{tool}' not present in containers/versions/lock.json; ad-hoc promotion is forbidden\n"
        )));
    };
    let versions = tool_versions(workspace)?;
    let Some(version_row) = versions.get(&tool) else {
        return Ok(ContainerCommandOutcome::failure(format!(
            "tool '{tool}' missing in containers/versions/versions.toml\n"
        )));
    };
    let lock_version = lock_row
        .get("version")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .trim()
        .to_string();
    let version = table_string(version_row, "version");
    if lock_version != version {
        return Ok(ContainerCommandOutcome::failure(format!(
            "tool '{tool}' version mismatch lock='{lock_version}' versions.toml='{version}'\n"
        )));
    }
    if to_status == "production" {
        let docker_digest = lock_row
            .get("resolved_image_digest")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_string();
        let sif_digest = lock_row
            .get("resolved_sif_sha256")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_string();
        if docker_digest.is_empty() && sif_digest.is_empty() {
            return Ok(ContainerCommandOutcome::failure(format!(
                "tool '{tool}' cannot be promoted to production without locked artifact digest\n"
            )));
        }
        let sbom_path = workspace.path(&format!("artifacts/containers/sbom/{tool}"));
        if !sbom_path.exists() {
            return Ok(ContainerCommandOutcome::failure(format!(
                "tool '{tool}' cannot be promoted to production without sbom artifacts at {}\n",
                sbom_path.display()
            )));
        }
    }
    set_registry_status(&all_registry_paths(workspace), &tool, &to_status)?;
    set_versions_status(workspace, &tool, &to_status)?;
    regenerate_lifecycle_outputs(workspace)?;
    if to_status == "production" {
        let sbom_check = run_argv_with_env(
            workspace,
            &[
                "cargo".to_string(),
                "run".to_string(),
                "-q".to_string(),
                "-p".to_string(),
                "bijux-dna-dev".to_string(),
                "--".to_string(),
                "containers".to_string(),
                "run".to_string(),
                "check-sbom-artifacts".to_string(),
            ],
            &[("REQUIRE_PROMOTED_SBOM".to_string(), "1".to_string())],
        )?;
        if !sbom_check.is_success() {
            return Ok(sbom_check);
        }
    }
    success_line(format!("promoted {tool} -> {to_status}"))
}

pub(super) fn demote_tool(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    let usage =
        "Usage: cargo run -p bijux-dna-dev -- containers run demote -- --tool <id> --stage <domain.stage> --reason <text> --removal-after <YYYY-MM-DD>";
    if matches!(args, [single] if single == "--help" || single == "-h") {
        return success_line(usage);
    }
    let options = match parse_named_options("demote", args) {
        Ok(options) => options,
        Err(error) if error.to_string() == "help" => return success_line(usage),
        Err(error) => return Ok(ContainerCommandOutcome::failure(format!("{error}\n"))),
    };
    let tool = parse_required_option("demote", &options, "tool")
        .map_err(|error| anyhow!("{usage}\n{error}"))?;
    let stage = parse_required_option("demote", &options, "stage")
        .map_err(|error| anyhow!("{usage}\n{error}"))?;
    let reason = parse_required_option("demote", &options, "reason")
        .map_err(|error| anyhow!("{usage}\n{error}"))?;
    let removal_after = parse_required_option("demote", &options, "removal-after")
        .map_err(|error| anyhow!("{usage}\n{error}"))?;
    parse_date(&removal_after, "removal-after")?;
    if !lock_items_by_tool(workspace)?.contains_key(&tool) {
        return Ok(ContainerCommandOutcome::failure(format!(
            "tool '{tool}' not present in containers/versions/lock.json; ad-hoc demotion is forbidden\n"
        )));
    }
    set_registry_status(&production_registry_paths(workspace), &tool, "experimental")?;
    set_versions_status(workspace, &tool, "experimental")?;
    append_toml_table(
        &registry_deprecations_path(workspace),
        &format!(
            "[[deprecations]]\ntool_id = \"{tool}\"\nstage = \"{stage}\"\ndeprecated_since = \"{}\"\nremoval_after = \"{removal_after}\"\nrationale = \"{}\"\n",
            Utc::now().date_naive().format("%Y-%m-%d"),
            reason.replace('"', "\\\""),
        ),
        "# schema_version = 1\n# owner = bijux-dna-policies\n# purpose = Contract config for configs/ci/registry/deprecations.toml\n# authority = bijux-dna-policies\n# stability = stable\n\n",
    )?;
    regenerate_lifecycle_outputs(workspace)?;
    success_line(format!("demoted {tool} -> experimental and appended deprecation entry"))
}

pub(super) fn deprecate_version(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    let usage = "Usage: cargo run -p bijux-dna-dev -- containers run deprecate-version -- --tool <id> --version <semver> --rationale <text> --sunset-date <YYYY-MM-DD> --replacement-tool <id> --replacement-version <semver> [--compatibility-mode allowed|blocked]";
    if matches!(args, [single] if single == "--help" || single == "-h") {
        return success_line(usage);
    }
    let options = match parse_named_options("deprecate-version", args) {
        Ok(options) => options,
        Err(error) if error.to_string() == "help" => return success_line(usage),
        Err(error) => return Ok(ContainerCommandOutcome::failure(format!("{error}\n"))),
    };
    let tool = parse_required_option("deprecate-version", &options, "tool")
        .map_err(|error| anyhow!("{usage}\n{error}"))?;
    let version = parse_required_option("deprecate-version", &options, "version")
        .map_err(|error| anyhow!("{usage}\n{error}"))?;
    let rationale = parse_required_option("deprecate-version", &options, "rationale")
        .map_err(|error| anyhow!("{usage}\n{error}"))?;
    let sunset_date = parse_required_option("deprecate-version", &options, "sunset-date")
        .map_err(|error| anyhow!("{usage}\n{error}"))?;
    let replacement_tool = parse_required_option("deprecate-version", &options, "replacement-tool")
        .map_err(|error| anyhow!("{usage}\n{error}"))?;
    let replacement_version =
        parse_required_option("deprecate-version", &options, "replacement-version")
            .map_err(|error| anyhow!("{usage}\n{error}"))?;
    let compatibility_mode =
        options.get("compatibility-mode").cloned().unwrap_or_else(|| "allowed".to_string());
    if compatibility_mode != "allowed" && compatibility_mode != "blocked" {
        return Ok(ContainerCommandOutcome::failure(
            "--compatibility-mode must be allowed|blocked\n".to_string(),
        ));
    }
    parse_date(&sunset_date, "sunset-date")?;
    let versions = tool_versions(workspace)?;
    if !versions.contains_key(&tool) {
        return Ok(ContainerCommandOutcome::failure(format!(
            "unknown tool in versions.toml: {tool}\n"
        )));
    }
    if !versions.contains_key(&replacement_tool) {
        return Ok(ContainerCommandOutcome::failure(format!(
            "unknown replacement_tool in versions.toml: {replacement_tool}\n"
        )));
    }
    let path = container_version_deprecations_path(workspace);
    if path.exists() {
        let value = load_toml(&path)?;
        for row in
            value.get("deprecation").and_then(toml::Value::as_array).cloned().unwrap_or_default()
        {
            let Some(row) = row.as_table() else {
                continue;
            };
            if table_string(row, "tool_id") == tool && table_string(row, "version") == version {
                return Ok(ContainerCommandOutcome::failure(format!(
                    "deprecation already exists for {tool}@{version}\n"
                )));
            }
        }
    }
    append_toml_table(
        &path,
        &format!(
            "[[deprecation]]\ntool_id = \"{tool}\"\nversion = \"{version}\"\ndeprecated_since = \"{}\"\nsunset_date = \"{sunset_date}\"\nreplacement_tool = \"{replacement_tool}\"\nreplacement_version = \"{replacement_version}\"\nrationale = \"{}\"\ncompatibility_mode = \"{compatibility_mode}\"\n",
            Utc::now().date_naive().format("%Y-%m-%d"),
            rationale.replace('"', "\\\""),
        ),
        "# schema_version = 1\n# owner = bijux-dna-platform\n\n",
    )?;
    regenerate_lifecycle_outputs(workspace)?;
    success_line(format!("deprecated {tool}@{version} (compatibility_mode={compatibility_mode})"))
}

pub(super) fn tool_lifecycle(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    let usage = "Usage:\n  cargo run -p bijux-dna-dev -- containers run tool-lifecycle -- --tool <id> --to experimental\n  cargo run -p bijux-dna-dev -- containers run tool-lifecycle -- --tool <id> --to stable\n\nNotes:\n- `stable` is the lifecycle alias for production container status.\n- Status changes must be done through this command (no manual edits).\n";
    if matches!(args, [single] if single == "--help" || single == "-h") {
        return success_line(usage);
    }
    let options = match parse_named_options("tool-lifecycle", args) {
        Ok(options) => options,
        Err(error) if error.to_string() == "help" => return success_line(usage),
        Err(error) => return Ok(ContainerCommandOutcome::failure(format!("{error}\n"))),
    };
    let tool = parse_required_option("tool-lifecycle", &options, "tool")
        .map_err(|error| anyhow!("{usage}{error}"))?;
    let to = parse_required_option("tool-lifecycle", &options, "to")
        .map_err(|error| anyhow!("{usage}{error}"))?;
    let resolved = match to.as_str() {
        "experimental" => "experimental",
        "stable" => "production",
        _ => {
            return Ok(ContainerCommandOutcome::failure(
                "--to must be experimental|stable\n".to_string(),
            ))
        }
    };
    promote_tool(workspace, &["--tool".to_string(), tool, "--to".to_string(), resolved.to_string()])
}

#[cfg(test)]
mod tests {
    use super::choose_lock_build_date_utc;

    #[test]
    fn prefers_git_timestamp_when_repository_is_not_shallow() {
        let current_source_sha256 = "abc123";
        let existing_lock = serde_json::json!({
            "source_sha256": current_source_sha256,
            "build_date_utc": "2026-04-25T23:10:27+02:00"
        });

        let chosen = choose_lock_build_date_utc(
            "2026-04-29T15:35:57+02:00",
            false,
            Some(&existing_lock),
            current_source_sha256,
        );

        assert_eq!(chosen, "2026-04-29T15:35:57+02:00");
    }

    #[test]
    fn preserves_existing_lock_date_when_shallow_and_source_is_unchanged() {
        let current_source_sha256 = "abc123";
        let existing_lock = serde_json::json!({
            "source_sha256": current_source_sha256,
            "build_date_utc": "2026-04-25T23:10:27+02:00"
        });

        let chosen = choose_lock_build_date_utc(
            "2026-04-29T15:35:57+02:00",
            true,
            Some(&existing_lock),
            current_source_sha256,
        );

        assert_eq!(chosen, "2026-04-25T23:10:27+02:00");
    }

    #[test]
    fn falls_back_to_git_timestamp_when_shallow_lock_is_stale() {
        let existing_lock = serde_json::json!({
            "source_sha256": "old-source",
            "build_date_utc": "2026-04-25T23:10:27+02:00"
        });

        let chosen = choose_lock_build_date_utc(
            "2026-04-29T15:35:57+02:00",
            true,
            Some(&existing_lock),
            "new-source",
        );

        assert_eq!(chosen, "2026-04-29T15:35:57+02:00");
    }
}
