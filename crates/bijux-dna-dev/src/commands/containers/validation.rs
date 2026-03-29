use super::*;
use super::runtime::*;

pub(super) fn check_tool_name_collision(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let images = images_metadata(workspace)?;
    let versions = tool_versions(workspace)?;
    let tool_ids = tool_status_manifest(workspace)?
        .into_keys()
        .collect::<BTreeSet<_>>();
    let docker_ids = docker_tool_ids(workspace)?;
    let apptainer_ids = apptainer_tool_ids(workspace);
    let domain_ids = walkdir::WalkDir::new(workspace.path("domain"))
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter_map(|entry| {
            let path = entry.path();
            let parent = path.parent()?.file_name()?.to_str()?;
            if parent != "tools" || path.extension()?.to_str()? != "yaml" {
                return None;
            }
            let stem = path.file_stem()?.to_str()?;
            (stem != "_schema").then(|| stem.to_string())
        })
        .collect::<BTreeSet<_>>();
    let mut tools = BTreeMap::new();
    let mut bin_to_tool = BTreeMap::new();
    let mut errors = Vec::new();
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
        let expected_bin = table_string(&row, "expected_bin");
        tools.insert(
            tool_id.clone(),
            (expected_bin.clone(), table_string(&row, "status")),
        );
        if !expected_bin.is_empty() {
            if let Some(previous) = bin_to_tool.insert(expected_bin.clone(), tool_id.clone()) {
                if previous != tool_id {
                    errors.push(format!(
                        "expected_bin collision: '{expected_bin}' used by both '{previous}' and '{tool_id}'"
                    ));
                }
            }
        }
    }
    let numeric_suffix_re = Regex::new(r"^([a-z_]+?)(\d+)$").expect("regex");
    for tool_id in tools.keys() {
        let Some(captures) = numeric_suffix_re.captures(tool_id) else {
            continue;
        };
        let base = captures
            .get(1)
            .map(|value| value.as_str())
            .unwrap_or_default();
        if !tools.contains_key(base) {
            continue;
        }
        for candidate in [base.to_string(), tool_id.clone()] {
            if !images.contains_key(&candidate) {
                errors.push(format!(
                    "name-collision: missing images entry for '{candidate}'"
                ));
            }
            if !versions.contains_key(&candidate) {
                errors.push(format!(
                    "name-collision: missing versions entry for '{candidate}'"
                ));
            }
        }
        let base_bin = tools
            .get(base)
            .map(|(bin, _)| bin.clone())
            .unwrap_or_default();
        let suffixed_bin = tools
            .get(tool_id)
            .map(|(bin, _)| bin.clone())
            .unwrap_or_default();
        if !base_bin.is_empty() && base_bin == suffixed_bin {
            errors.push(format!(
                "name-collision: expected_bin must differ for '{base}' and '{tool_id}' (both '{base_bin}')"
            ));
        }
    }
    let surfaces = [
        ("registry", tools.keys().cloned().collect::<BTreeSet<_>>()),
        (
            "images",
            images
                .iter()
                .filter(|&(_key, value)| value.is_table())
                .map(|(key, _value)| key.clone())
                .collect::<BTreeSet<_>>(),
        ),
        (
            "versions",
            versions.keys().cloned().collect::<BTreeSet<_>>(),
        ),
        ("tool_ids", tool_ids),
        ("docker", docker_ids),
        ("apptainer", apptainer_ids),
        ("domain_tools", domain_ids),
    ];
    let all_ids = surfaces
        .iter()
        .flat_map(|(_, ids)| ids.iter().cloned())
        .collect::<BTreeSet<_>>();
    let norm_re = Regex::new(r"^[a-z][a-z0-9_]*$").expect("regex");
    for tool_id in &all_ids {
        if !norm_re.is_match(tool_id) {
            errors.push(format!("id normalization: '{tool_id}' is not snake_case"));
        }
    }
    for tool_id in &all_ids {
        let present = surfaces
            .iter()
            .filter_map(|(name, ids)| ids.contains(tool_id).then_some(*name))
            .collect::<Vec<_>>();
        if !present.contains(&"registry")
            && present.iter().any(|name| {
                matches!(
                    *name,
                    "images" | "versions" | "tool_ids" | "docker" | "apptainer"
                )
            })
        {
            errors.push(format!(
                "id parity: '{tool_id}' present in {present:?} but missing from registry"
            ));
        }
    }
    let name_map = workspace.path("containers/docs/TOOL_NAME_MAP.md");
    if name_map.exists() {
        let text = read_utf8(&name_map)?;
        for tool_id in tools.keys() {
            if !text.contains(&format!("`{tool_id}`")) {
                errors.push(format!("tool-name-map missing tool id '{tool_id}'"));
            }
        }
    } else {
        errors.push("missing containers/docs/TOOL_NAME_MAP.md".to_string());
    }
    if errors.is_empty() {
        return success_line("tool-name-collision: OK");
    }
    failure_lines("tool-name-collision: failed", &errors)
}

pub(super) fn check_tool_container_coverage(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let images = images_metadata(workspace)?;
    let docker_ids = docker_tool_ids(workspace)?;
    let apptainer_ids = apptainer_tool_ids(workspace);
    let parity_exemptions = images
        .get("parity_exemptions")
        .and_then(toml::Value::as_table)
        .into_iter()
        .flat_map(|table| {
            table.iter().filter_map(|(tool_id, enabled)| {
                enabled
                    .as_bool()
                    .filter(|enabled| *enabled)
                    .map(|_| tool_id.clone())
            })
        })
        .chain(
            images
                .get("apptainer_parity_exemptions")
                .and_then(toml::Value::as_table)
                .into_iter()
                .flat_map(|table| {
                    table.iter().filter_map(|(tool_id, enabled)| {
                        enabled
                            .as_bool()
                            .filter(|enabled| *enabled)
                            .map(|_| tool_id.clone())
                    })
                }),
        )
        .collect::<BTreeSet<_>>();
    let mut errors = Vec::new();
    for row in registry_tool_rows(workspace)? {
        let status = table_string(&row, "status");
        if status != "production" || !table_bool(&row, "container") {
            continue;
        }
        let tool_id = {
            let id = table_string(&row, "id");
            if id.is_empty() {
                table_string(&row, "tool_id")
            } else {
                id
            }
        };
        let runtimes = table_array_strings(&row, "runtimes")
            .into_iter()
            .collect::<BTreeSet<_>>();
        let dockerfile = table_string(&row, "dockerfile");
        let apptainer_def = table_string(&row, "apptainer_def");
        if runtimes.contains("docker") && dockerfile.is_empty() {
            errors.push(format!(
                "{tool_id}: runtime includes docker but dockerfile is unset"
            ));
        }
        if runtimes.contains("apptainer") && apptainer_def.is_empty() {
            errors.push(format!(
                "{tool_id}: runtime includes apptainer but apptainer_def is unset"
            ));
        }
        if dockerfile.is_empty() && apptainer_def.is_empty() {
            errors.push(format!(
                "{tool_id}: supported container tool has no container paths"
            ));
        }
        if !dockerfile.is_empty() {
            let docker_path = workspace.path(&dockerfile);
            if !docker_path.exists() {
                errors.push(format!("{tool_id} dockerfile missing: {dockerfile}"));
            }
            let expected = format!("Dockerfile.{tool_id}");
            if docker_path.file_name().and_then(|name| name.to_str()) != Some(expected.as_str()) {
                errors.push(format!(
                    "{tool_id} dockerfile naming mismatch: expected {expected}"
                ));
            }
        }
        if !apptainer_def.is_empty() {
            let apptainer_path = workspace.path(&apptainer_def);
            if !apptainer_path.exists() {
                errors.push(format!("{tool_id} apptainer def missing: {apptainer_def}"));
            }
            let expected = format!("{tool_id}.def");
            if apptainer_path.file_name().and_then(|name| name.to_str()) != Some(expected.as_str())
            {
                errors.push(format!(
                    "{tool_id} apptainer naming mismatch: expected {expected}"
                ));
            }
        }
        if !dockerfile.is_empty()
            && apptainer_def.is_empty()
            && !parity_exemptions.contains(&tool_id)
        {
            errors.push(format!(
                "{tool_id} has dockerfile but no apptainer_def and is not exempt (set configs/ci/tools/images.toml [parity_exemptions].{tool_id} = true)"
            ));
        }
        if !dockerfile.is_empty() && !docker_ids.contains(&tool_id) {
            errors.push(format!(
                "{tool_id}: docker coverage missing concrete Dockerfile"
            ));
        }
        if !apptainer_def.is_empty() && !apptainer_ids.contains(&tool_id) {
            errors.push(format!(
                "{tool_id}: apptainer coverage missing concrete definition"
            ));
        }
    }
    if errors.is_empty() {
        return success_line("tool/container coverage: OK");
    }
    failure_lines("tool/container coverage check failed:", &errors)
}

pub(super) fn check_toolkit_bundles(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let bundles = toolkit_bundles(workspace)?;
    if bundles.is_empty() {
        return Ok(ContainerCommandOutcome::failure(
            "toolkit bundles: no [bundles.*] entries found\n",
        ));
    }
    let images = images_metadata(workspace)?;
    let docker_ids = docker_tool_ids(workspace)?;
    let apptainer_ids = apptainer_tool_ids(workspace);
    let mut registry = BTreeMap::new();
    for row in registry_tool_rows(workspace)? {
        let tool = {
            let id = table_string(&row, "id");
            if id.is_empty() {
                table_string(&row, "tool_id")
            } else {
                id
            }
        };
        if !tool.is_empty() {
            registry.insert(tool, row);
        }
    }
    let mut errors = Vec::new();
    for (bundle_id, spec) in bundles {
        let tools = table_array_strings(&spec, "tools");
        if tools.is_empty() {
            errors.push(format!("{bundle_id}: tools must be a non-empty array"));
            continue;
        }
        for tool in tools {
            let Some(registry_row) = registry.get(&tool) else {
                errors.push(format!("{bundle_id}: tool '{tool}' missing from registry"));
                continue;
            };
            let Some(image_meta) = images.get(&tool).and_then(toml::Value::as_table) else {
                errors.push(format!(
                    "{bundle_id}: tool '{tool}' missing images.toml metadata"
                ));
                continue;
            };
            if table_string(image_meta, "version").is_empty() {
                errors.push(format!(
                    "{bundle_id}: tool '{tool}' images.toml entry missing version"
                ));
            }
            let status = table_string(registry_row, "status");
            if !matches!(status.as_str(), "production" | "experimental" | "planned") {
                errors.push(format!(
                    "{bundle_id}: tool '{tool}' has unsupported status '{status}'"
                ));
                continue;
            }
            if status == "planned" {
                if image_meta.get("enabled").and_then(toml::Value::as_bool) != Some(false) {
                    errors.push(format!(
                        "{bundle_id}: planned tool '{tool}' must be enabled=false in images.toml"
                    ));
                }
                continue;
            }
            let mut policy = table_string(image_meta, "shipping_policy");
            let has_apptainer = apptainer_ids.contains(&tool);
            let has_docker = docker_ids.contains(&tool);
            if policy.is_empty() {
                policy = if has_apptainer && has_docker {
                    "docker_apptainer".to_string()
                } else if has_apptainer {
                    "apptainer_only".to_string()
                } else if has_docker {
                    "docker_only".to_string()
                } else {
                    "none".to_string()
                };
            }
            match policy.as_str() {
                "apptainer_only" if !has_apptainer => {
                    errors.push(format!(
                        "{bundle_id}: production tool '{tool}' requires apptainer container"
                    ));
                }
                "docker_only" if !has_docker => {
                    errors.push(format!(
                        "{bundle_id}: production tool '{tool}' requires docker container"
                    ));
                }
                "docker_apptainer" if !(has_apptainer && has_docker) => {
                    errors.push(format!(
                        "{bundle_id}: production tool '{tool}' requires both docker and apptainer containers"
                    ));
                }
                "none" if !(has_apptainer || has_docker) => {
                    errors.push(format!(
                        "{bundle_id}: production tool '{tool}' has no container definition"
                    ));
                }
                _ => {}
            }
        }
    }
    if errors.is_empty() {
        return success_line("toolkit bundle completeness: OK");
    }
    failure_lines("toolkit bundle completeness check failed:", &errors)
}

