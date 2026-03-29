use super::*;

pub(super) fn check_apptainer_cache_policy(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let policy = workspace.path("configs/ci/tools/apptainer_cache_policy.toml");
    if !policy.is_file() {
        return Ok(ContainerCommandOutcome::failure(format!(
            "apptainer cache policy: missing {}\n",
            policy.display()
        )));
    }
    success_line("apptainer cache policy: OK")
}

pub(super) fn check_apptainer_frontend_reproducibility(
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

pub(super) fn check_apptainer_frontend_security(
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

pub(super) fn check_apptainer_frontend_smoke_proof(
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

pub(super) fn check_apptainer_frontend_version_output_lock(
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

pub(super) fn check_apptainer_hardening(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let tool_status = tool_status_manifest(workspace)?;
    let required_labels = canonical_container_label_keys();
    let mut errors = Vec::new();
    let version_re = Regex::new(r"org\.opencontainers\.image\.version\s+([^\s]+)").expect("regex");
    let from_re = Regex::new(r"(?m)^\s*From:\s+(.+?)\s*$").expect("regex");
    let interactive_re = Regex::new(r"\b(read -p|select |dialog|whiptail)\b").expect("regex");
    let umask_re = Regex::new(r"(?m)^\s*umask\s+0?22\s*$").expect("regex");
    let allowed_from_re = Regex::new(r"^(ubuntu|debian|python|quay\.io/)").expect("regex");
    for path in apptainer_def_paths(workspace) {
        let rel = workspace.rel(&path).display().to_string();
        let tool_id = path
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or_default();
        let status = tool_status
            .get(tool_id)
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());
        let text = read_utf8(&path)?;
        let head = text.lines().take(24).collect::<Vec<_>>().join("\n");
        for marker in [
            format!("# Tool ID: {tool_id}"),
            "# Version policy:".to_string(),
            "# Upstream source:".to_string(),
            "# Build date policy:".to_string(),
        ] {
            if !head.contains(&marker) {
                errors.push(format!("{rel}: missing standard header marker '{marker}'"));
            }
        }
        for label in required_labels {
            if !text.contains(label) {
                errors.push(format!("{rel}: missing label {label}"));
            }
        }
        for (alias, keys) in [
            ("tool", vec!["org.opencontainers.image.tool", "tool"]),
            (
                "version",
                vec!["org.opencontainers.image.version", "version"],
            ),
            ("source", vec!["org.opencontainers.image.source", "source"]),
            (
                "license_ref",
                vec!["org.opencontainers.image.licenses", "license_ref"],
            ),
            (
                "build_date",
                vec!["org.opencontainers.image.created", "build_date"],
            ),
            (
                "git_sha",
                vec!["org.opencontainers.image.revision", "git_sha"],
            ),
        ] {
            if !keys.iter().any(|key| text.contains(key)) {
                errors.push(format!("{rel}: missing label contract key '{alias}'"));
            }
        }
        if text.contains("%environment") {
            let env = text
                .split("%environment")
                .nth(1)
                .and_then(|body| body.split("\n%").next())
                .unwrap_or_default();
            for env_line in ["PATH=", "LC_ALL=", "TZ="] {
                if !env.contains(env_line) {
                    errors.push(format!("{rel}: %environment missing {env_line}"));
                }
            }
            if env.contains("/Users/") || env.contains("/home/") {
                errors.push(format!("{rel}: %environment contains user-specific path"));
            }
        } else {
            errors.push(format!("{rel}: missing %environment section"));
        }
        if text.contains("%post") {
            let post = text
                .split("%post")
                .nth(1)
                .and_then(|body| body.split("\n%").next())
                .unwrap_or_default();
            let first_non_empty = post
                .lines()
                .map(str::trim)
                .find(|line| !line.is_empty())
                .unwrap_or_default();
            if !first_non_empty.contains("set -eux") {
                errors.push(format!("{rel}: %post must start with set -eux"));
            }
            if !umask_re.is_match(post) {
                errors.push(format!("{rel}: %post must set deterministic umask 022"));
            }
            if interactive_re.is_match(post) {
                errors.push(format!(
                    "{rel}: %post contains interactive prompt constructs"
                ));
            }
            if (post.contains("wget ") || post.contains("curl "))
                && !text.contains("NETWORK_SOURCE_VERIFIED_BY_LOCK")
                && !post.contains("sha256sum")
            {
                errors.push(format!(
                    "{rel}: network download without checksum policy marker"
                ));
            }
            if post.contains("apt-get") && !post.contains("rm -rf /var/lib/apt/lists/*") {
                errors.push(format!(
                    "{rel}: apt usage requires cleanup of /var/lib/apt/lists/*"
                ));
            }
            if post.contains("latest")
                || post.contains("main")
                || post.contains("master")
                || post.contains("HEAD")
            {
                // This script was originally handled by a separate post-pin check, so keep the
                // hardening surface focused on hardening-only findings.
            }
        } else {
            errors.push(format!("{rel}: missing %post section"));
        }
        if let Some(captures) = version_re.captures(&text) {
            let value = captures
                .get(1)
                .map(|value| value.as_str().trim().trim_matches('"').to_ascii_lowercase())
                .unwrap_or_default();
            if status == "production"
                && matches!(
                    value.as_str(),
                    "latest" | "latest-pinned" | "main" | "master" | "head" | "unknown" | ""
                )
            {
                errors.push(format!(
                    "{rel}: floating/unknown image.version '{value}' is forbidden for production tool"
                ));
            }
        }
        if let Some(captures) = from_re.captures(&text) {
            let from_line = captures
                .get(1)
                .map(|value| value.as_str().trim())
                .unwrap_or_default();
            if !from_line.contains("@sha256:") {
                errors.push(format!("{rel}: base image must be digest pinned"));
            }
            if !allowed_from_re.is_match(from_line) {
                errors.push(format!(
                    "{rel}: base image repo must follow policy (ubuntu/debian/python/quay.io/*)"
                ));
            }
        }
        if text.contains("/opt/bijux/VERSION.json") || text.contains("bijux-tool-info") {
            errors.push(format!(
                "{rel}: duplicate in-image self-report metadata is forbidden; publish metadata must flow through OCI labels"
            ));
        }
        if text.contains("chmod 777") {
            errors.push(format!("{rel}: chmod 777 forbidden for runtime UID safety"));
        }
        let has_help_doc = text
            .split("%help")
            .nth(1)
            .is_some_and(|help| !help.trim().is_empty());
        if text.contains("%runscript") {
            let run = text
                .split("%runscript")
                .nth(1)
                .and_then(|body| body.split("\n%").next())
                .unwrap_or_default();
            if !run.contains("--help") && !has_help_doc {
                errors.push(format!(
                    "{rel}: runscript/help must provide predictable --help behavior"
                ));
            }
        } else {
            errors.push(format!("{rel}: missing %runscript section"));
        }
    }
    if errors.is_empty() {
        return success_line("apptainer hardening: OK");
    }
    failure_lines("apptainer hardening: failed", &errors)
}

pub(super) fn check_apptainer_post_pins(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    if env_or_empty("CI").is_empty() {
        return success_line("apptainer post pin policy: SKIP (CI-only gate)");
    }
    let versions = tool_versions(workspace)?;
    let policy = load_toml(&workspace.path("configs/ci/tools/hpc_frontend_build_policy.toml"))?;
    let host = command_hostname();
    let mut errors = Vec::new();
    if let Some(pattern) = policy
        .get("compute_hostname_regex")
        .and_then(toml::Value::as_str)
    {
        let pattern = pattern.trim();
        if !pattern.is_empty()
            && !host.is_empty()
            && Regex::new(pattern).is_ok_and(|regex| regex.is_match(&host))
        {
            errors.push(format!(
                "CI runner host '{host}' matches compute node policy; %post checks refused outside frontend/login node"
            ));
        }
    }
    let floating_re = Regex::new(r"\b(latest|main|master|HEAD)\b").expect("regex");
    let download_re = Regex::new(r"\b(curl|wget)\b").expect("regex");
    for path in apptainer_def_paths(workspace) {
        let rel = workspace.rel(&path).display().to_string();
        let tool = path
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_string();
        let text = read_utf8(&path)?;
        let post = text
            .split("%post")
            .nth(1)
            .and_then(|body| body.split("\n%").next())
            .unwrap_or_default()
            .to_string();
        if post.trim().is_empty() {
            errors.push(format!("{rel}: missing %post section"));
            continue;
        }
        if floating_re.is_match(&post) {
            errors.push(format!(
                "{rel}: %post contains floating ref (latest/main/master/HEAD)"
            ));
        }
        if download_re.is_match(&post) {
            let has_sha = post.contains("sha256sum") || post.contains("shasum -a 256");
            let row = versions.get(&tool);
            let source_sha = row
                .map(|row| table_string(row, "source_sha256"))
                .unwrap_or_default();
            let pin = row
                .map(|row| table_string(row, "pinned_commit"))
                .unwrap_or_default();
            if !has_sha {
                errors.push(format!(
                    "{rel}: %post downloads without checksum verification command"
                ));
            }
            if source_sha.is_empty() && pin.is_empty() {
                errors.push(format!(
                    "{rel}: tool downloads in %post but versions.toml has neither source_sha256 nor pinned_commit"
                ));
            }
        }
    }
    if errors.is_empty() {
        return success_line("apptainer post pin policy: OK");
    }
    failure_lines("apptainer post pin policy: failed", &errors)
}

pub(super) fn check_apptainer_version_label_sync(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    if env_or_empty("CI").is_empty() {
        return success_line("apptainer version label sync: SKIP (CI-only gate)");
    }
    let versions = tool_versions(workspace)?;
    let mut errors = Vec::new();
    let version_re =
        Regex::new(r"org\.opencontainers\.image\.version\s+([^\n\r]+)").expect("regex");
    for path in apptainer_def_paths(workspace) {
        let rel = workspace.rel(&path).display().to_string();
        let tool = path
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_string();
        let text = read_utf8(&path)?;
        let Some(row) = versions.get(&tool) else {
            errors.push(format!("{rel}: missing versions.toml entry"));
            continue;
        };
        let expected = table_string(row, "version");
        let Some(captures) = version_re.captures(&text) else {
            errors.push(format!(
                "{rel}: missing org.opencontainers.image.version label"
            ));
            continue;
        };
        let label_value = captures
            .get(1)
            .map(|value| {
                value
                    .as_str()
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string()
            })
            .unwrap_or_default();
        let placeholder = matches!(
            label_value.as_str(),
            "${TOOL_VERSION}" | "$TOOL_VERSION" | "unknown" | "planned" | "latest-pinned"
        ) || label_value.ends_with("-planned");
        if !placeholder && label_value != expected {
            errors.push(format!(
                "{rel}: label version '{label_value}' != versions.toml '{expected}'"
            ));
        }
    }
    if errors.is_empty() {
        return success_line("apptainer version label sync: OK");
    }
    failure_lines("apptainer version label sync: failed", &errors)
}

pub(super) fn check_bijux_apptainer_built(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    if env_or_empty("CI").is_empty() {
        return success_line("bijux apptainer built: SKIP (CI-only gate)");
    }
    let summary_path = workspace.path("artifacts/containers/summary.json");
    if !summary_path.exists() {
        return Ok(ContainerCommandOutcome::failure(
            "bijux apptainer built: missing artifacts/containers/summary.json\n",
        ));
    }
    let summary = read_json(&summary_path)?;
    let rows = summary
        .get("items")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|row| row.get("runtime").and_then(serde_json::Value::as_str) == Some("apptainer"))
        .filter_map(|row| {
            let tool = row
                .get("tool")
                .and_then(serde_json::Value::as_str)
                .map(|tool| tool.trim().to_string())?;
            Some((tool, row))
        })
        .collect::<BTreeMap<_, _>>();
    let bijux_defs = apptainer_def_paths(workspace)
        .into_iter()
        .filter(|path| path.starts_with(workspace.path("containers/apptainer/shared")))
        .filter_map(|path| {
            path.file_stem()
                .and_then(|name| name.to_str())
                .map(ToOwned::to_owned)
        })
        .collect::<Vec<_>>();
    let mut errors = Vec::new();
    for tool in bijux_defs {
        let Some(row) = rows.get(&tool) else {
            errors.push(format!("{tool}: missing apptainer summary row"));
            continue;
        };
        if row.get("status").and_then(serde_json::Value::as_str) != Some("ok") {
            errors.push(format!("{tool}: apptainer status is not ok"));
            continue;
        }
        let manifest_path = PathBuf::from(
            row.get("manifest")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default(),
        );
        if !manifest_path.exists() {
            errors.push(format!(
                "{tool}: missing manifest at {}",
                manifest_path.display()
            ));
            continue;
        }
        let manifest = read_json(&manifest_path)?;
        let sif_sha = manifest
            .get("resolved_image_digest")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_string();
        if sif_sha.is_empty() {
            errors.push(format!(
                "{tool}: missing resolved_image_digest (sif sha256) in manifest"
            ));
        }
    }
    if errors.is_empty() {
        return success_line("bijux apptainer built: OK");
    }
    failure_lines("bijux apptainer built: failed", &errors)
}

pub(super) fn generate_local_apptainer_digests(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    let usage =
        "Usage: cargo run -p bijux-dna-dev -- containers run generate-local-apptainer-digests -- [<output-path>]";
    if matches!(args, [single] if single == "--help" || single == "-h") {
        return success_line(usage);
    }
    let out = out_path_arg(
        workspace,
        args,
        "artifacts/containers/hpc/local-sif-digests.json",
        usage,
    )?;
    let sif_dir = std::env::var("SIF_DIR").map_or_else(
        |_| workspace.path("artifacts/containers/apptainer/sif"),
        PathBuf::from,
    );
    let mut rows = Vec::new();
    if sif_dir.exists() {
        let mut paths = fs::read_dir(&sif_dir)
            .with_context(|| format!("read {}", sif_dir.display()))?
            .filter_map(std::result::Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("sif"))
            .collect::<Vec<_>>();
        paths.sort();
        for path in paths {
            let tool = path
                .file_stem()
                .and_then(|name| name.to_str())
                .unwrap_or_default();
            rows.push(serde_json::json!({
                "tool": tool,
                "sif_path": path.display().to_string(),
                "sha256": sha256_hex(&fs::read(&path).with_context(|| format!("read {}", path.display()))?),
            }));
        }
    }
    write_utf8(
        &out,
        &json_string_pretty(&serde_json::json!({
            "schema_version": "bijux.local.sif_digests.v1",
            "items": rows,
        }))?,
    )?;
    success_line(out.display().to_string())
}

pub(super) fn compare_frontend_local_sif_hash(
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

pub(super) fn check_missing_images(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let coverage = validation::check_tool_container_coverage(workspace)?;
    if !coverage.is_success() {
        return Ok(coverage);
    }
    let bundles = validation::check_toolkit_bundles(workspace)?;
    if !bundles.is_success() {
        return Ok(bundles);
    }
    success_line("missing images gate: OK")
}

pub(super) fn check_non_bijux_sources(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let sources_doc = workspace.path("containers/apptainer/shared/NON_BIJUX_SOURCES.md");
    if !sources_doc.exists() {
        return Ok(ContainerCommandOutcome::failure(format!(
            "missing required provenance index: {}\n",
            sources_doc.display()
        )));
    }
    let defs = apptainer_tool_ids(workspace);
    let text = read_utf8(&sources_doc)?;
    let row_re = Regex::new(
        r"\|\s*`([^`]+)`\s*\|\s*`([^`]+)`\s*\|\s*(.+?)\s*\|\s*(\S+)\s*\|\s*`([^`]+)`\s*\|\s*`([^`]+)`\s*\|\s*(.+?)\s*\|",
    )
    .expect("regex");
    let mut rows = BTreeMap::new();
    for line in text.lines() {
        let Some(captures) = row_re.captures(line) else {
            continue;
        };
        rows.insert(
            captures
                .get(1)
                .map(|value| value.as_str().to_string())
                .unwrap_or_default(),
            (
                captures
                    .get(2)
                    .map(|value| value.as_str().to_string())
                    .unwrap_or_default(),
                captures
                    .get(3)
                    .map(|value| value.as_str().to_string())
                    .unwrap_or_default(),
                captures
                    .get(4)
                    .map(|value| value.as_str().to_string())
                    .unwrap_or_default(),
                captures
                    .get(5)
                    .map(|value| value.as_str().to_string())
                    .unwrap_or_default(),
                captures
                    .get(6)
                    .map(|value| value.as_str().to_string())
                    .unwrap_or_default(),
                captures
                    .get(7)
                    .map(|value| value.as_str().to_string())
                    .unwrap_or_default(),
            ),
        );
    }
    let checksum_re = Regex::new(r"^[0-9a-f]{64}$").expect("regex");
    let mut errors = Vec::new();
    for tool_id in &defs {
        let Some((def_path, why_non_bijux, upstream, license_field, checksum, patching_rules)) =
            rows.get(tool_id)
        else {
            errors.push(format!("{tool_id}: missing row in NON_BIJUX_SOURCES.md"));
            continue;
        };
        let expected_path = format!("containers/apptainer/shared/{tool_id}.def");
        if def_path != &expected_path {
            errors.push(format!(
                "{tool_id}: def path mismatch, expected {expected_path}, got {def_path}"
            ));
        }
        if !upstream.starts_with("http://") && !upstream.starts_with("https://") {
            errors.push(format!("{tool_id}: upstream_source must be URL"));
        }
        if why_non_bijux.trim().is_empty() {
            errors.push(format!("{tool_id}: why_non_bijux must be non-empty"));
        }
        if license_field.trim().is_empty() {
            errors.push(format!("{tool_id}: upstream_license must be non-empty"));
        }
        if patching_rules.trim().is_empty() {
            errors.push(format!("{tool_id}: patching_rules must be non-empty"));
        }
        if checksum.starts_with("sha256:") {
            let digest = checksum.trim_start_matches("sha256:");
            if digest != "pending" && !checksum_re.is_match(digest) {
                errors.push(format!(
                    "{tool_id}: upstream_checksum must be sha256:<64hex> or sha256:pending"
                ));
            }
        } else {
            errors.push(format!(
                "{tool_id}: upstream_checksum must start with sha256:"
            ));
        }
    }
    for tool_id in rows.keys() {
        if !defs.contains(tool_id) {
            errors.push(format!(
                "{tool_id}: listed in NON_BIJUX_SOURCES.md but no .def exists"
            ));
        }
    }
    if errors.is_empty() {
        return success_line("non-bijux source coverage: OK");
    }
    failure_lines("non-bijux source coverage check failed:", &errors)
}

pub(super) fn check_owners(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let owners_path = workspace.path("containers/OWNERS.toml");
    if !owners_path.exists() {
        return Ok(ContainerCommandOutcome::failure(
            "missing containers/OWNERS.toml\n",
        ));
    }
    let owners_data = load_toml(&owners_path)?;
    let owner_rows = owners_data
        .get("owner")
        .and_then(toml::Value::as_array)
        .cloned()
        .unwrap_or_default();
    if owner_rows.is_empty() {
        return Ok(ContainerCommandOutcome::failure(
            "containers/OWNERS.toml has no [[owner]] rows\n",
        ));
    }
    let mut rows = Vec::new();
    for row in owner_rows {
        let Some(row) = row.as_table() else {
            continue;
        };
        let tool_id = table_string(row, "tool_id");
        let team = table_string(row, "team");
        let contact = table_string(row, "contact");
        if tool_id.is_empty() || team.is_empty() || contact.is_empty() {
            return Ok(ContainerCommandOutcome::failure(
                "each [[owner]] row must include tool_id, team, contact\n",
            ));
        }
        if tool_id == "*" {
            return Ok(ContainerCommandOutcome::failure(
                "containers/OWNERS.toml: wildcard tool_id='*' is not allowed; map each tool explicitly\n",
            ));
        }
        rows.push((tool_id, team));
    }
    let tool_ids = tool_status_manifest(workspace)?
        .into_keys()
        .collect::<Vec<_>>();
    let mut errors = Vec::new();
    for tool_id in tool_ids {
        let matches = rows
            .iter()
            .filter(|(pattern, _)| pattern == &tool_id)
            .count();
        if matches != 1 {
            errors.push(format!(
                "{tool_id}: expected exactly one owner match, got {matches}"
            ));
        }
    }
    if errors.is_empty() {
        return success_line("container owners: OK");
    }
    failure_lines("container owners check failed:", &errors)
}

pub(super) fn check_registry_vs_defs(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let mut registry_ids = BTreeSet::new();
    let mut registry_container_ids = BTreeSet::new();
    for row in registry_tool_rows(workspace)? {
        let tool_id = table_string(&row, "id");
        let tool_id = if tool_id.is_empty() {
            table_string(&row, "tool_id")
        } else {
            tool_id
        };
        if tool_id.is_empty() {
            continue;
        }
        registry_ids.insert(tool_id.clone());
        let status = table_string(&row, "status");
        if table_bool(&row, "container") && matches!(status.as_str(), "production" | "experimental")
        {
            registry_container_ids.insert(tool_id);
        }
    }
    let mut retired = BTreeSet::new();
    let retired_doc = workspace.path("containers/docs/RETIRED_DEFS.md");
    if retired_doc.exists() {
        for line in read_utf8(&retired_doc)?.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("| `") {
                let cols = trimmed
                    .trim_matches('|')
                    .split('|')
                    .map(str::trim)
                    .collect::<Vec<_>>();
                if let Some(tool) = cols.first() {
                    let tool = tool.trim_matches('`').trim().to_string();
                    if !tool.is_empty() {
                        retired.insert(tool);
                    }
                }
            }
        }
    }
    let def_ids = docker_tool_ids(workspace)?
        .into_iter()
        .chain(apptainer_tool_ids(workspace))
        .collect::<BTreeSet<_>>();
    let orphans = def_ids
        .difference(&registry_ids)
        .filter(|tool| !retired.contains(*tool))
        .cloned()
        .collect::<Vec<_>>();
    let missing = registry_container_ids
        .difference(&def_ids)
        .cloned()
        .collect::<Vec<_>>();
    let mut errors = Vec::new();
    if !orphans.is_empty() {
        errors.push("registry-vs-defs: defs without registry entry (and not retired):".to_string());
        errors.extend(orphans.into_iter().map(|tool| format!("- {tool}")));
    }
    if !missing.is_empty() {
        errors.push("registry-vs-defs: registry container tools missing defs:".to_string());
        errors.extend(missing.into_iter().map(|tool| format!("- {tool}")));
    }
    if errors.is_empty() {
        return success_line(format!(
            "registry-vs-defs: OK ({} defs, {} registry container tools)",
            def_ids.len(),
            registry_container_ids.len()
        ));
    }
    failure_lines("registry-vs-defs: failed", &errors)
}

pub(super) fn run_bijux_with_env(
    workspace: &Workspace,
    args: &[String],
    overrides: &[(&str, String)],
) -> Result<ContainerCommandOutcome> {
    let mut envs = artifact_env(workspace)?;
    for (key, value) in overrides {
        envs.push(((*key).to_string(), value.clone()));
    }
    let argv = [bijux_command_prefix(), args.to_vec()].concat();
    run_argv_with_env(workspace, &argv, &envs)
}

pub(super) fn run_argv(workspace: &Workspace, argv: &[String]) -> Result<ContainerCommandOutcome> {
    run_argv_with_env(workspace, argv, &[])
}

pub(super) fn run_argv_with_env(
    workspace: &Workspace,
    argv: &[String],
    envs: &[(String, String)],
) -> Result<ContainerCommandOutcome> {
    let (program, args) = argv
        .split_first()
        .context("container command requires a program")?;
    run_program_with_env(workspace, program, args, envs)
}

pub(super) fn run_program_with_env(
    workspace: &Workspace,
    program: &str,
    args: &[String],
    envs: &[(String, String)],
) -> Result<ContainerCommandOutcome> {
    let runner = ProcessRunner::new(workspace);
    let output = runner.run_owned_with_env(program, args, envs)?;
    Ok(ContainerCommandOutcome::from_output(output))
}

pub(super) fn artifact_env(workspace: &Workspace) -> Result<Vec<(String, String)>> {
    let artifact_root = artifact_root_path(workspace)?;
    let cargo_target_dir = artifact_root.join("target");
    let cargo_home = artifact_root.join("cargo/home");
    let tmpdir = artifact_root.join("tmp");
    for dir in [&artifact_root, &cargo_target_dir, &cargo_home, &tmpdir] {
        bijux_dna_infra::ensure_dir(dir).with_context(|| format!("create {}", dir.display()))?;
    }
    Ok(vec![
        (
            "ARTIFACT_ROOT".to_string(),
            artifact_root.display().to_string(),
        ),
        ("ISO_ROOT".to_string(), artifact_root.display().to_string()),
        (
            "CARGO_TARGET_DIR".to_string(),
            cargo_target_dir.display().to_string(),
        ),
        ("CARGO_HOME".to_string(), cargo_home.display().to_string()),
        ("TMPDIR".to_string(), tmpdir.display().to_string()),
        ("TMP".to_string(), tmpdir.display().to_string()),
        ("TEMP".to_string(), tmpdir.display().to_string()),
    ])
}

pub(super) fn artifact_root_path(workspace: &Workspace) -> Result<PathBuf> {
    let configured = std::env::var("ARTIFACT_ROOT").unwrap_or_else(|_| "artifacts".to_string());
    let path = if PathBuf::from(&configured).is_absolute() {
        PathBuf::from(&configured)
    } else {
        workspace.root.join(&configured)
    };
    let display = path.display().to_string();
    if !display.contains("/artifacts") && !display.ends_with("artifacts") {
        return Err(anyhow!(
            "artifact root must stay under artifacts/: {display}"
        ));
    }
    Ok(path)
}

pub(super) fn primary_tools_csv(workspace: &Workspace) -> Result<String> {
    let result = run_argv(
        workspace,
        &[
            bijux_command_prefix(),
            vec![
                "registry".to_string(),
                "list-tools".to_string(),
                "--kind".to_string(),
                "primary".to_string(),
            ],
        ]
        .concat(),
    )?;
    if !result.is_success() {
        return Ok(String::new());
    }
    Ok(result
        .stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(","))
}

pub(super) fn list_tools_for_stage(workspace: &Workspace, stage: &str) -> Result<String> {
    let result = run_argv(
        workspace,
        &[
            bijux_command_prefix(),
            vec![
                "registry".to_string(),
                "list-tools".to_string(),
                "--stage".to_string(),
                stage.to_string(),
                "--kind".to_string(),
                "all".to_string(),
            ],
        ]
        .concat(),
    )?;
    if !result.is_success() {
        return Ok(String::new());
    }
    Ok(result
        .stdout
        .replace(',', "\n")
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>()
        .join(","))
}

pub(super) fn resolve_toolkit_tools(workspace: &Workspace, bundle: &str) -> Result<String> {
    let data: toml::Value = toml::from_str(&std::fs::read_to_string(
        workspace.path("configs/ci/tools/toolkit_bundles.toml"),
    )?)?;
    let tools = data
        .get("bundles")
        .and_then(|value| value.get(bundle))
        .and_then(|value| value.get("tools"))
        .and_then(toml::Value::as_array)
        .cloned()
        .unwrap_or_default();
    if tools.is_empty() {
        return Err(anyhow!("unknown or empty toolkit bundle: {bundle}"));
    }
    Ok(tools
        .into_iter()
        .filter_map(|tool| tool.as_str().map(ToOwned::to_owned))
        .collect::<Vec<_>>()
        .join(","))
}

pub(super) fn ensure_no_args(command: &str, args: &[String]) -> Result<()> {
    if args.is_empty() {
        return Ok(());
    }
    Err(anyhow!("{command} does not accept positional arguments"))
}

pub(super) fn checked_container_type() -> Result<String> {
    let container_type = env_or_default("CONTAINER_TYPE", "docker-arm64");
    match container_type.as_str() {
        "docker-arm64" | "docker-amd64" | "apptainer" => Ok(container_type),
        _ => Err(anyhow!(
            "ERROR: unsupported CONTAINER_TYPE={container_type}\nsupported: docker-arm64 | docker-amd64 | apptainer"
        )),
    }
}

pub(super) fn require_tools_or_stage(tools: &str, stage: &str) -> Result<()> {
    if tools.is_empty() && stage.is_empty() {
        return Err(anyhow!("ERROR: set TOOLS=<tool_id> or STAGE=<stage>"));
    }
    Ok(())
}

pub(super) fn required_env(key: &str) -> Result<String> {
    let value = env_or_empty(key);
    if value.is_empty() {
        return Err(anyhow!("missing required env var: {key}"));
    }
    Ok(value)
}

pub(super) fn env_or_empty(key: &str) -> String {
    std::env::var(key).unwrap_or_default()
}

pub(super) fn env_or_default(key: &str, fallback: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| fallback.to_string())
}

pub(super) fn container_artifact_dir() -> String {
    env_or_default("CONTAINER_ARTIFACT_DIR", "artifacts/containers")
}

pub(super) fn bijux_command_prefix() -> Vec<String> {
    std::env::var("BIJUX_BIN")
        .unwrap_or_else(|_| "cargo run -q --bin bijux-dna --".to_string())
        .split_whitespace()
        .map(ToOwned::to_owned)
        .collect()
}

pub(super) fn run_runtime_smoke_contract(
    workspace: &Workspace,
    runtime: &str,
    tools_csv: String,
) -> Result<ContainerCommandOutcome> {
    run_environment_smoke_for(workspace, runtime, Some(tools_csv), None)
}

pub(super) fn run_environment_prep_for(
    workspace: &Workspace,
    runtime: &str,
    tools: Option<String>,
    stage: Option<String>,
) -> Result<ContainerCommandOutcome> {
    run_environment_prep_for_with_env(workspace, runtime, tools, stage, &[])
}

pub(super) fn run_environment_prep_for_with_env(
    workspace: &Workspace,
    runtime: &str,
    tools: Option<String>,
    stage: Option<String>,
    envs: &[(String, String)],
) -> Result<ContainerCommandOutcome> {
    let mut argv = bijux_command_prefix();
    argv.extend([
        "environment".to_string(),
        "prep".to_string(),
        runtime.to_string(),
    ]);
    if let Some(stage) = stage.filter(|value| !value.is_empty()) {
        argv.push("--stage".to_string());
        argv.push(stage);
    } else if let Some(tools) = tools.filter(|value| !value.is_empty()) {
        argv.push(tools);
    } else {
        argv.push(primary_tools_csv(workspace)?);
    }
    run_argv_with_env(workspace, &argv, envs)
}

pub(super) fn run_environment_smoke_for(
    workspace: &Workspace,
    runtime: &str,
    tools: Option<String>,
    stage: Option<String>,
) -> Result<ContainerCommandOutcome> {
    run_environment_smoke_for_with_env(workspace, runtime, tools, stage, &[])
}

pub(super) fn run_environment_smoke_for_with_env(
    workspace: &Workspace,
    runtime: &str,
    tools: Option<String>,
    stage: Option<String>,
    envs: &[(String, String)],
) -> Result<ContainerCommandOutcome> {
    let mut argv = bijux_command_prefix();
    argv.extend([
        "environment".to_string(),
        "smoke".to_string(),
        runtime.to_string(),
    ]);
    if let Some(stage) = stage.filter(|value| !value.is_empty()) {
        argv.push("--stage".to_string());
        argv.push(stage);
    } else if let Some(tools) = tools.filter(|value| !value.is_empty()) {
        argv.push(tools);
    } else {
        argv.push(primary_tools_csv(workspace)?);
    }
    run_argv_with_env(workspace, &argv, envs)
}

pub(super) fn resolved_smoke_tools(workspace: &Workspace) -> Result<String> {
    let tools = env_or_empty("TOOLS");
    if !tools.is_empty() {
        return Ok(tools);
    }
    primary_tools_csv(workspace)
}

pub(super) fn compare_apptainer_smoke_modes(root: &Path) -> Result<ContainerCommandOutcome> {
    fn load_statuses(base: &Path) -> Result<BTreeMap<String, String>> {
        let mut statuses = BTreeMap::new();
        for entry in std::fs::read_dir(base).with_context(|| format!("read {}", base.display()))? {
            let path = entry?.path();
            if !path.is_file()
                || path.extension().and_then(|ext| ext.to_str()) != Some("json")
                || matches!(
                    path.file_name().and_then(|name| name.to_str()),
                    Some("report.json" | "summary.json")
                )
            {
                continue;
            }
            let payload: serde_json::Value = serde_json::from_str(
                &std::fs::read_to_string(&path)
                    .with_context(|| format!("read {}", path.display()))?,
            )
            .with_context(|| format!("parse {}", path.display()))?;
            let tool = payload
                .get("tool")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default();
            let status = payload
                .get("status")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default();
            if !tool.is_empty() {
                statuses.insert(tool.to_string(), status.to_string());
            }
        }
        Ok(statuses)
    }

    let left_dir = root.join("apptainer-bijux-run");
    let right_dir = root.join("apptainer-apptainer-run");
    if !left_dir.exists() || !right_dir.exists() {
        return Ok(ContainerCommandOutcome::failure(
            "missing smoke artifact dirs for compare\n".to_string(),
        ));
    }
    let left = load_statuses(&left_dir)?;
    let right = load_statuses(&right_dir)?;
    let missing_left = right
        .keys()
        .filter(|tool| !left.contains_key(*tool))
        .cloned()
        .collect::<Vec<_>>();
    let missing_right = left
        .keys()
        .filter(|tool| !right.contains_key(*tool))
        .cloned()
        .collect::<Vec<_>>();
    let mismatch = left
        .keys()
        .filter(|tool| right.get(*tool).is_some() && right.get(*tool) != left.get(*tool))
        .map(|tool| {
            format!(
                "{tool}:{}!={}",
                left.get(tool).cloned().unwrap_or_default(),
                right.get(tool).cloned().unwrap_or_default()
            )
        })
        .collect::<Vec<_>>();
    if missing_left.is_empty() && missing_right.is_empty() && mismatch.is_empty() {
        return Ok(ContainerCommandOutcome::success(format!(
            "smoke mode compare OK for {} tools\n",
            left.len()
        )));
    }
    let mut stdout = String::from("smoke mode mismatch detected\n");
    if !missing_left.is_empty() {
        stdout.push_str(&format!(
            "missing in bijux-run: {}\n",
            missing_left.join(",")
        ));
    }
    if !missing_right.is_empty() {
        stdout.push_str(&format!(
            "missing in apptainer-run: {}\n",
            missing_right.join(",")
        ));
    }
    if !mismatch.is_empty() {
        stdout.push_str(&format!("status mismatch: {}\n", mismatch.join(",")));
    }
    Ok(ContainerCommandOutcome::failure(stdout))
}

pub(super) fn sampled_apptainer_defs(workspace: &Workspace, seed: &str, count: usize) -> Vec<PathBuf> {
    let mut scored = apptainer_def_paths(workspace)
        .into_iter()
        .map(|path| {
            let score = sha256_hex(format!("{seed}:{}", path.display()).as_bytes());
            (score, path)
        })
        .collect::<Vec<_>>();
    scored.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));
    let take = count.min(scored.len());
    scored
        .into_iter()
        .take(take)
        .map(|(_, path)| path)
        .collect()
}

