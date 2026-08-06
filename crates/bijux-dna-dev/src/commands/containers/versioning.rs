#![allow(clippy::too_many_lines)]

use super::{
    anyhow, append_toml_table, container_support_summary, container_version_deprecations_path,
    env_or_empty, failure_lines, governed_container_file_ids, load_toml, out_path_arg, parse_date,
    read_utf8, run_argv, success_line, table_string, tool_versions, write_utf8, BTreeMap, BTreeSet,
    ContainerCommandOutcome, Context, Local, PathBuf, Result, Utc, Workspace,
};

pub(super) fn extract_version_map_content(workspace: &Workspace) -> Result<String> {
    bijux_dna_containers::api::render_container_version_map(&workspace.root)
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
    bijux_dna_containers::api::render_container_version_index(&workspace.root)
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
            bijux_dna_containers::api::CONTAINER_VERSION_LOCK_GENERATOR_PATH,
        ])
        .output()
        .with_context(|| "inspect lock discipline diff".to_string())?;
    let changed = String::from_utf8_lossy(&diff.stdout);
    let has_versions =
        changed.lines().any(|line| line.trim() == "containers/versions/versions.toml");
    let has_lock = changed.lines().any(|line| line.trim() == "containers/versions/lock.json");
    let has_generator = changed.lines().any(|line| {
        line.trim() == bijux_dna_containers::api::CONTAINER_VERSION_LOCK_GENERATOR_PATH
    });
    if (has_versions || has_generator) && !has_lock {
        return Ok(ContainerCommandOutcome::failure(
            "lock change discipline: version source or generator changed but lock.json did not\n",
        ));
    }
    if !has_versions && !has_generator && has_lock {
        return Ok(ContainerCommandOutcome::failure(
            "lock change discipline: lock.json changed without version source or generator change\n",
        ));
    }
    success_line("lock change discipline: OK")
}

pub(super) fn check_lock_schema(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    bijux_dna_containers::api::load_container_version_lock(&workspace.root)?;
    success_line("lock schema: OK")
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
    let support = container_support_summary(workspace)?;
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
        let unsupported =
            support.get(&tool).is_some_and(|(disposition, _)| disposition == "unsupported");
        if source_sha.is_empty() && pin.is_empty() && !unsupported {
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
            } else if pin.chars().all(|ch| ch == '0') {
                errors.push(format!("{tool}: pinned_commit must not be an all-zero placeholder"));
            } else if !matches!(pin.len(), 7 | 40) {
                errors.push(format!("{tool}: pinned_commit must be short(7) or full(40) git hash"));
            } else if !pin.chars().all(|ch| ch.is_ascii_hexdigit()) {
                errors.push(format!("{tool}: pinned_commit must contain only hexadecimal digits"));
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
        let previous_version = table_string(&previous_row, "version");
        let current_version = table_string(current_row, "version");
        if previous_version == current_version && !current_version.is_empty() {
            for field in ["source", "source_sha256", "pinned_commit"] {
                let previous_value = table_string(&previous_row, field);
                let current_value = table_string(current_row, field);
                if previous_value != current_value {
                    errors.push(format!(
                        "{tool}: {field} changed without a version change ({previous_value} -> {current_value})"
                    ));
                }
            }
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
    let lock_tools = bijux_dna_containers::api::load_container_version_lock(&workspace.root)?
        .items
        .into_iter()
        .map(|item| item.tool)
        .collect::<BTreeSet<_>>();
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

pub(super) fn generate_version_lock_content(workspace: &Workspace) -> Result<String> {
    bijux_dna_containers::api::render_container_version_lock(&workspace.root)
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

fn is_noncanonical_version_authority_path(rel: &str) -> bool {
    if !rel.starts_with("containers/") || rel.starts_with("containers/docs/") {
        return false;
    }
    if rel.starts_with("containers/runs/") && rel.ends_with("/evidence.lock.json") {
        return false;
    }
    if ["requirements.lock", "package-lock.json", "pnpm-lock.yaml", "poetry.lock", "uv.lock"]
        .iter()
        .any(|name| rel.ends_with(name))
    {
        return false;
    }
    !matches!(
        rel,
        "containers/contracts/catalog.lock.json"
            | "containers/contracts/schemas/catalog-lock.v1.json"
            | "containers/contracts/schemas/version-lock.v4.json"
            | "containers/contracts/schemas/version-map.v2.json"
            | "containers/versions/versions.toml"
            | "containers/versions/lock.json"
            | "containers/versions/LOCK.md"
            | "containers/versions/index.md"
    )
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
        .filter(|rel| is_noncanonical_version_authority_path(rel))
        .collect::<Vec<_>>();
    if !forbidden.is_empty() {
        let mut stderr =
            String::from("non-canonical version/lock files found under containers/ (use containers/versions/* only):\n");
        stderr.push_str(&forbidden.join("\n"));
        stderr.push('\n');
        return Ok(ContainerCommandOutcome::failure(stderr));
    }

    let mut errors = Vec::new();
    if let Err(error) = bijux_dna_containers::api::validate_container_version_lock(&workspace.root)
    {
        errors.push(format!("- version lock: {error:#}"));
    }

    let version_source_markers = [
        "VERSION_SOURCE: containers/contracts/catalog.toml",
        "VERSION_SOURCE: containers/versions/versions.toml",
    ];
    for root in [workspace.path("containers/apptainer"), workspace.path("containers/docker")] {
        for entry in walkdir::WalkDir::new(&root)
            .into_iter()
            .filter_map(std::result::Result::ok)
            .filter(|entry| entry.file_type().is_file())
        {
            let ext = entry.path().extension().and_then(|ext| ext.to_str());
            let file_name =
                entry.path().file_name().and_then(|name| name.to_str()).unwrap_or_default();
            if ext != Some("def") && !file_name.starts_with("Dockerfile.arm64.") {
                continue;
            }
            let raw = read_utf8(entry.path()).unwrap_or_default();
            if !version_source_markers.iter().any(|marker| raw.contains(marker)) {
                errors.push(format!(
                    "- version authority: missing canonical VERSION_SOURCE marker in {}",
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

pub(super) fn regenerate_version_outputs(workspace: &Workspace) -> Result<()> {
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
                "failed to regenerate version output with `{}`: {}",
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
        "# schema_version = 1\n# owner = bijux-genomics\n\n",
    )?;
    regenerate_version_outputs(workspace)?;
    success_line(format!("deprecated {tool}@{version} (compatibility_mode={compatibility_mode})"))
}

#[cfg(test)]
mod tests {
    use super::is_noncanonical_version_authority_path;

    #[test]
    fn distinguishes_run_evidence_locks_from_version_authorities() {
        assert!(!is_noncanonical_version_authority_path(
            "containers/runs/run-1134-pydamage/evidence.lock.json"
        ));
        assert!(is_noncanonical_version_authority_path(
            "containers/contracts/alternate-version-lock.json"
        ));
    }
}