pub(super) fn check_hpc_image_naming(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    let usage = "Usage: cargo run -p bijux-dna-dev -- containers run check-hpc-image-naming";
    if matches!(args, [single] if single == "--help" || single == "-h") {
        return success_line(usage);
    }
    if !args.is_empty() {
        return Err(anyhow!(usage.to_string()));
    }
    write_ensure_images_plan_report(workspace)?;
    let cfg = workspace.path("configs/ci/tools/hpc_image_naming.toml");
    let report = workspace.path("artifacts/containers/ensure-images/report.json");
    if !cfg.exists() {
        return Ok(ContainerCommandOutcome::failure(
            "hpc image naming: missing config\n",
        ));
    }
    if !report.exists() {
        return Ok(ContainerCommandOutcome::failure(
            "hpc image naming: missing ensure-images report\n",
        ));
    }
    let conf = load_toml(&cfg)?;
    let rep = read_json(&report)?;
    let prefix = conf
        .get("registry_prefix")
        .and_then(toml::Value::as_str)
        .unwrap_or_default()
        .trim_end_matches('/')
        .to_string();
    let tool_re = Regex::new(
        conf.get("tool_regex")
            .and_then(toml::Value::as_str)
            .unwrap_or_default(),
    )
    .context("invalid tool_regex in hpc_image_naming.toml")?;
    let version_re = Regex::new(
        conf.get("version_regex")
            .and_then(toml::Value::as_str)
            .unwrap_or_default(),
    )
    .context("invalid version_regex in hpc_image_naming.toml")?;
    let tag_format = conf
        .get("tag_format")
        .and_then(toml::Value::as_str)
        .unwrap_or_default()
        .to_string();
    let rows = rep
        .get("hpc_image_refs")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut errors = Vec::new();
    for row in &rows {
        let tool = row
            .get("tool")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_string();
        let version = row
            .get("version")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_string();
        let image_ref = row
            .get("hpc_image_ref")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_string();
        if !tool_re.is_match(&tool) {
            errors.push(format!("{tool}: tool id does not match tool_regex"));
        }
        if !version_re.is_match(&version) {
            errors.push(format!(
                "{tool}: version '{version}' does not match version_regex"
            ));
        }
        let expected_tag = tag_format
            .replace("{tool}", &tool)
            .replace("{version}", &version);
        let expected_ref = format!("{prefix}/{tool}:{expected_tag}");
        if image_ref != expected_ref {
            errors.push(format!(
                "{tool}: hpc_image_ref mismatch, expected {expected_ref}, got {image_ref}"
            ));
        }
    }
    if errors.is_empty() {
        return success_line(format!("hpc image naming: OK ({})", rows.len()));
    }
    failure_lines("hpc image naming: FAILED", &errors)
}

pub(super) fn check_planned_actionability(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let planned = workspace.path("containers/docs/PLANNED.md");
    if !planned.exists() {
        return Ok(ContainerCommandOutcome::failure(
            "planned actionability: missing containers/docs/PLANNED.md\n",
        ));
    }
    let text = read_utf8(&planned)?;
    let mut errors = Vec::new();
    for header in ["| Tool |", "Owner"] {
        if !text.contains(header) {
            errors.push(format!(
                "PLANNED.md missing required column/header marker: {header}"
            ));
        }
    }
    let mut rows = Vec::new();
    let mut in_table = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("| Tool ") && trimmed.contains("Owner") {
            in_table = true;
            continue;
        }
        if in_table && trimmed.starts_with("|---") {
            continue;
        }
        if in_table && trimmed.starts_with('|') {
            rows.push(trimmed.to_string());
        } else if in_table && trimmed.is_empty() {
            break;
        }
    }
    if rows.is_empty() {
        errors.push("PLANNED.md has no actionable planned tool rows".to_string());
    }
    for row in rows {
        let cols = row
            .trim_matches('|')
            .split('|')
            .map(str::trim)
            .collect::<Vec<_>>();
        if cols.len() < 5 {
            errors.push(format!("PLANNED.md malformed row: {row}"));
            continue;
        }
        let tool = cols[0];
        let owner = cols[4];
        if matches!(owner, "" | "-" | "`-`" | "`\"") {
            errors.push(format!("{tool}: missing owner"));
        }
    }
    if errors.is_empty() {
        return success_line(format!(
            "planned actionability: OK ({})",
            text.lines()
                .filter(|line| line.trim().starts_with('|'))
                .count()
                .saturating_sub(2)
        ));
    }
    failure_lines("planned actionability: FAILED", &errors)
}

pub(super) fn check_bijux_template_markers(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let template = workspace.path("containers/apptainer/shared/TEMPLATE.def.inc");
    let mut errors = Vec::new();
    if !template.exists() {
        errors
            .push("missing template file containers/apptainer/shared/TEMPLATE.def.inc".to_string());
    }
    for path in fs::read_dir(workspace.path("containers/apptainer/shared"))
        .with_context(|| {
            format!(
                "read {}",
                workspace.path("containers/apptainer/shared").display()
            )
        })?
        .filter_map(std::result::Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("def"))
    {
        let head = read_utf8(&path)?
            .lines()
            .take(20)
            .collect::<Vec<_>>()
            .join("\n");
        if !head.contains("BIJUX_TEMPLATE: v1") {
            errors.push(format!(
                "{}: missing BIJUX_TEMPLATE: v1 marker",
                workspace.rel(&path).display()
            ));
        }
    }
    if errors.is_empty() {
        return success_line("bijux-template-markers: OK");
    }
    failure_lines("bijux-template-markers: failed", &errors)
}

pub(super) fn check_tool_id_contract(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let manifest = workspace.path("containers/TOOL_IDS.txt");
    if !manifest.is_file() {
        return Ok(ContainerCommandOutcome::failure(format!(
            "missing {}\n",
            manifest.display()
        )));
    }
    let lines = read_utf8(&manifest)?
        .lines()
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    let required_headers = [
        "# GENERATED FILE - DO NOT EDIT",
        "# Regenerate with: cargo run -p bijux-dna-dev -- containers run generate-tool-ids",
        "# format: <tool_id><TAB><status>",
    ];
    let allowed_status = ["production", "experimental", "planned"]
        .into_iter()
        .collect::<BTreeSet<_>>();
    let tool_re = Regex::new(r"^[a-z][a-z0-9_]*$").expect("regex");
    let docker_ids = docker_tool_ids(workspace)?;
    let apptainer_ids = apptainer_tool_ids(workspace);
    let mut seen = BTreeSet::new();
    let mut status_by_id = BTreeMap::new();
    let mut errors = Vec::new();
    for (index, header) in required_headers.iter().enumerate() {
        if lines.get(index).map(std::string::String::as_str) != Some(*header) {
            errors.push(format!(
                "header line {} mismatch: expected '{}'",
                index + 1,
                header
            ));
        }
    }
    for (index, raw) in lines.iter().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts = raw.split('\t').collect::<Vec<_>>();
        if parts.len() != 2 {
            errors.push(format!(
                "line {}: expected exactly 2 TAB-separated fields",
                index + 1
            ));
            continue;
        }
        let tool_id = parts[0].trim().to_string();
        let status = parts[1].trim().to_string();
        if !tool_re.is_match(&tool_id) {
            errors.push(format!("line {}: invalid tool_id '{tool_id}'", index + 1));
        }
        if !allowed_status.contains(status.as_str()) {
            errors.push(format!("line {}: invalid status '{status}'", index + 1));
        }
        if !seen.insert(tool_id.clone()) {
            errors.push(format!("line {}: duplicate tool_id '{tool_id}'", index + 1));
        }
        status_by_id.insert(tool_id, status);
    }
    for (tool_id, status) in status_by_id {
        let ap_count = usize::from(apptainer_ids.contains(&tool_id));
        let docker_count = usize::from(docker_ids.contains(&tool_id));
        if matches!(status.as_str(), "production" | "experimental") {
            if ap_count != 1 {
                errors.push(format!(
                    "tool '{tool_id}' ({status}) must map to exactly one apptainer def (found {ap_count})"
                ));
            }
            if docker_count != 1 {
                errors.push(format!(
                    "tool '{tool_id}' ({status}) must map to exactly one dockerfile (found {docker_count})"
                ));
            }
        } else {
            if ap_count > 1 {
                errors.push(format!(
                    "tool '{tool_id}' ({status}) has ambiguous apptainer defs (found {ap_count})"
                ));
            }
            if docker_count > 1 {
                errors.push(format!(
                    "tool '{tool_id}' ({status}) has ambiguous dockerfiles (found {docker_count})"
                ));
            }
        }
    }
    if errors.is_empty() {
        return success_line("tool id contract: OK");
    }
    failure_lines("tool id contract check failed:", &errors)
}

pub(super) fn check_docker_arch_policy(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let amd64_dir = workspace.path("containers/docker/amd64");
    let policy_doc = workspace.path("containers/docker/multiarch-policy.md");
    if !policy_doc.is_file() {
        return Ok(ContainerCommandOutcome::failure(
            "docker arch policy: missing containers/docker/multiarch-policy.md\n",
        ));
    }
    let text = read_utf8(&policy_doc)?;
    let mut errors = Vec::new();
    if !text.contains("arm64") {
        errors.push("policy doc must mention arm64 support contract".to_string());
    }
    for marker in ["build strategy", "publish strategy", "promotion criteria"] {
        if !text.to_ascii_lowercase().contains(marker) {
            errors.push(format!(
                "policy doc missing required multiarch marker: {marker}"
            ));
        }
    }
    for marker in ["cross-build", "buildx", "naming convention", "amd64"] {
        if !text.to_ascii_lowercase().contains(marker) {
            errors.push(format!(
                "policy doc missing required amd64-plan marker: {marker}"
            ));
        }
    }
    if amd64_dir.is_dir()
        && fs::read_dir(&amd64_dir)
            .with_context(|| format!("read {}", amd64_dir.display()))?
            .filter_map(std::result::Result::ok)
            .any(|entry| {
                entry
                    .path()
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.starts_with("Dockerfile."))
            })
    {
        errors.push(
            "amd64 Dockerfiles detected under containers/docker/amd64\nThis repo currently ships docker/arm64 definitions only by contract."
                .to_string(),
        );
    }
    if errors.is_empty() {
        return success_line("docker arch policy: OK (arm64-only)");
    }
    failure_lines("docker arch policy: failed", &errors)
}