pub(super) fn write_frontend_repro_summary(
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

pub(super) fn write_frontend_security_summary(
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

pub(super) fn write_ensure_images_plan_report(workspace: &Workspace) -> Result<()> {
    let images_toml = workspace.path("configs/ci/tools/images.toml");
    let lock_sha_file = workspace.path("configs/ci/registry/tool_registry_lock.sha256");
    let hpc_naming_toml = workspace.path("configs/ci/tools/hpc_image_naming.toml");
    let out_dir = workspace.path("artifacts/containers/ensure-images");
    let report = out_dir.join("report.json");
    if !images_toml.is_file() || !lock_sha_file.is_file() || !hpc_naming_toml.is_file() {
        return Err(anyhow!(
            "ensure-images plan requires configs/ci/tools/images.toml, configs/ci/registry/tool_registry_lock.sha256, and configs/ci/tools/hpc_image_naming.toml"
        ));
    }
    bijux_dna_infra::ensure_dir(&out_dir)
        .with_context(|| format!("create {}", out_dir.display()))?;
    let images_sha = sha256_file_hex(&images_toml)?;
    let lock_sha = std::fs::read_to_string(&lock_sha_file)
        .with_context(|| format!("read {}", lock_sha_file.display()))?
        .trim()
        .to_string();
    let combined_sha = {
        use sha2::{Digest, Sha256};
        format!(
            "{:x}",
            Sha256::digest(format!("{images_sha}\n{lock_sha}\n").as_bytes())
        )
    };
    let images: toml::Value = toml::from_str(&std::fs::read_to_string(&images_toml)?)?;
    let naming: toml::Value = toml::from_str(&std::fs::read_to_string(&hpc_naming_toml)?)?;
    let prefix = naming
        .get("registry_prefix")
        .and_then(toml::Value::as_str)
        .unwrap_or_default()
        .trim_end_matches('/')
        .to_string();
    let tag_format = naming
        .get("tag_format")
        .and_then(toml::Value::as_str)
        .unwrap_or_default()
        .to_string();
    let tool_re = Regex::new(
        naming
            .get("tool_regex")
            .and_then(toml::Value::as_str)
            .unwrap_or_default(),
    )?;
    let version_re = Regex::new(
        naming
            .get("version_regex")
            .and_then(toml::Value::as_str)
            .unwrap_or_default(),
    )?;
    let hpc_refs = images
        .as_table()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|(tool, meta)| {
            let version = meta
                .get("version")
                .and_then(toml::Value::as_str)
                .unwrap_or_default()
                .trim()
                .to_string();
            (!version.is_empty() && tool_re.is_match(&tool) && version_re.is_match(&version)).then(
                || {
                    let tag = tag_format
                        .replace("{tool}", &tool)
                        .replace("{version}", &version);
                    serde_json::json!({
                        "tool": tool,
                        "version": version,
                        "hpc_image_ref": format!("{prefix}/{tool}:{tag}"),
                    })
                },
            )
        })
        .collect::<Vec<_>>();
    write_utf8(
        &report,
        &format!(
            "{}\n",
            serde_json::to_string_pretty(&serde_json::json!({
                "schema_version": "bijux.containers.ensure_images.v3",
                "action": "plan",
                "reason": "native-control-plane",
                "images_toml": "configs/ci/tools/images.toml",
                "hpc_naming_toml": "configs/ci/tools/hpc_image_naming.toml",
                "tool_registry_lock": "configs/ci/registry/tool_registry_lock.sha256",
                "images_sha": images_sha,
                "lock_sha": lock_sha,
                "combined_sha": combined_sha,
                "selected_tools": [],
                "hpc_image_refs": hpc_refs,
            }))?
        ),
    )?;
    Ok(())
}

pub(super) fn write_vuln_hook_report(
    workspace: &Workspace,
    sbom_root: &Path,
    out: &Path,
    toolkit: &str,
    promoted_only: bool,
) -> Result<()> {
    let scanner = if command_exists("grype") {
        Some("grype")
    } else if command_exists("trivy") {
        Some("trivy")
    } else {
        None
    };
    let mut allowed_tools = BTreeSet::new();
    if promoted_only {
        for (tool, row) in lock_items_by_tool(workspace)? {
            if row.get("status").and_then(serde_json::Value::as_str) == Some("production") {
                allowed_tools.insert(tool);
            }
        }
    }
    if !toolkit.trim().is_empty() {
        let bundle_tools = resolve_toolkit_tools(workspace, toolkit)?
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
            .collect::<BTreeSet<_>>();
        if allowed_tools.is_empty() {
            allowed_tools = bundle_tools;
        } else {
            allowed_tools = allowed_tools
                .intersection(&bundle_tools)
                .cloned()
                .collect::<BTreeSet<_>>();
        }
    }
    let per_tool_dir = workspace.path("artifacts/containers/vuln");
    bijux_dna_infra::ensure_dir(&per_tool_dir)
        .with_context(|| format!("create {}", per_tool_dir.display()))?;
    let mut rows = Vec::new();
    for entry in WalkDir::new(sbom_root)
        .into_iter()
        .filter_map(std::result::Result::ok)
    {
        if !entry.file_type().is_file()
            || entry.path().extension().and_then(|ext| ext.to_str()) != Some("txt")
            || !entry
                .path()
                .file_name()
                .and_then(|value| value.to_str())
                .is_some_and(|name| name.ends_with(".packages.txt"))
        {
            continue;
        }
        let tool = entry
            .path()
            .parent()
            .and_then(|parent| parent.file_name())
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_string();
        if !allowed_tools.is_empty() && !allowed_tools.contains(&tool) {
            continue;
        }
        let mut row = serde_json::json!({
            "sbom": entry.path().display().to_string(),
            "scanner": scanner.unwrap_or("none"),
            "status": "not_scanned",
            "summary": "",
            "tool": tool,
        });
        if let Some(scanner) = scanner {
            let output = if scanner == "grype" {
                std::process::Command::new("grype")
                    .args([
                        format!("sbom:{}", entry.path().display()),
                        "-o".to_string(),
                        "json".to_string(),
                    ])
                    .current_dir(&workspace.root)
                    .output()
            } else {
                std::process::Command::new("trivy")
                    .args([
                        "sbom".to_string(),
                        "--format".to_string(),
                        "json".to_string(),
                        entry.path().display().to_string(),
                    ])
                    .current_dir(&workspace.root)
                    .output()
            }
            .with_context(|| format!("run {scanner} for {}", entry.path().display()))?;
            let summary = if output.stdout.is_empty() {
                String::from_utf8_lossy(&output.stderr)
                    .chars()
                    .take(500)
                    .collect::<String>()
            } else {
                String::from_utf8_lossy(&output.stdout)
                    .chars()
                    .take(2000)
                    .collect::<String>()
            };
            row["status"] = serde_json::Value::String(if output.status.success() {
                "ok".to_string()
            } else {
                "error".to_string()
            });
            row["summary"] = serde_json::Value::String(summary);
        }
        let tool_name = row
            .get("tool")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown");
        write_utf8(
            &per_tool_dir.join(format!("{tool_name}.json")),
            &format!("{}\n", serde_json::to_string_pretty(&row)?),
        )?;
        rows.push(row);
    }
    write_utf8(
        out,
        &format!(
            "{}\n",
            serde_json::to_string_pretty(&serde_json::json!({
                "schema_version": "bijux.container.vuln_hook.v1",
                "scanner": scanner.unwrap_or("none"),
                "toolkit": if toolkit.trim().is_empty() { "all" } else { toolkit },
                "promoted_only": promoted_only,
                "items": rows,
            }))?
        ),
    )?;
    Ok(())
}

pub(super) fn command_exists(program: &str) -> bool {
    std::process::Command::new(program)
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

pub(super) fn sha256_file_hex(path: &Path) -> Result<String> {
    Ok(sha256_hex(
        &std::fs::read(path).with_context(|| format!("read {}", path.display()))?,
    ))
}

pub(super) fn merge_outcomes(
    mut left: ContainerCommandOutcome,
    right: ContainerCommandOutcome,
) -> ContainerCommandOutcome {
    left.exit_code = if left.exit_code != 0 {
        left.exit_code
    } else {
        right.exit_code
    };
    left.stdout.push_str(&right.stdout);
    left.stderr.push_str(&right.stderr);
    left
}