pub(super) fn check_docker_arm64_completeness(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let docker = docker_tool_ids(workspace)?;
    let mut required = BTreeSet::new();
    for row in registry_tool_rows(workspace)? {
        let tool = {
            let id = table_string(&row, "id");
            if id.is_empty() {
                table_string(&row, "tool_id")
            } else {
                id
            }
        };
        let runtimes = table_array_strings(&row, "runtimes");
        if !tool.is_empty() && runtimes.iter().any(|runtime| runtime == "docker") {
            required.insert(tool);
        }
    }
    let waiver_path = workspace.path("containers/docker/arm64/WAIVERS.toml");
    let mut waived = BTreeSet::new();
    if waiver_path.exists() {
        let data = load_toml(&waiver_path)?;
        for row in data
            .get("waiver")
            .and_then(toml::Value::as_array)
            .cloned()
            .unwrap_or_default()
        {
            let Some(row) = row.as_table() else {
                continue;
            };
            let tool = table_string(row, "tool_id");
            let reason = table_string(row, "reason");
            let owner = table_string(row, "owner");
            let expires = table_string(row, "expires_on");
            if tool.is_empty() {
                return Ok(ContainerCommandOutcome::failure(
                    "docker arm64 completeness: waiver missing tool_id\n",
                ));
            }
            if reason.is_empty() || owner.is_empty() || expires.is_empty() {
                return Ok(ContainerCommandOutcome::failure(format!(
                    "docker arm64 completeness: waiver for {tool} missing reason/owner/expires_on\n"
                )));
            }
            waived.insert(tool);
        }
    }
    let missing = required
        .difference(&docker)
        .filter(|tool| !waived.contains(*tool))
        .cloned()
        .collect::<Vec<_>>();
    if missing.is_empty() {
        return success_line("docker arm64 completeness: OK");
    }
    failure_lines(
        "docker arm64 completeness: missing dockerfile for docker runtime registry tools:",
        &missing,
    )
}

pub(super) fn check_docker_context(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let mut errors = Vec::new();
    let scan_roots = [
        workspace.path("makes"),
        workspace.path("crates/bijux-dna-dev/src"),
    ];
    let broad_build_re = Regex::new(r"\bdocker\s+build\b.*\s\.\s*$").expect("regex");
    let host_copy_re = Regex::new(r"\b(COPY|ADD)\s+(\.\./|/Users/|~/)").expect("regex");
    for root in scan_roots {
        if !root.exists() {
            continue;
        }
        for entry in walkdir::WalkDir::new(&root)
            .into_iter()
            .filter_map(std::result::Result::ok)
            .filter(|entry| entry.file_type().is_file())
        {
            let path = entry.path();
            let ext = path
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or_default();
            if ext != "sh" && ext != "mk" {
                continue;
            }
            for (index, line) in read_utf8(path)?.lines().enumerate() {
                let trimmed = line.trim();
                if !trimmed.contains("docker build") {
                    continue;
                }
                if broad_build_re.is_match(trimmed)
                    || trimmed.ends_with("docker build")
                    || trimmed.ends_with("docker build .")
                {
                    errors.push(format!(
                        "{}:{}: docker build must not use repo-root context '.'",
                        workspace.rel(path).display(),
                        index + 1
                    ));
                }
                if trimmed.contains("-f containers/docker/")
                    && !trimmed.contains(" containers/docker/")
                {
                    errors.push(format!(
                        "{}:{}: docker build should use containers/docker/<arch> as context",
                        workspace.rel(path).display(),
                        index + 1
                    ));
                }
            }
        }
    }
    let dockerignore = workspace.path("containers/docker/arm64/.dockerignore");
    if dockerignore.exists() {
        let dockerignore_text = read_utf8(&dockerignore)?;
        for pattern in [
            ".git",
            "artifacts",
            "assets",
            "**/*.pem",
            "**/*.key",
            ".env",
        ] {
            if !dockerignore_text.contains(pattern) {
                errors.push(format!(
                    "containers/docker/arm64/.dockerignore: missing pattern '{pattern}'"
                ));
            }
        }
    } else {
        errors.push(
            "containers/docker/arm64/.dockerignore: missing (required for context minimization)"
                .to_string(),
        );
    }
    for path in dockerfile_paths(workspace)? {
        for (index, line) in read_utf8(&path)?.lines().enumerate() {
            let trimmed = line.trim();
            if Regex::new(r"^(COPY|ADD)\s+\.\s")
                .expect("regex")
                .is_match(trimmed)
            {
                errors.push(format!(
                    "{}:{}: forbidden broad context copy ('COPY . ...' or 'ADD . ...')",
                    workspace.rel(&path).display(),
                    index + 1
                ));
            }
            if host_copy_re.is_match(trimmed) {
                errors.push(format!(
                    "{}:{}: forbidden host/workspace path copy in Dockerfile",
                    workspace.rel(&path).display(),
                    index + 1
                ));
            }
        }
    }
    if errors.is_empty() {
        return success_line("docker context policy: OK");
    }
    failure_lines("docker context check failed:", &errors)
}

pub(super) fn check_docker_hardening(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let exceptions_doc = workspace.path("containers/docker/NONROOT_EXCEPTIONS.md");
    let entrypoint_doc = workspace.path("containers/docker/ENTRYPOINT_EXCEPTIONS.md");
    if !exceptions_doc.exists() {
        return Ok(ContainerCommandOutcome::failure(
            "missing containers/docker/NONROOT_EXCEPTIONS.md\n",
        ));
    }
    if !entrypoint_doc.exists() {
        return Ok(ContainerCommandOutcome::failure(
            "missing containers/docker/ENTRYPOINT_EXCEPTIONS.md\n",
        ));
    }
    let row_re = Regex::new(r"\|\s*`([^`]+)`\s*\|").expect("regex");
    let allowed = row_re
        .captures_iter(&read_utf8(&exceptions_doc)?)
        .filter_map(|captures| captures.get(1).map(|value| value.as_str().to_string()))
        .collect::<BTreeSet<_>>();
    let entrypoint_allowed = row_re
        .captures_iter(&read_utf8(&entrypoint_doc)?)
        .filter_map(|captures| captures.get(1).map(|value| value.as_str().to_string()))
        .collect::<BTreeSet<_>>();
    let required_labels = canonical_container_label_keys();
    let entrypoint_re = Regex::new(r"^ENTRYPOINT\s+\[").expect("regex");
    let cmd_re = Regex::new(r"^CMD\s+\[").expect("regex");
    let cmd_line_re = Regex::new(r"^CMD\s+\[(.+)\]\s*$").expect("regex");
    let user_re = Regex::new(r"^USER\s+(.+)$").expect("regex");
    let healthcheck_re = Regex::new(r"^HEALTHCHECK\s+(.+)$").expect("regex");
    let sh_entrypoint_re = Regex::new(r#"^ENTRYPOINT\s+\["/bin/sh",\s*"-c""#).expect("regex");
    let mut errors = Vec::new();
    for path in dockerfile_paths(workspace)? {
        let tool_id = path
            .file_name()
            .and_then(|name| name.to_str())
            .and_then(|name| name.strip_prefix("Dockerfile."))
            .unwrap_or_default()
            .to_string();
        let text = read_utf8(&path)?;
        let rel = workspace.rel(&path).display().to_string();
        for label in required_labels {
            if !text.contains(label) {
                errors.push(format!("{rel}: missing label {label}"));
            }
        }
        let pipe_shell_re =
            Regex::new(r"curl\s+[^|\n]*\|\s*(bash|sh)\b|wget\s+[^|\n]*\|\s*(bash|sh)\b")
                .expect("regex");
        if pipe_shell_re.is_match(&text) {
            errors.push(format!("{rel}: forbidden curl|bash or wget|sh pattern"));
        }
        let first_from = text
            .lines()
            .find(|line| line.trim().starts_with("FROM "))
            .unwrap_or_default()
            .trim()
            .to_string();
        if !first_from.contains("@sha256:") {
            errors.push(format!("{rel}: FROM must be digest-pinned"));
        }
        let has_entrypoint = text.lines().any(|line| entrypoint_re.is_match(line.trim()));
        let has_cmd = text.lines().any(|line| cmd_re.is_match(line.trim()));
        let entrypoint_exempt =
            entrypoint_allowed.contains(&tool_id) || entrypoint_allowed.contains("*");
        if !has_cmd && !entrypoint_exempt {
            errors.push(format!("{rel}: missing JSON-form CMD"));
        } else if has_cmd {
            let cmd_text = text
                .lines()
                .find_map(|line| cmd_line_re.captures(line.trim()))
                .and_then(|captures| {
                    captures
                        .get(1)
                        .map(|value| value.as_str().to_ascii_lowercase())
                })
                .unwrap_or_default();
            if !entrypoint_exempt
                && !["--help", "-h", "--version"]
                    .iter()
                    .any(|needle| cmd_text.contains(needle))
            {
                errors.push(format!("{rel}: CMD should default to --help/-h/--version"));
            }
        }
        if has_entrypoint && !entrypoint_exempt {
            errors.push(format!(
                "{rel}: ENTRYPOINT is forbidden unless listed in ENTRYPOINT_EXCEPTIONS.md"
            ));
        }
        if sh_entrypoint_re.is_match(
            text.lines()
                .find(|line| line.trim().starts_with("ENTRYPOINT"))
                .unwrap_or_default(),
        ) && !entrypoint_exempt
        {
            errors.push(format!("{rel}: ENTRYPOINT must not use /bin/sh -c wrapper"));
        }
        let nonroot = text
            .lines()
            .filter_map(|line| user_re.captures(line.trim()))
            .filter_map(|captures| {
                captures
                    .get(1)
                    .map(|value| value.as_str().trim().to_string())
            })
            .any(|user| user != "root" && user != "0");
        if !nonroot && !allowed.contains(&tool_id) && !allowed.contains("*") {
            errors.push(format!(
                "{rel}: no non-root USER and not listed in NONROOT_EXCEPTIONS.md"
            ));
        }
        if text.contains("HEALTHCHECK") {
            let line = text
                .lines()
                .find_map(|line| healthcheck_re.captures(line.trim()))
                .and_then(|captures| captures.get(1).map(|value| value.as_str().to_string()))
                .unwrap_or_default();
            if !line.contains("--interval=") || !line.contains("--timeout=") {
                errors.push(format!(
                    "{rel}: HEALTHCHECK must define --interval and --timeout"
                ));
            }
            if !text.contains("--version") && !text.to_ascii_lowercase().contains("healthcheck") {
                errors.push(format!(
                    "{rel}: HEALTHCHECK should verify tool --version or explicit health check"
                ));
            }
        }
    }
    if errors.is_empty() {
        return success_line("docker hardening: OK");
    }
    failure_lines("docker hardening: failed", &errors)
}

pub(super) fn check_docker_labels(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let required = [
        "org.opencontainers.image.title",
        "org.opencontainers.image.version",
        "org.opencontainers.image.source",
        "org.opencontainers.image.licenses",
    ];
    let tool_re =
        Regex::new(r#"org\.opencontainers\.image\.tool="?([A-Za-z0-9_.-]+)"?"#).expect("regex");
    let version_re =
        Regex::new(r#"org\.opencontainers\.image\.version="?([A-Za-z0-9_.:-]+)"?"#).expect("regex");
    let apptainer_version_re =
        Regex::new(r"org\.opencontainers\.image\.version\s+([^\s]+)").expect("regex");
    let mut docker_versions = BTreeMap::new();
    let mut errors = Vec::new();
    for path in dockerfile_paths(workspace)? {
        let text = read_utf8(&path)?;
        let rel = workspace.rel(&path).display().to_string();
        let missing = required
            .iter()
            .filter(|label| !text.contains(**label))
            .copied()
            .collect::<Vec<_>>();
        if !missing.is_empty() {
            errors.push(format!("{rel} missing labels: {}", missing.join(", ")));
        }
        let tool_id = path
            .file_name()
            .and_then(|name| name.to_str())
            .and_then(|name| name.strip_prefix("Dockerfile."))
            .unwrap_or_default()
            .to_string();
        if let Some(captures) = tool_re.captures(&text) {
            let label = captures
                .get(1)
                .map(|value| value.as_str())
                .unwrap_or_default();
            if label != tool_id {
                errors.push(format!("{rel} tool label mismatch: {label} != {tool_id}"));
            }
        }
        if let Some(captures) = version_re.captures(&text) {
            docker_versions.insert(
                tool_id,
                captures
                    .get(1)
                    .map(|value| value.as_str().to_string())
                    .unwrap_or_default(),
            );
        }
        if text.contains("/opt/bijux/VERSION.json") || text.contains("bijux-tool-info") {
            errors.push(format!(
                "{rel}: duplicate in-image self-report metadata is forbidden; publish metadata must flow through OCI labels"
            ));
        }
    }
    for path in apptainer_def_paths(workspace) {
        let tool_id = path
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_string();
        let Some(docker_version) = docker_versions.get(&tool_id) else {
            continue;
        };
        let text = read_utf8(&path)?;
        let Some(captures) = apptainer_version_re.captures(&text) else {
            continue;
        };
        let apptainer_version = captures
            .get(1)
            .map(|value| value.as_str().trim().trim_matches('"').to_string())
            .unwrap_or_default();
        if docker_version != &apptainer_version {
            errors.push(format!(
                "version parity mismatch for {tool_id}: docker={docker_version} apptainer={apptainer_version}"
            ));
        }
    }
    if errors.is_empty() {
        return success_line("docker label policy: OK");
    }
    failure_lines("docker label policy check failed:", &errors)
}

pub(super) fn check_docker_unpinned_apt(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let ci_mode = matches!(
        env_or_empty("CI").trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes"
    );
    let mut errors = Vec::new();
    for dockerfile in dockerfile_paths(workspace)? {
        let rel = workspace.rel(&dockerfile).display().to_string();
        for line in read_utf8(&dockerfile)?.lines() {
            if !line.contains("apt-get install") && !line.contains("apt install") {
                continue;
            }
            let mut segment = if let Some((_, tail)) = line.split_once("apt-get install") {
                tail.to_string()
            } else if let Some((_, tail)) = line.split_once("apt install") {
                tail.to_string()
            } else {
                continue;
            };
            let option_re = Regex::new(r"--[a-zA-Z0-9-]+(?:=[^\s]+)?").expect("regex");
            segment = option_re.replace_all(&segment, " ").into_owned();
            segment = segment.replace("&&", " ").replace('\\', " ");
            for token in segment.split_whitespace().filter(|token| !token.is_empty()) {
                if matches!(
                    token,
                    "install"
                        | "apt-get"
                        | "apt"
                        | "update"
                        | "rm"
                        | "-rf"
                        | "/var/lib/apt/lists/*"
                        | ";"
                        | "|"
                ) {
                    continue;
                }
                if token.starts_with('$') || token.starts_with('"') || token.starts_with('/') {
                    continue;
                }
                if !token.contains('=') {
                    errors.push(format!("{rel}: unpinned apt package '{token}'"));
                }
            }
        }
    }
    if errors.is_empty() {
        return success_line("docker apt pin check: OK");
    }
    if ci_mode {
        return failure_lines("docker apt pin check: failed", &errors);
    }
    Ok(ContainerCommandOutcome::success(format!(
        "docker apt pin check: WARN (non-CI mode)\n{}\n",
        errors
            .into_iter()
            .map(|error| format!("- {error}"))
            .collect::<Vec<_>>()
            .join("\n")
    )))
}

pub(super) fn check_docker_version_sync(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let versions = tool_versions(workspace)?;
    let arg_re = Regex::new(r"^ARG\s+TOOL_VERSION\s*=\s*([^\s#]+)\s*$").expect("regex");
    let mut errors = Vec::new();
    for dockerfile in dockerfile_paths(workspace)? {
        let tool = dockerfile
            .file_name()
            .and_then(|name| name.to_str())
            .and_then(|name| name.strip_prefix("Dockerfile."))
            .unwrap_or_default()
            .to_string();
        let text = read_utf8(&dockerfile)?;
        let Some(docker_version) = text
            .lines()
            .find_map(|line| arg_re.captures(line.trim()))
            .and_then(|captures| {
                captures.get(1).map(|value| {
                    value
                        .as_str()
                        .trim()
                        .trim_matches('"')
                        .trim_matches('\'')
                        .to_string()
                })
            })
        else {
            errors.push(format!(
                "{}: missing ARG TOOL_VERSION=<version>",
                workspace.rel(&dockerfile).display()
            ));
            continue;
        };
        let Some(registry_row) = versions.get(&tool) else {
            errors.push(format!(
                "{}: tool '{tool}' missing in versions.toml",
                workspace.rel(&dockerfile).display()
            ));
            continue;
        };
        let registry_version = table_string(registry_row, "version");
        let placeholder = matches!(
            docker_version.as_str(),
            "unknown" | "planned" | "latest-pinned"
        ) || docker_version.ends_with("-planned");
        if !placeholder && docker_version != registry_version {
            errors.push(format!(
                "{}: TOOL_VERSION '{docker_version}' != versions.toml '{registry_version}'",
                workspace.rel(&dockerfile).display()
            ));
        }
        if !text.contains(r#"org.opencontainers.image.version="${TOOL_VERSION}""#) {
            errors.push(format!(
                "{}: image version label must reference TOOL_VERSION build arg",
                workspace.rel(&dockerfile).display()
            ));
        }
    }
    if errors.is_empty() {
        return success_line("docker version sync: OK");
    }
    failure_lines("docker version sync: failed", &errors)
}

pub(super) fn check_dockerfiles_built(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    if env_or_empty("CI").is_empty() {
        return success_line("dockerfiles built check: SKIP (CI-only gate)");
    }
    let summary_path = workspace.path("artifacts/containers/summary.json");
    if !summary_path.exists() {
        return Ok(ContainerCommandOutcome::failure(
            "dockerfiles built check: missing artifacts/containers/summary.json\n",
        ));
    }
    let summary = read_json(&summary_path)?;
    let expected_tools = dockerfile_paths(workspace)?
        .into_iter()
        .filter_map(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .and_then(|name| name.strip_prefix("Dockerfile."))
                .map(ToOwned::to_owned)
        })
        .collect::<Vec<_>>();
    let rows = summary
        .get("items")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|row| {
            row.get("runtime").and_then(serde_json::Value::as_str) == Some("docker-arm64")
        })
        .filter_map(|row| {
            let tool = row
                .get("tool")
                .and_then(serde_json::Value::as_str)
                .map(|tool| tool.trim().to_string())?;
            Some((tool, row))
        })
        .collect::<BTreeMap<_, _>>();
    let mut errors = Vec::new();
    for tool in expected_tools {
        let Some(row) = rows.get(&tool) else {
            errors.push(format!("{tool}: missing docker-arm64 summary row"));
            continue;
        };
        if row.get("status").and_then(serde_json::Value::as_str) != Some("ok") {
            errors.push(format!("{tool}: build/smoke status is not ok"));
            continue;
        }
        let manifest_path = PathBuf::from(
            row.get("manifest")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default(),
        );
        if !manifest_path.exists() {
            errors.push(format!(
                "{tool}: manifest missing at {}",
                manifest_path.display()
            ));
            continue;
        }
        let manifest = read_json(&manifest_path)?;
        let digest = manifest
            .get("resolved_image_digest")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_string();
        if digest.is_empty() {
            errors.push(format!("{tool}: missing resolved_image_digest in manifest"));
        }
    }
    if errors.is_empty() {
        return success_line("dockerfiles built check: OK");
    }
    failure_lines("dockerfiles built check: failed", &errors)
}

pub(super) fn check_no_secrets(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let mut scan = Vec::new();
    scan.extend(apptainer_def_paths(workspace));
    scan.extend(dockerfile_paths(workspace)?);
    let patterns = [
        Regex::new(r"AKIA[0-9A-Z]{16}").expect("regex"),
        Regex::new(r#"(?i)(secret|token|password)\s*[:=]\s*['"]?[A-Za-z0-9_\-]{8,}"#)
            .expect("regex"),
        Regex::new(r"ghp_[A-Za-z0-9]{20,}").expect("regex"),
        Regex::new(r"github_pat_[A-Za-z0-9_]{20,}").expect("regex"),
        Regex::new(r"xox[baprs]-[A-Za-z0-9-]{10,}").expect("regex"),
        Regex::new(r"AIza[0-9A-Za-z\-_]{35}").expect("regex"),
        Regex::new(r#"(?i)aws_secret_access_key\s*[:=]\s*['"]?[A-Za-z0-9/+=]{30,}"#)
            .expect("regex"),
        Regex::new(r"(?i)-----BEGIN (?:RSA|OPENSSH|EC) PRIVATE KEY-----").expect("regex"),
    ];
    let mut errors = Vec::new();
    for path in scan {
        for (index, line) in read_utf8(&path)?.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if patterns.iter().any(|pattern| pattern.is_match(line)) {
                errors.push(format!(
                    "{}:{}: potential secret pattern matched",
                    workspace.rel(&path).display(),
                    index + 1
                ));
            }
        }
    }
    if errors.is_empty() {
        return success_line("container secret scan: OK");
    }
    failure_lines("container secret scan: FAILED", &errors)
}

pub(super) fn check_runtime_downloads(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let mut runtime_allowed = BTreeMap::new();
    let network_dir = workspace.path("containers/network");
    if network_dir.exists() {
        for entry in fs::read_dir(&network_dir)
            .with_context(|| format!("read {}", network_dir.display()))?
            .filter_map(std::result::Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("toml"))
        {
            let value = load_toml(&entry)?;
            let tool_id = value
                .get("tool_id")
                .and_then(toml::Value::as_str)
                .unwrap_or_else(|| {
                    entry
                        .file_stem()
                        .and_then(|name| name.to_str())
                        .unwrap_or_default()
                })
                .trim()
                .to_string();
            runtime_allowed.insert(
                tool_id,
                value
                    .get("runtime_network")
                    .and_then(toml::Value::as_bool)
                    .unwrap_or(false),
            );
        }
    }
    let download_re = Regex::new(r"\b(curl|wget)\b").expect("regex");
    let mut errors = Vec::new();
    for path in apptainer_def_paths(workspace) {
        let text = read_utf8(&path)?;
        let tool = path
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_string();
        let mut chunks = Vec::new();
        if let Some(runscript) = text
            .split("%runscript")
            .nth(1)
            .and_then(|body| body.split("\n%").next())
        {
            chunks.push(runscript.to_string());
        }
        if let Some(environment) = text
            .split("%environment")
            .nth(1)
            .and_then(|body| body.split("\n%").next())
        {
            chunks.push(environment.to_string());
        }
        for chunk in chunks {
            if download_re.is_match(&chunk) && !runtime_allowed.get(&tool).copied().unwrap_or(false)
            {
                errors.push(format!(
                    "{}: runtime curl/wget forbidden unless runtime_network=true",
                    workspace.rel(&path).display()
                ));
            }
        }
    }
    for path in dockerfile_paths(workspace)? {
        let tool = path
            .file_name()
            .and_then(|name| name.to_str())
            .and_then(|name| name.strip_prefix("Dockerfile."))
            .unwrap_or_default()
            .to_string();
        for (index, line) in read_utf8(&path)?.lines().enumerate() {
            let trimmed = line.trim();
            if (trimmed.starts_with("ENTRYPOINT") || trimmed.starts_with("CMD"))
                && download_re.is_match(trimmed)
                && !runtime_allowed.get(&tool).copied().unwrap_or(false)
            {
                errors.push(format!(
                    "{}:{}: runtime curl/wget in CMD/ENTRYPOINT forbidden unless runtime_network=true",
                    workspace.rel(&path).display(),
                    index + 1
                ));
            }
        }
    }
    if errors.is_empty() {
        return success_line("runtime download policy: OK");
    }
    failure_lines("runtime download policy: FAILED", &errors)
}

pub(super) fn check_vuln_allowlist(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let path = std::env::var("ALLOWLIST").map_or_else(
        |_| workspace.path("configs/ci/tools/vuln_allowlist.toml"),
        PathBuf::from,
    );
    if !path.exists() {
        return Ok(ContainerCommandOutcome::failure(format!(
            "vuln allowlist: missing {}\n",
            path.display()
        )));
    }
    let data = load_toml(&path)?;
    let cve_re = Regex::new(r"^CVE-\d{4}-\d{4,}$").expect("regex");
    let now = Utc::now();
    let mut seen = BTreeSet::new();
    let mut errors = Vec::new();
    for (index, row) in data
        .get("allowlist")
        .and_then(toml::Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .enumerate()
    {
        let Some(row) = row.as_table() else {
            continue;
        };
        let cve = table_string(row, "cve").to_ascii_uppercase();
        let reason = table_string(row, "reason");
        let expires = table_string(row, "expires_utc");
        if cve.is_empty() || !cve_re.is_match(&cve) {
            errors.push(format!("allowlist[{index}] invalid cve: {cve:?}"));
            continue;
        }
        if !seen.insert(cve.clone()) {
            errors.push(format!("duplicate allowlisted cve: {cve}"));
        }
        if reason.len() < 12 {
            errors.push(format!("{cve}: reason/justification too short"));
        }
        if expires.is_empty() {
            errors.push(format!("{cve}: missing expires_utc"));
            continue;
        }
        let parsed = chrono::DateTime::parse_from_rfc3339(&expires.replace('Z', "+00:00"));
        let Ok(parsed) = parsed else {
            errors.push(format!("{cve}: invalid expires_utc format: {expires}"));
            continue;
        };
        if parsed <= now.fixed_offset() {
            errors.push(format!("{cve}: allowlist entry expired at {expires}"));
        }
    }
    if errors.is_empty() {
        return success_line(format!("vuln allowlist: OK ({})", seen.len()));
    }
    failure_lines("vuln allowlist: FAILED", &errors)
}

pub(super) fn check_vuln_hook(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let out = iso_root_path(workspace).join("containers/vuln_scan_report.json");
    let allowlist = check_vuln_allowlist(workspace)?;
    if !allowlist.is_success() {
        return Ok(allowlist);
    }
    write_vuln_hook_report(
        workspace,
        &workspace.path("artifacts/containers/sbom"),
        &out,
        &env_or_default("TOOLKIT", "fastq_core"),
        env_or_default("PROMOTED_ONLY", "1") == "1",
    )?;
    if !out.is_file() {
        return Ok(ContainerCommandOutcome::failure(format!(
            "vuln hook: missing report {}\n",
            out.display()
        )));
    }
    let payload = read_json(&out)?;
    let items = payload
        .get("items")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let rows = items
        .into_iter()
        .filter_map(|row| {
            let tool = row
                .get("tool")
                .and_then(serde_json::Value::as_str)
                .map(|tool| tool.trim().to_string())?;
            Some((tool, row))
        })
        .collect::<BTreeMap<_, _>>();
    let promoted = lock_items_by_tool(workspace)?
        .into_iter()
        .filter_map(|(tool, row)| {
            (row.get("status").and_then(serde_json::Value::as_str) == Some("production"))
                .then_some(tool)
        })
        .collect::<Vec<_>>();
    if rows.is_empty() && env_or_empty("CI").is_empty() {
        return success_line("vuln hook: SKIP (no local vuln scan items)");
    }
    let promoted_only = matches!(
        env_or_default("PROMOTED_ONLY", "1")
            .trim()
            .to_ascii_lowercase()
            .as_str(),
        "1" | "true" | "yes"
    );
    let mut errors = Vec::new();
    if promoted_only && !promoted.is_empty() {
        for tool in promoted {
            let Some(row) = rows.get(&tool) else {
                errors.push(format!("{tool}: missing vuln scan item for promoted tool"));
                continue;
            };
            let status = row
                .get("status")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default();
            if !matches!(status, "ok" | "not_scanned") {
                errors.push(format!("{tool}: vuln scan status is {status}"));
            }
            let per_tool = workspace.path(&format!("artifacts/containers/vuln/{tool}.json"));
            if !per_tool.exists() {
                errors.push(format!(
                    "{tool}: missing per-tool vuln summary {}",
                    per_tool.display()
                ));
            }
        }
    }
    if errors.is_empty() {
        return success_line("vuln hook: OK");
    }
    failure_lines("vuln hook: FAILED", &errors)
}

pub(super) fn check_sbom_artifacts(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let manifest_root = workspace.path("artifacts/containers");
    if !manifest_root.exists() {
        if !env_or_empty("CI").is_empty() {
            return Ok(ContainerCommandOutcome::failure(
                "sbom artifacts: missing artifacts/containers\n",
            ));
        }
        return success_line("sbom artifacts: SKIP (no artifacts/containers)");
    }
    let strict_promoted =
        !env_or_empty("CI").is_empty() || env_or_default("REQUIRE_PROMOTED_SBOM", "0") == "1";
    let promoted = lock_items_by_tool(workspace)?
        .into_iter()
        .filter_map(|(tool, row)| {
            (row.get("status").and_then(serde_json::Value::as_str) == Some("production"))
                .then_some(tool)
        })
        .collect::<BTreeSet<_>>();
    let mut manifests = BTreeMap::<String, Vec<(PathBuf, serde_json::Value)>>::new();
    for path in fs::read_dir(&manifest_root)
        .with_context(|| format!("read {}", manifest_root.display()))?
        .filter_map(std::result::Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("json"))
    {
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();
        if matches!(name, "summary.json" | "report.json") {
            continue;
        }
        let Ok(data) = read_json(&path) else {
            continue;
        };
        let tool = data
            .get("tool")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_string();
        if !tool.is_empty() {
            manifests.entry(tool).or_default().push((path, data));
        }
    }
    let tools_to_check = if strict_promoted {
        promoted.iter().cloned().collect::<Vec<_>>()
    } else {
        let shared = manifests
            .keys()
            .filter(|tool| promoted.contains(*tool))
            .cloned()
            .collect::<Vec<_>>();
        if shared.is_empty() {
            manifests.keys().cloned().collect::<Vec<_>>()
        } else {
            shared
        }
    };
    let mut seen = 0;
    let mut errors = Vec::new();
    for tool in tools_to_check {
        let rows = manifests.get(&tool).cloned().unwrap_or_default();
        if rows.is_empty() {
            errors.push(format!(
                "{tool}: missing smoke/build manifest under artifacts/containers/"
            ));
            continue;
        }
        let ok_rows = rows
            .into_iter()
            .filter(|(_, data)| {
                data.get("status").and_then(serde_json::Value::as_str) == Some("ok")
            })
            .collect::<Vec<_>>();
        if ok_rows.is_empty() {
            errors.push(format!(
                "{tool}: has manifests but no successful status=ok result"
            ));
            continue;
        }
        for (manifest_path, data) in ok_rows {
            seen += 1;
            let sbom = data
                .get("sbom_path")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .trim()
                .to_string();
            let smoke_log = data
                .get("smoke_log_path")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .trim()
                .to_string();
            let smoke_log_sha = data
                .get("smoke_log_checksum_path")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .trim()
                .to_string();
            if sbom.is_empty() {
                errors.push(format!("{}: missing sbom_path", manifest_path.display()));
                continue;
            }
            let sbom_path = PathBuf::from(&sbom);
            if !sbom_path.exists() {
                errors.push(format!(
                    "{}: sbom_path does not exist: {sbom}",
                    manifest_path.display()
                ));
            } else if sbom_path.metadata().map(|meta| meta.len()).unwrap_or(0) == 0 {
                errors.push(format!(
                    "{}: sbom_path is empty: {sbom}",
                    manifest_path.display()
                ));
            } else if !sbom_path
                .display()
                .to_string()
                .replace('\\', "/")
                .contains(&format!("/sbom/{tool}/"))
            {
                errors.push(format!(
                    "{}: sbom_path not in required layout /sbom/{tool}/: {sbom}",
                    manifest_path.display()
                ));
            }
            if smoke_log.is_empty() || !PathBuf::from(&smoke_log).exists() {
                errors.push(format!(
                    "{}: missing smoke_log_path or file not found: {smoke_log}",
                    manifest_path.display()
                ));
            }
            if smoke_log_sha.is_empty() || !PathBuf::from(&smoke_log_sha).exists() {
                errors.push(format!(
                    "{}: missing smoke_log_checksum_path or file not found: {smoke_log_sha}",
                    manifest_path.display()
                ));
            }
        }
    }
    if errors.is_empty() {
        return success_line(format!("sbom artifacts: OK ({seen} manifests)"));
    }
    failure_lines("sbom artifacts: FAILED", &errors)
}

pub(super) fn check_time_locale_determinism(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let mut errors = Vec::new();
    for path in apptainer_def_paths(workspace) {
        let text = read_utf8(&path)?;
        let env = text
            .split("%environment")
            .nth(1)
            .and_then(|body| body.split("\n%").next())
            .unwrap_or_default();
        if !env.contains("TZ=UTC") {
            errors.push(format!(
                "{}: %environment must set TZ=UTC",
                workspace.rel(&path).display()
            ));
        }
        if !env.contains("LC_ALL=C") {
            errors.push(format!(
                "{}: %environment must set LC_ALL=C",
                workspace.rel(&path).display()
            ));
        }
    }
    if errors.is_empty() {
        return success_line("time/locale determinism: OK");
    }
    failure_lines("time/locale determinism: FAILED", &errors)
}

pub(super) fn check_tool_invocation_normalization(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let mut errors = Vec::new();
    for row in registry_tool_rows(workspace)? {
        let runtimes = table_array_strings(&row, "runtimes");
        if !runtimes
            .iter()
            .any(|runtime| runtime == "apptainer" || runtime == "docker")
        {
            continue;
        }
        let tool = {
            let id = table_string(&row, "id");
            if id.is_empty() {
                table_string(&row, "tool_id")
            } else {
                id
            }
        };
        let expected_bin = table_string(&row, "expected_bin");
        if tool.is_empty() {
            continue;
        }
        if expected_bin.is_empty() {
            errors.push(format!("{tool}: missing expected_bin"));
            continue;
        }
        for field in ["smoke_version_cmd", "smoke_help_cmd"] {
            let command = table_string(&row, field);
            if command.is_empty() {
                errors.push(format!("{tool}: missing {field}"));
                continue;
            }
            let token = command.split_whitespace().next().unwrap_or_default();
            if token != expected_bin {
                errors.push(format!(
                    "{tool}: {field} must start with expected_bin '{expected_bin}', got '{token}'"
                ));
            }
        }
    }
    if errors.is_empty() {
        return success_line("tool invocation normalization: OK");
    }
    failure_lines("tool invocation normalization: FAILED", &errors)
}

pub(super) fn check_smoke_inputs_policy(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let policy = workspace.path("configs/ci/tools/smoke_inputs_policy.toml");
    if !policy.is_file() {
        return Ok(ContainerCommandOutcome::failure(format!(
            "smoke-inputs policy: missing {}\n",
            policy.display()
        )));
    }
    let data = load_toml(&policy)?;
    let entries = data
        .get("tool_inputs")
        .and_then(toml::Value::as_table)
        .cloned()
        .unwrap_or_default();
    let mut errors = Vec::new();
    for (tool, row) in entries.clone() {
        let Some(row) = row.as_table() else {
            errors.push(format!("{tool}: policy row must be table"));
            continue;
        };
        let rel = table_string(row, "path");
        if rel.is_empty() {
            errors.push(format!("{tool}: missing path"));
            continue;
        }
        let path = workspace.path(&rel);
        if !path.exists() {
            errors.push(format!("{tool}: missing input file {rel}"));
            continue;
        }
        if !path.is_file() {
            errors.push(format!("{tool}: input path is not a file {rel}"));
            continue;
        }
        if path.metadata().map(|meta| meta.len()).unwrap_or(0) == 0 {
            errors.push(format!("{tool}: input file is empty {rel}"));
        }
    }
    if errors.is_empty() {
        return success_line(format!("smoke-inputs policy: OK ({})", entries.len()));
    }
    failure_lines("smoke-inputs policy: FAILED", &errors)
}

pub(super) fn load_runtime_manifest_rows(
    path: &std::path::Path,
) -> Result<BTreeMap<String, serde_json::Value>> {
    let mut rows = BTreeMap::new();
    for entry in fs::read_dir(path)
        .with_context(|| format!("read {}", path.display()))?
        .filter_map(std::result::Result::ok)
    {
        let manifest_path = entry.path();
        if manifest_path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let name = manifest_path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default();
        if matches!(
            name,
            "summary.json"
                | "report.json"
                | "lock.json"
                | "security_summary.json"
                | "sbom_index.json"
        ) {
            continue;
        }
        let Ok(row) = read_json(&manifest_path) else {
            continue;
        };
        let tool = row
            .get("tool")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_string();
        if !tool.is_empty() {
            rows.insert(tool, row);
        }
    }
    Ok(rows)
}

pub(super) fn normalized_version_output(row: &serde_json::Value) -> String {
    row.get("normalized_version_output")
        .and_then(serde_json::Value::as_str)
        .or_else(|| {
            row.get("version_output")
                .and_then(serde_json::Value::as_str)
        })
        .unwrap_or_default()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase()
}

pub(super) fn registry_tool_id(row: &toml::map::Map<String, toml::Value>) -> String {
    let id = table_string(row, "id");
    if id.is_empty() {
        table_string(row, "tool_id")
    } else {
        id
    }
}

pub(super) fn check_cross_runtime_representative(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
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

pub(super) fn check_cross_runtime_representative_at_paths(
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

pub(super) fn check_cross_runtime_smoke(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
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

pub(super) fn check_cross_runtime_smoke_at_paths(
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

pub(super) fn check_smoke_failure_classification(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let manifests = workspace.path("artifacts/containers/manifests");
    if !manifests.exists() {
        return success_line("smoke failure classification: SKIP (no manifests)");
    }
    let allowed = BTreeSet::from([
        "build".to_string(),
        "runtime".to_string(),
        "smoke_mismatch".to_string(),
    ]);
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

pub(super) fn check_smoke_contract(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
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
        if let Some(table) = images
            .get("smoke_exemptions")
            .and_then(toml::Value::as_table)
        {
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
        for row in value
            .get("tools")
            .and_then(toml::Value::as_array)
            .cloned()
            .unwrap_or_default()
        {
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
            let help_exit = row
                .get("smoke_help_exit_code")
                .map_or(Some(0), toml::Value::as_integer);
            let minimal_exit = row
                .get("smoke_minimal_exit_code")
                .map_or(Some(0), toml::Value::as_integer);
            let negative_exit = row
                .get("smoke_negative_exit_code")
                .map_or(Some(2), toml::Value::as_integer);

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
                errors.push(format!(
                    "{rel}: {tool_id} missing expected_bin tool binary contract"
                ));
            }
            if minimal_cmd.is_empty() {
                errors.push(format!(
                    "{rel}: {tool_id} resolved smoke_minimal_cmd is empty"
                ));
            }
            if minimal_exit.is_none() {
                errors.push(format!(
                    "{rel}: {tool_id} smoke_minimal_exit_code must be integer"
                ));
            }
            if negative_cmd.is_empty() {
                errors.push(format!(
                    "{rel}: {tool_id} resolved smoke_negative_cmd is empty"
                ));
            }
            if negative_exit.is_none() {
                errors.push(format!(
                    "{rel}: {tool_id} smoke_negative_exit_code must be integer"
                ));
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

pub(super) fn check_smoke_contract_lock(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let lock_path = std::env::var("LOCK_PATH").map_or_else(
        |_| workspace.path("containers/versions/lock.json"),
        PathBuf::from,
    );
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
    for item in lock
        .get("items")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default()
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
            errors.push(format!(
                "{tool}: smoke_log_dir not in required layout: {log_dir}"
            ));
        }
    }

    if errors.is_empty() {
        return success_line(format!("smoke lock gate: OK ({total} tools)"));
    }
    failure_lines("smoke lock gate: FAILED", &errors)
}

pub(super) fn vcf_imputation_core_tools() -> [&'static str; 8] {
    [
        "glimpse", "impute5", "minimac4", "shapeit5", "beagle", "eagle", "bcftools", "plink2",
    ]
}

pub(super) fn load_summary_rows(path: &std::path::Path) -> Result<BTreeMap<String, serde_json::Value>> {
    let summary = read_json(path)?;
    Ok(summary
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
        .collect())
}

pub(super) fn normalized_parity_output(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '.' {
                ch.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub(super) fn check_vcf_imputation_toolchain(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let required =
        load_toml(&workspace.path("configs/ci/tools/required_tools_vcf_downstream.toml"))?;
    let registry =
        load_toml(&workspace.path("configs/ci/registry/tool_registry_vcf_downstream.toml"))?;
    let registry_vcf = load_toml(&workspace.path("configs/ci/registry/tool_registry_vcf.toml"))?;

    let required_set = required
        .get("required_tools")
        .and_then(toml::Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|value| value.as_str().map(|tool| tool.trim().to_string()))
        .collect::<BTreeSet<_>>();
    let registry_rows = registry
        .get("tools")
        .and_then(toml::Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|value| value.as_table().cloned())
        .collect::<Vec<_>>();
    let registry_vcf_rows = registry_vcf
        .get("tools")
        .and_then(toml::Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|value| value.as_table().cloned())
        .collect::<Vec<_>>();
    let registry_ids = registry_rows
        .iter()
        .map(registry_tool_id)
        .filter(|tool| !tool.is_empty())
        .collect::<BTreeSet<_>>();
    let rows = registry_rows
        .into_iter()
        .map(|row| (registry_tool_id(&row), row))
        .filter(|(tool, _)| !tool.is_empty())
        .collect::<BTreeMap<_, _>>();
    let rows_vcf = registry_vcf_rows
        .into_iter()
        .map(|row| (registry_tool_id(&row), row))
        .filter(|(tool, _)| !tool.is_empty())
        .collect::<BTreeMap<_, _>>();

    let mut errors = Vec::new();
    let missing_in_required = registry_ids
        .difference(&required_set)
        .cloned()
        .collect::<Vec<_>>();
    let missing_in_registry = required_set
        .difference(&registry_ids)
        .cloned()
        .collect::<Vec<_>>();
    if !missing_in_required.is_empty() {
        errors.push(format!(
            "required_tools_vcf_downstream missing registry ids: {missing_in_required:?}"
        ));
    }
    if !missing_in_registry.is_empty() {
        errors.push(format!(
            "required_tools_vcf_downstream has unknown ids: {missing_in_registry:?}"
        ));
    }

    for tool in vcf_imputation_core_tools() {
        let row = rows.get(tool).or_else(|| rows_vcf.get(tool));
        let Some(row) = row else {
            errors.push(format!("{tool}: missing in VCF registry surfaces"));
            continue;
        };
        if !table_bool(row, "container") {
            errors.push(format!(
                "{tool}: container=false in vcf downstream registry"
            ));
        }
        let runtimes = table_array_strings(row, "runtimes")
            .into_iter()
            .collect::<BTreeSet<_>>();
        if !runtimes.contains("docker") || !runtimes.contains("apptainer") {
            errors.push(format!(
                "{tool}: runtimes must include docker+apptainer, got {runtimes:?}"
            ));
        }
        for key in [
            "smoke_version_cmd",
            "smoke_help_cmd",
            "version_cmd",
            "help_cmd",
            "expected_bin",
        ] {
            if table_string(row, key).is_empty() {
                errors.push(format!("{tool}: missing {key}"));
            }
        }
        let dockerfile = table_string(row, "dockerfile");
        let apptainer_def = table_string(row, "apptainer_def");
        if dockerfile.is_empty() || !workspace.path(&dockerfile).exists() {
            errors.push(format!(
                "{tool}: dockerfile missing: {}",
                if dockerfile.is_empty() {
                    "<empty>"
                } else {
                    &dockerfile
                }
            ));
        }
        if apptainer_def.is_empty() || !workspace.path(&apptainer_def).exists() {
            errors.push(format!(
                "{tool}: apptainer_def missing: {}",
                if apptainer_def.is_empty() {
                    "<empty>"
                } else {
                    &apptainer_def
                }
            ));
        }
        let license_file = workspace.path(&format!("containers/licenses/{tool}.license.toml"));
        if !license_file.exists() {
            errors.push(format!(
                "{tool}: missing license metadata {}",
                workspace.rel(&license_file).display()
            ));
        }
        let tool_doc = workspace.path(&format!("containers/docs/tools/{tool}.md"));
        if !tool_doc.exists() {
            errors.push(format!(
                "{tool}: missing tool doc {}",
                workspace.rel(&tool_doc).display()
            ));
        }
    }

    if errors.is_empty() {
        return success_line(format!(
            "vcf imputation toolchain check: OK ({}) core tools",
            vcf_imputation_core_tools().len()
        ));
    }
    failure_lines("vcf imputation toolchain check: FAILED", &errors)
}

pub(super) fn check_imputation_runtime_constraints(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let doc_path = workspace.path("containers/docs/IMPUTATION_RUNTIME_CONSTRAINTS.md");
    if !doc_path.is_file() {
        return Ok(ContainerCommandOutcome::failure(format!(
            "missing {}\n",
            doc_path.display()
        )));
    }
    let doc = read_utf8(&doc_path)?;
    let mut errors = Vec::new();
    for tool in vcf_imputation_core_tools() {
        if !doc.contains(&format!("| `{tool}` |")) {
            errors.push(format!("missing constraints row for {tool}"));
        }
    }
    for column in ["cpu_threads_min", "ram_gb_min", "scratch_gb_min"] {
        if !doc.contains(column) {
            errors.push(format!("constraints column {column} is required"));
        }
    }
    if errors.is_empty() {
        return success_line("imputation runtime constraints: OK");
    }
    failure_lines("imputation runtime constraints: FAILED", &errors)
}

pub(super) fn check_imputation_network_policy(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let doc_path = workspace.path("containers/docs/IMPUTATION_NETWORK_POLICY.md");
    if !doc_path.is_file() {
        return Ok(ContainerCommandOutcome::failure(format!(
            "missing {}\n",
            doc_path.display()
        )));
    }
    let mut errors = Vec::new();
    for tool in vcf_imputation_core_tools() {
        let path = workspace.path(&format!("containers/network/{tool}.network.toml"));
        if !path.exists() {
            errors.push(format!(
                "missing network metadata: {}",
                workspace.rel(&path).display()
            ));
            continue;
        }
        let data = load_toml(&path)?;
        if data
            .get("runtime_network")
            .and_then(toml::Value::as_bool)
            .unwrap_or(true)
        {
            errors.push(format!("{tool}: runtime_network must be false"));
        }
    }
    if errors.is_empty() {
        return success_line("imputation network policy: OK");
    }
    failure_lines("imputation network policy: FAILED", &errors)
}

pub(super) fn check_imputation_hardening(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let nonroot_ex = read_utf8(&workspace.path("containers/docker/NONROOT_EXCEPTIONS.md"))?;
    let entrypoint_ex = read_utf8(&workspace.path("containers/docker/ENTRYPOINT_EXCEPTIONS.md"))?;
    let wildcard_nonroot = nonroot_ex.contains("`*`");
    let wildcard_entrypoint = entrypoint_ex.contains("`*`");
    let user_regex = Regex::new(r"(?m)^USER\s+").expect("regex");
    let entrypoint_regex = Regex::new(r"(?m)^ENTRYPOINT\s+\[").expect("regex");
    let cmd_regex = Regex::new(r"(?m)^CMD\s+\[").expect("regex");
    let mut errors = Vec::new();
    for tool in vcf_imputation_core_tools() {
        let dockerfile = workspace.path(&format!("containers/docker/arm64/Dockerfile.{tool}"));
        if !dockerfile.exists() {
            errors.push(format!("{tool}: missing dockerfile"));
            continue;
        }
        let text = read_utf8(&dockerfile)?;
        if !user_regex.is_match(&text)
            && !wildcard_nonroot
            && !nonroot_ex.contains(&format!("`{tool}`"))
        {
            errors.push(format!(
                "{tool}: runs as root and is not listed in NONROOT_EXCEPTIONS.md"
            ));
        }
        if (!entrypoint_regex.is_match(&text) || !cmd_regex.is_match(&text))
            && !wildcard_entrypoint
            && !entrypoint_ex.contains(&format!("`{tool}`"))
        {
            errors.push(format!(
                "{tool}: missing JSON ENTRYPOINT/CMD and not listed in ENTRYPOINT_EXCEPTIONS.md"
            ));
        }
    }
    if errors.is_empty() {
        return success_line("imputation hardening policy: OK");
    }
    failure_lines("imputation hardening policy: FAILED", &errors)
}

pub(super) fn check_imputation_release_smoke(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let docker_summary = PathBuf::from(env_or_default(
        "DOCKER_SUMMARY",
        &workspace
            .path("artifacts/containers/docker-arm64/summary.json")
            .display()
            .to_string(),
    ));
    let apptainer_summary = PathBuf::from(env_or_default(
        "APPTAINER_SUMMARY",
        &workspace
            .path("artifacts/containers/apptainer/summary.json")
            .display()
            .to_string(),
    ));
    if !docker_summary.is_file() || !apptainer_summary.is_file() {
        if !env_or_empty("CI").is_empty() {
            return Ok(ContainerCommandOutcome::failure(format!(
                "imputation release smoke: missing summary files docker='{}' apptainer='{}'\n",
                docker_summary.display(),
                apptainer_summary.display()
            )));
        }
        return success_line("imputation release smoke: SKIP (missing local summary files)");
    }

    let docker_rows = load_summary_rows(&docker_summary)?;
    let apptainer_rows = load_summary_rows(&apptainer_summary)?;
    let mut errors = Vec::new();
    for (runtime, rows) in [("docker", &docker_rows), ("apptainer", &apptainer_rows)] {
        for tool in vcf_imputation_core_tools() {
            let Some(row) = rows.get(tool) else {
                errors.push(format!("{runtime}:{tool}: missing summary row"));
                continue;
            };
            if row.get("status").and_then(serde_json::Value::as_str) != Some("ok") {
                errors.push(format!("{runtime}:{tool}: status is not ok"));
            }
            let paths = row
                .get("smoke_output_paths")
                .and_then(serde_json::Value::as_object)
                .cloned()
                .unwrap_or_default();
            for key in ["version", "help"] {
                let output_path = paths
                    .get(key)
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default()
                    .trim()
                    .to_string();
                if output_path.is_empty() {
                    errors.push(format!(
                        "{runtime}:{tool}: missing smoke_output_paths.{key}"
                    ));
                } else if !PathBuf::from(&output_path).exists() {
                    errors.push(format!(
                        "{runtime}:{tool}: missing output file {output_path}"
                    ));
                }
            }
            if row
                .get("version_output")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .trim()
                .is_empty()
            {
                errors.push(format!("{runtime}:{tool}: empty version_output"));
            }
            if row
                .get("resolved_image_digest")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .trim()
                .is_empty()
            {
                errors.push(format!("{runtime}:{tool}: missing resolved_image_digest"));
            }
        }
    }
    if errors.is_empty() {
        return success_line("imputation release smoke: OK");
    }
    failure_lines("imputation release smoke: FAILED", &errors)
}

pub(super) fn check_imputation_cross_runtime_parity(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let docker_summary = PathBuf::from(env_or_default(
        "DOCKER_SUMMARY",
        &workspace
            .path("artifacts/containers/docker-arm64/summary.json")
            .display()
            .to_string(),
    ));
    let apptainer_summary = PathBuf::from(env_or_default(
        "APPTAINER_SUMMARY",
        &workspace
            .path("artifacts/containers/apptainer/summary.json")
            .display()
            .to_string(),
    ));
    if !docker_summary.is_file() || !apptainer_summary.is_file() {
        if !env_or_empty("CI").is_empty() {
            return Ok(ContainerCommandOutcome::failure(format!(
                "imputation cross-runtime parity: missing summary files docker='{}' apptainer='{}'\n",
                docker_summary.display(),
                apptainer_summary.display()
            )));
        }
        return success_line("imputation cross-runtime parity: SKIP (missing local summary files)");
    }

    let docker_rows = load_summary_rows(&docker_summary)?;
    let apptainer_rows = load_summary_rows(&apptainer_summary)?;
    let mut errors = Vec::new();
    for tool in vcf_imputation_core_tools() {
        let Some(docker_row) = docker_rows.get(tool) else {
            errors.push(format!("{tool}: missing from one runtime summary"));
            continue;
        };
        let Some(apptainer_row) = apptainer_rows.get(tool) else {
            errors.push(format!("{tool}: missing from one runtime summary"));
            continue;
        };
        let docker_version = normalized_parity_output(
            docker_row
                .get("version_output")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default(),
        );
        let apptainer_version = normalized_parity_output(
            apptainer_row
                .get("version_output")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default(),
        );
        if docker_version.is_empty() || apptainer_version.is_empty() {
            errors.push(format!("{tool}: empty version output for parity check"));
            continue;
        }
        if !docker_version.contains(tool) || !apptainer_version.contains(tool) {
            errors.push(format!(
                "{tool}: version outputs do not contain expected tool token"
            ));
            continue;
        }
        let declared = docker_row
            .get("declared_version")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase();
        if !declared.is_empty()
            && !matches!(declared.as_str(), "unknown" | "planned" | "latest-pinned")
            && (!docker_version.contains(&declared) || !apptainer_version.contains(&declared))
        {
            errors.push(format!(
                "{tool}: declared_version `{declared}` not present in both runtime outputs"
            ));
        }
    }
    if errors.is_empty() {
        return success_line("imputation cross-runtime parity: OK");
    }
    failure_lines("imputation cross-runtime parity: FAILED", &errors)
}

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

pub(super) fn check_build_provenance(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
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

pub(super) fn check_digest_changes_on_version_change(
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

pub(super) fn check_digest_output_policy(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
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

pub(super) fn check_runtime_tool_digest_recording(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
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

pub(super) fn check_rebuild_repro(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
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

pub(super) fn check_apptainer_rebuild_repro(
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

pub(super) fn check_apptainer_bijux_header(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
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

pub(super) fn check_hpc_frontend_policy_enforcement(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
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

pub(super) fn check_image_size_regression(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
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
    for row in policy
        .get("acknowledgement")
        .and_then(toml::Value::as_array)
        .cloned()
        .unwrap_or_default()
    {
        let Some(row) = row.as_table() else {
            continue;
        };
        let tool = table_string(row, "tool_id");
        let from_version = table_string(row, "from_version");
        let to_version = table_string(row, "to_version");
        let limit = row
            .get("max_growth_percent")
            .and_then(toml::Value::as_float)
            .unwrap_or(default_limit);
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
            let tool = row
                .get("tool")
                .and_then(serde_json::Value::as_str)?
                .to_string();
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
            let tool = row
                .get("tool")
                .and_then(serde_json::Value::as_str)?
                .to_string();
            Some((tool, row))
        })
        .collect::<BTreeMap<_, _>>();
    let mut checked = 0usize;
    let mut errors = Vec::new();
    for (tool, current_row) in current_items {
        if current_row
            .get("status")
            .and_then(serde_json::Value::as_str)
            != Some("production")
        {
            continue;
        }
        let Some(previous_row) = previous_items.get(&tool) else {
            continue;
        };
        let old_size = previous_row
            .get("image_size_bytes")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(0);
        let new_size = current_row
            .get("image_size_bytes")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(0);
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

pub(super) fn check_lock_matches_built_output(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
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
    let lock_items = lock_data
        .get("items")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let lock_tools = lock_items
        .iter()
        .filter_map(|item| item.get("tool").and_then(serde_json::Value::as_str))
        .map(ToOwned::to_owned)
        .collect::<BTreeSet<_>>();
    let lock_rows = lock_items
        .into_iter()
        .filter_map(|item| {
            let tool = item
                .get("tool")
                .and_then(serde_json::Value::as_str)?
                .to_string();
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
        for row in data
            .get("tools")
            .and_then(toml::Value::as_array)
            .cloned()
            .unwrap_or_default()
        {
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
    for item in summary
        .get("items")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default()
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
        let manifest = item
            .get("manifest")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
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
            errors.push(format!(
                "{tool}: missing from containers/versions/lock.json"
            ));
        }
        let Some(docker_manifest) = docker_manifest_by_tool.get(&tool) else {
            if strict_missing {
                errors.push(format!(
                    "{tool}: missing docker-arm64 manifest in artifacts/containers/summary.json"
                ));
            }
            continue;
        };
        if docker_manifest
            .get("status")
            .and_then(serde_json::Value::as_str)
            != Some("ok")
        {
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
                errors.push(format!(
                    "{tool}: missing version_output for lock/version comparison"
                ));
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
            errors.push(format!(
                "{tool}: missing resolved_image_digest in docker manifest"
            ));
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

pub(super) fn check_release_checklist(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let checklist_path = workspace.path("containers/docs/RELEASE_CHECKLIST.md");
    if !checklist_path.exists() {
        return Ok(ContainerCommandOutcome::failure(
            "release checklist check: missing containers/docs/RELEASE_CHECKLIST.md\n",
        ));
    }
    let checklist = read_utf8(&checklist_path)?;
    let registry = crate::catalog::containers::container_registry(workspace)?;
    let command_regex =
        Regex::new(r"cargo run -p bijux-dna-dev -- containers run ([a-z0-9-]+)").expect("regex");
    let missing = command_regex
        .captures_iter(&checklist)
        .filter_map(|capture| capture.get(1).map(|value| value.as_str().to_string()))
        .filter(|command| !registry.iter().any(|row| row.id == *command))
        .collect::<Vec<_>>();
    if missing.is_empty() {
        return success_line("release checklist mapping: OK");
    }
    failure_lines(
        "release checklist check: missing native checklist commands:",
        &missing,
    )
}

pub(super) fn check_toolkit_bundle_buildable(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let bundles = load_toml(&workspace.path("configs/ci/tools/toolkit_bundles.toml"))?;
    let images = load_toml(&workspace.path("configs/ci/tools/images.toml"))?;
    let bundle_table = bundles
        .get("bundles")
        .and_then(toml::Value::as_table)
        .cloned()
        .unwrap_or_default();
    let image_table = images.as_table().cloned().unwrap_or_default();
    let apptainer = apptainer_def_paths(workspace)
        .into_iter()
        .filter_map(|path| {
            path.file_stem()
                .and_then(|value| value.to_str())
                .map(ToOwned::to_owned)
        })
        .collect::<BTreeSet<_>>();
    let docker = dockerfile_paths(workspace)?
        .into_iter()
        .filter_map(|path| {
            path.file_name()
                .and_then(|value| value.to_str())
                .and_then(|value| {
                    value
                        .split_once("Dockerfile.")
                        .map(|(_, tool)| tool.to_string())
                })
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

pub(super) fn check_vcf_downstream_bundle_coverage(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
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
                .then(|| {
                    path.file_stem()
                        .and_then(|value| value.to_str())
                        .map(ToOwned::to_owned)
                })
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
    let phasing_required = BTreeSet::from([
        "beagle".to_string(),
        "eagle".to_string(),
        "shapeit5".to_string(),
    ]);
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

pub(super) fn summary(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
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

pub(super) fn run_env_prep(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    ensure_no_args("env-prep", args)?;
    let container_type = checked_container_type()?;
    let tools = env_or_empty("TOOLS");
    let stage = env_or_empty("STAGE");
    require_tools_or_stage(&tools, &stage)?;
    let mut argv = bijux_command_prefix();
    argv.extend([
        "environment".to_string(),
        "prep".to_string(),
        container_type,
    ]);
    if stage.is_empty() {
        argv.push(tools);
    } else {
        argv.push("--stage".to_string());
        argv.push(stage);
    }
    run_argv(workspace, &argv)
}

pub(super) fn run_env_smoke(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    ensure_no_args("env-smoke", args)?;
    let container_type = checked_container_type()?;
    let tools = env_or_empty("TOOLS");
    let stage = env_or_empty("STAGE");
    require_tools_or_stage(&tools, &stage)?;
    let mut argv = bijux_command_prefix();
    argv.extend([
        "environment".to_string(),
        "smoke".to_string(),
        container_type,
    ]);
    if stage.is_empty() {
        argv.push(tools);
    } else {
        argv.push("--stage".to_string());
        argv.push(stage);
    }
    run_argv(workspace, &argv)
}

pub(super) fn run_container_smoke(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    ensure_no_args("container-smoke", args)?;
    let tools = env_or_empty("TOOLS");
    let stage = env_or_empty("STAGE");
    require_tools_or_stage(&tools, &stage)?;
    let prep = run_env_prep(workspace, &[])?;
    if !prep.is_success() {
        return Ok(prep);
    }
    let smoke = run_env_smoke(workspace, &[])?;
    Ok(merge_outcomes(prep, smoke))
}

pub(super) fn run_containers_smoke(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    ensure_no_args("containers-smoke", args)?;
    checked_container_type()?;
    let list = run_argv(
        workspace,
        &[
            bijux_command_prefix(),
            vec!["registry".to_string(), "list-stages".to_string()],
        ]
        .concat(),
    )?;
    if !list.is_success() {
        return Ok(list);
    }
    let mut aggregate = ContainerCommandOutcome::success(String::new());
    for stage in list
        .stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        let header = format!("== stage {stage}\n");
        aggregate.stdout.push_str(&header);
        let prep = run_argv(
            workspace,
            &[
                bijux_command_prefix(),
                vec![
                    "environment".to_string(),
                    "prep".to_string(),
                    checked_container_type()?,
                    "--stage".to_string(),
                    stage.to_string(),
                ],
            ]
            .concat(),
        )?;
        aggregate = merge_outcomes(aggregate, prep.clone());
        if !prep.is_success() {
            return Ok(aggregate);
        }
        let smoke = run_argv(
            workspace,
            &[
                bijux_command_prefix(),
                vec![
                    "environment".to_string(),
                    "smoke".to_string(),
                    checked_container_type()?,
                    "--stage".to_string(),
                    stage.to_string(),
                ],
            ]
            .concat(),
        )?;
        aggregate = merge_outcomes(aggregate, smoke.clone());
        if !smoke.is_success() {
            return Ok(aggregate);
        }
    }
    Ok(aggregate)
}

pub(super) fn run_build_contract(workspace: &Workspace, tools_csv: &str) -> Result<ContainerCommandOutcome> {
    let container_type = checked_container_type()?;
    run_environment_prep_for(
        workspace,
        &container_type,
        Some(tools_csv.to_string()),
        None,
    )
}

pub(super) fn run_test_images(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    ensure_no_args("test-images", args)?;
    let container_type = checked_container_type()?;
    let stage = env_or_empty("STAGE");
    let tools = env_or_empty("TOOLS");
    if container_type == "docker-arm64" {
        let tools_csv = if !stage.is_empty() {
            list_tools_for_stage(workspace, &stage)?
        } else if !tools.is_empty() {
            tools
        } else {
            primary_tools_csv(workspace)?
        };
        return run_environment_smoke_for(workspace, "docker-arm64", Some(tools_csv), None);
    }
    if !stage.is_empty() {
        return run_env_smoke(workspace, &[]);
    }
    if !tools.is_empty() {
        return run_env_smoke(workspace, &[]);
    }
    run_containers_smoke(workspace, &[])
}

pub(super) fn run_test_images_stage(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    ensure_no_args("test-images-stage", args)?;
    if env_or_empty("STAGE").is_empty() {
        return Ok(ContainerCommandOutcome {
            exit_code: 2,
            stdout: String::new(),
            stderr: "ERROR: set STAGE=<domain.stage|stage> (example: STAGE=fastq.trim_reads)\n"
                .to_string(),
        });
    }
    run_env_smoke(workspace, &[])
}

pub(super) fn run_test_images_tool(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    ensure_no_args("test-images-tool", args)?;
    if env_or_empty("TOOLS").is_empty() {
        return Ok(ContainerCommandOutcome {
            exit_code: 2,
            stdout: String::new(),
            stderr: "ERROR: set TOOLS=<tool_id>\n".to_string(),
        });
    }
    run_env_smoke(workspace, &[])
}

pub(super) fn run_image_smoke_vcf(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    ensure_no_args("image-smoke-vcf", args)?;
    let stages = run_argv(
        workspace,
        &[
            bijux_command_prefix(),
            vec!["registry".to_string(), "list-stages".to_string()],
        ]
        .concat(),
    )?;
    if !stages.is_success() {
        return Ok(stages);
    }
    let mut tools = BTreeSet::new();
    for stage in stages
        .stdout
        .lines()
        .map(str::trim)
        .filter(|stage| stage.starts_with("vcf."))
    {
        for tool in list_tools_for_stage(workspace, stage)?
            .split(',')
            .map(str::trim)
            .filter(|tool| !tool.is_empty())
        {
            tools.insert(tool.to_string());
        }
    }
    if tools.is_empty() {
        return Ok(ContainerCommandOutcome {
            exit_code: 2,
            stdout: String::new(),
            stderr: "ERROR: no VCF tools found via registry stage/tool mapping\n".to_string(),
        });
    }
    let tools_csv = tools.into_iter().collect::<Vec<_>>().join(",");
    if checked_container_type()? == "apptainer" {
        run_environment_smoke_for(workspace, "apptainer", Some(tools_csv), None)
    } else {
        run_environment_smoke_for(workspace, "docker-arm64", Some(tools_csv), None)
    }
}

pub(super) fn run_image_qa(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    ensure_no_args("image-qa", args)?;
    let container_type = checked_container_type()?;
    if container_type != "docker-arm64" {
        return Ok(ContainerCommandOutcome::success(format!(
            "skip: image-qa is docker-only (CONTAINER_TYPE={container_type})\n"
        )));
    }
    run_program_with_env(
        workspace,
        "cargo",
        &[
            "run".to_string(),
            "--bin".to_string(),
            "image_qa".to_string(),
            "--".to_string(),
            "--platform".to_string(),
            env_or_default("PLATFORM", "docker-arm64"),
        ],
        &artifact_env(workspace)?,
    )
}

pub(super) fn run_apptainer_ensure(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    ensure_no_args("apptainer-ensure", args)?;
    let domain = env_or_empty("DOMAIN");
    let stages = env_or_empty("STAGES");
    if domain.is_empty() || stages.is_empty() {
        return Ok(ContainerCommandOutcome {
            exit_code: 2,
            stdout: String::new(),
            stderr: "ERROR: set DOMAIN=<domain> and STAGES=<comma-separated>\nexample: make apptainer-ensure DOMAIN=fastq STAGES=validate_pre,trim,filter,stats,qc_post\n".to_string(),
        });
    }
    run_bijux_with_env(
        workspace,
        &[
            "env".to_string(),
            "ensure-images".to_string(),
            "--domain".to_string(),
            domain,
            "--stages".to_string(),
            stages,
        ],
        &[(
            "BIJUX_HPC_ROOT",
            env_or_default("BIJUX_HPC_ROOT", "$HOME/bijux"),
        )],
    )
}

pub(super) fn run_apptainer_ensure_stage(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    ensure_no_args("apptainer-ensure-stage", args)?;
    let domain = env_or_empty("DOMAIN");
    let stages = env_or_empty("STAGES");
    if domain.is_empty() || stages.is_empty() {
        return Ok(ContainerCommandOutcome {
            exit_code: 2,
            stdout: String::new(),
            stderr: "ERROR: set DOMAIN and STAGES for apptainer-ensure-stage\n".to_string(),
        });
    }
    run_bijux_with_env(
        workspace,
        &[
            "env".to_string(),
            "ensure-images".to_string(),
            "--domain".to_string(),
            domain,
            "--stages".to_string(),
            stages,
        ],
        &[(
            "BIJUX_HPC_ROOT",
            env_or_default("BIJUX_HPC_ROOT", "$HOME/bijux"),
        )],
    )
}

pub(super) fn run_registry_tools(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
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

pub(super) fn run_container_lint(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
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

pub(super) fn run_ensure_images(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
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

pub(super) fn run_container_doctor(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
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
        ("lock_file_drift", versioning::check_version_lock(workspace)?),
        ("lock_vs_built", check_lock_matches_built_output(workspace)?),
        ("outdated_versions", versioning::check_version_deprecations(workspace)?),
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

pub(super) fn run_release_gate(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
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

pub(super) fn run_vuln_scan_hook(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
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

pub(super) fn run_apptainer_build_all(
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

pub(super) fn run_docker_build_all(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
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

pub(super) fn current_host_name(workspace: &Workspace) -> String {
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

pub(super) fn run_build_apptainer_all(
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

pub(super) fn run_build_apptainer_hpc_frontend(
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

pub(super) fn run_apptainer_frontend_smoke(
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

pub(super) fn run_apptainer_frontend_security(
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
        ("check-version-hash-pin", versioning::check_version_hash_pin(workspace)?),
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

pub(super) fn run_apptainer_frontend_reproducibility(
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
