#![allow(clippy::too_many_lines)]

use super::{
    anyhow, apptainer_def_paths, apptainer_tool_ids, canonical_container_label_keys,
    container_support_summary, docker_tool_ids, dockerfile_paths, env_or_default, env_or_empty,
    failure_lines, fs, images_metadata, iso_root_path, load_toml, read_json, read_utf8,
    registry_tool_rows, success_line, table_array_strings, table_string, tool_status_manifest,
    tool_versions, toolkit_bundles, write_ensure_images_plan_report, write_vuln_hook_report,
    BTreeMap, BTreeSet, ContainerCommandOutcome, Context, PathBuf, Regex, Result, Utc, WalkDir,
    Workspace,
};

pub(in super::super) fn check_tool_name_collision(
    workspace: &Workspace,
) -> Result<ContainerCommandOutcome> {
    let images = images_metadata(workspace)?;
    let versions = tool_versions(workspace)?;
    let catalog_statuses = tool_status_manifest(workspace)?;
    let tool_ids = catalog_statuses.keys().cloned().collect::<BTreeSet<_>>();
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
    let registry_rows = registry_tool_rows(workspace)?
        .into_iter()
        .filter_map(|row| {
            let tool_id = table_string(&row, "id");
            let tool_id = if tool_id.is_empty() { table_string(&row, "tool_id") } else { tool_id };
            (!tool_id.is_empty()).then_some((tool_id, row))
        })
        .collect::<BTreeMap<_, _>>();
    let mut tools = BTreeMap::new();
    let mut bin_to_tool = BTreeMap::new();
    let mut errors = Vec::new();
    for (tool_id, status) in &catalog_statuses {
        let expected_bin = registry_rows
            .get(tool_id)
            .map(|row| table_string(row, "expected_bin"))
            .unwrap_or_default();
        tools.insert(tool_id.clone(), (expected_bin.clone(), status.clone()));
        if !expected_bin.is_empty() {
            if let Some(previous) = bin_to_tool.insert(expected_bin.clone(), tool_id.clone()) {
                if previous != *tool_id {
                    errors.push(format!(
                        "expected_bin collision: '{expected_bin}' used by both '{previous}' and '{tool_id}'"
                    ));
                }
            }
        }
    }
    let numeric_suffix_re = Regex::new(r"^([a-z_]+?)(\d+)$")?;
    for tool_id in tools.keys() {
        let Some(captures) = numeric_suffix_re.captures(tool_id) else {
            continue;
        };
        let base = captures.get(1).map(|value| value.as_str()).unwrap_or_default();
        if !tools.contains_key(base) {
            continue;
        }
        for candidate in [base.to_string(), tool_id.clone()] {
            if !images.contains_key(&candidate) {
                errors.push(format!("name-collision: missing images entry for '{candidate}'"));
            }
            if !versions.contains_key(&candidate) {
                errors.push(format!("name-collision: missing versions entry for '{candidate}'"));
            }
        }
        let base_bin = tools.get(base).map(|(bin, _)| bin.clone()).unwrap_or_default();
        let suffixed_bin = tools.get(tool_id).map(|(bin, _)| bin.clone()).unwrap_or_default();
        if !base_bin.is_empty() && base_bin == suffixed_bin {
            errors.push(format!(
                "name-collision: expected_bin must differ for '{base}' and '{tool_id}' (both '{base_bin}')"
            ));
        }
    }
    let surfaces = [
        ("catalog", tools.keys().cloned().collect::<BTreeSet<_>>()),
        (
            "images",
            images
                .iter()
                .filter(|&(_key, value)| value.is_table())
                .map(|(key, _value)| key.clone())
                .collect::<BTreeSet<_>>(),
        ),
        ("versions", versions.keys().cloned().collect::<BTreeSet<_>>()),
        ("tool_ids", tool_ids),
        ("docker", docker_ids),
        ("apptainer", apptainer_ids),
        ("domain_tools", domain_ids),
    ];
    let all_ids = surfaces.iter().flat_map(|(_, ids)| ids.iter().cloned()).collect::<BTreeSet<_>>();
    let norm_re = Regex::new(r"^[a-z][a-z0-9_]*$")?;
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
        if !present.contains(&"catalog")
            && present.iter().any(|name| {
                matches!(*name, "images" | "versions" | "tool_ids" | "docker" | "apptainer")
            })
        {
            errors.push(format!(
                "id parity: '{tool_id}' present in {present:?} but missing from governed container catalog"
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

pub(in super::super) fn check_tool_container_coverage(
    workspace: &Workspace,
) -> Result<ContainerCommandOutcome> {
    let images = images_metadata(workspace)?;
    let docker_ids = docker_tool_ids(workspace)?;
    let apptainer_ids = apptainer_tool_ids(workspace);
    let parity_exemptions = images
        .get("parity_exemptions")
        .and_then(toml::Value::as_table)
        .into_iter()
        .flat_map(|table| {
            table.iter().filter_map(|(tool_id, enabled)| {
                enabled.as_bool().filter(|enabled| *enabled).map(|_| tool_id.clone())
            })
        })
        .chain(
            images
                .get("apptainer_parity_exemptions")
                .and_then(toml::Value::as_table)
                .into_iter()
                .flat_map(|table| {
                    table.iter().filter_map(|(tool_id, enabled)| {
                        enabled.as_bool().filter(|enabled| *enabled).map(|_| tool_id.clone())
                    })
                }),
        )
        .collect::<BTreeSet<_>>();
    let support = container_support_summary(workspace)?;
    let mut errors = Vec::new();
    for row in registry_tool_rows(workspace)? {
        let tool_id = {
            let id = table_string(&row, "id");
            if id.is_empty() {
                table_string(&row, "tool_id")
            } else {
                id
            }
        };
        if support.get(&tool_id).is_none_or(|(disposition, _)| disposition != "retained") {
            continue;
        }
        let runtimes = table_array_strings(&row, "runtimes").into_iter().collect::<BTreeSet<_>>();
        let dockerfile = table_string(&row, "dockerfile");
        let apptainer_def = table_string(&row, "apptainer_def");
        if runtimes.contains("docker") && dockerfile.is_empty() {
            errors.push(format!("{tool_id}: runtime includes docker but dockerfile is unset"));
        }
        if runtimes.contains("apptainer") && apptainer_def.is_empty() {
            errors
                .push(format!("{tool_id}: runtime includes apptainer but apptainer_def is unset"));
        }
        if dockerfile.is_empty() && apptainer_def.is_empty() {
            errors.push(format!("{tool_id}: supported container tool has no container paths"));
        }
        if !dockerfile.is_empty() {
            let docker_path = workspace.path(&dockerfile);
            if !docker_path.exists() {
                errors.push(format!("{tool_id} dockerfile missing: {dockerfile}"));
            }
            let expected = format!("Dockerfile.arm64.{tool_id}");
            if docker_path.file_name().and_then(|name| name.to_str()) != Some(expected.as_str()) {
                errors.push(format!("{tool_id} dockerfile naming mismatch: expected {expected}"));
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
                errors.push(format!("{tool_id} apptainer naming mismatch: expected {expected}"));
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
            errors.push(format!("{tool_id}: docker coverage missing concrete Dockerfile"));
        }
        if !apptainer_def.is_empty() && !apptainer_ids.contains(&tool_id) {
            errors.push(format!("{tool_id}: apptainer coverage missing concrete definition"));
        }
    }
    if errors.is_empty() {
        return success_line("tool/container coverage: OK");
    }
    failure_lines("tool/container coverage check failed:", &errors)
}

pub(in super::super) fn check_toolkit_bundles(
    workspace: &Workspace,
) -> Result<ContainerCommandOutcome> {
    let bundles = toolkit_bundles(workspace)?;
    if bundles.is_empty() {
        return Ok(ContainerCommandOutcome::failure(
            "toolkit bundles: no [bundles.*] entries found\n",
        ));
    }
    let images = images_metadata(workspace)?;
    let docker_ids = docker_tool_ids(workspace)?;
    let apptainer_ids = apptainer_tool_ids(workspace);
    let support = container_support_summary(workspace)?;
    let mut errors = Vec::new();
    for (bundle_id, spec) in bundles {
        let tools = table_array_strings(&spec, "tools");
        if tools.is_empty() {
            errors.push(format!("{bundle_id}: tools must be a non-empty array"));
            continue;
        }
        for tool in tools {
            let Some((disposition, _)) = support.get(&tool) else {
                errors.push(format!("{bundle_id}: tool '{tool}' missing from container catalog"));
                continue;
            };
            if disposition != "retained" {
                errors.push(format!(
                    "{bundle_id}: tool '{tool}' has unsupported catalog disposition '{disposition}'"
                ));
                continue;
            }
            let Some(image_meta) = images.get(&tool).and_then(toml::Value::as_table) else {
                errors.push(format!("{bundle_id}: tool '{tool}' missing images.toml metadata"));
                continue;
            };
            if table_string(image_meta, "version").is_empty() {
                errors
                    .push(format!("{bundle_id}: tool '{tool}' images.toml entry missing version"));
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
                        "{bundle_id}: retained tool '{tool}' requires apptainer container"
                    ));
                }
                "docker_only" if !has_docker => {
                    errors.push(format!(
                        "{bundle_id}: retained tool '{tool}' requires docker container"
                    ));
                }
                "docker_apptainer" if !(has_apptainer && has_docker) => {
                    errors.push(format!(
                        "{bundle_id}: retained tool '{tool}' requires both docker and apptainer containers"
                    ));
                }
                "none" if !(has_apptainer || has_docker) => {
                    errors.push(format!(
                        "{bundle_id}: retained tool '{tool}' has no container definition"
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

pub(in super::super) fn check_hpc_image_naming(
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
        return Ok(ContainerCommandOutcome::failure("hpc image naming: missing config\n"));
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
    let tool_re =
        Regex::new(conf.get("tool_regex").and_then(toml::Value::as_str).unwrap_or_default())
            .context("invalid tool_regex in hpc_image_naming.toml")?;
    let version_re =
        Regex::new(conf.get("version_regex").and_then(toml::Value::as_str).unwrap_or_default())
            .context("invalid version_regex in hpc_image_naming.toml")?;
    let tag_format =
        conf.get("tag_format").and_then(toml::Value::as_str).unwrap_or_default().to_string();
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
            errors.push(format!("{tool}: version '{version}' does not match version_regex"));
        }
        let expected_tag = tag_format.replace("{tool}", &tool).replace("{version}", &version);
        let expected_ref = format!("{prefix}/{tool}:{expected_tag}");
        if image_ref != expected_ref {
            errors.push(format!(
                "{tool}: hpc_image_ref mismatch, expected {expected_ref}, got {image_ref}"
            ));
        }
    }
    let hpc_root = workspace.path("artifacts/containers/hpc");
    if hpc_root.exists() {
        for entry in WalkDir::new(&hpc_root).into_iter().filter_map(std::result::Result::ok) {
            let path = entry.path();
            if !entry.file_type().is_file()
                || path.extension().and_then(|ext| ext.to_str()) != Some("sif")
            {
                continue;
            }
            let stem = path.file_stem().and_then(|stem| stem.to_str()).unwrap_or_default();
            let normalized = stem.strip_prefix("sha256:").unwrap_or(stem).trim();
            if normalized.eq_ignore_ascii_case("pending")
                || (!normalized.is_empty() && normalized.chars().all(|char| char == '0'))
            {
                errors.push(format!(
                    "placeholder SIF artifact is forbidden: {}",
                    workspace.rel(path).display()
                ));
            }
        }
    }
    if errors.is_empty() {
        return success_line(format!("hpc image naming: OK ({})", rows.len()));
    }
    failure_lines("hpc image naming: FAILED", &errors)
}

pub(in super::super) fn check_bijux_template_markers(
    workspace: &Workspace,
) -> Result<ContainerCommandOutcome> {
    let template = workspace.path("containers/contracts/apptainer-recipe-contract.def.inc");
    let mut errors = Vec::new();
    if !template.exists() {
        errors.push(
            "missing template file containers/contracts/apptainer-recipe-contract.def.inc"
                .to_string(),
        );
    }
    for path in fs::read_dir(workspace.path("containers/apptainer"))
        .with_context(|| format!("read {}", workspace.path("containers/apptainer").display()))?
        .filter_map(std::result::Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("def"))
    {
        let head = read_utf8(&path)?.lines().take(20).collect::<Vec<_>>().join("\n");
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

pub(in super::super) fn check_tool_id_contract(
    workspace: &Workspace,
) -> Result<ContainerCommandOutcome> {
    let manifest = workspace.path("containers/TOOL_IDS.txt");
    if !manifest.is_file() {
        return Ok(ContainerCommandOutcome::failure(format!("missing {}\n", manifest.display())));
    }
    let lines = read_utf8(&manifest)?.lines().map(ToOwned::to_owned).collect::<Vec<_>>();
    let required_headers = [
        "# GENERATED FILE - DO NOT EDIT",
        "# Regenerate with: cargo run -p bijux-dna-dev -- containers run generate-tool-ids",
        "# format: <tool_id><TAB><support_disposition>",
    ];
    let allowed_status =
        ["retained", "unsupported", "retired", "excluded"].into_iter().collect::<BTreeSet<_>>();
    let tool_re = Regex::new(r"^[a-z][a-z0-9_]*$")?;
    let docker_ids = docker_tool_ids(workspace)?;
    let apptainer_ids = apptainer_tool_ids(workspace);
    let mut seen = BTreeSet::new();
    let mut status_by_id = BTreeMap::new();
    let mut errors = Vec::new();
    for (index, header) in required_headers.iter().enumerate() {
        if lines.get(index).map(std::string::String::as_str) != Some(*header) {
            errors.push(format!("header line {} mismatch: expected '{}'", index + 1, header));
        }
    }
    for (index, raw) in lines.iter().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts = raw.split('\t').collect::<Vec<_>>();
        if parts.len() != 2 {
            errors.push(format!("line {}: expected exactly 2 TAB-separated fields", index + 1));
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
        if status == "retained" {
            if ap_count != 1 {
                errors.push(format!(
                    "tool '{tool_id}' ({status}) must map to exactly one apptainer def (found {ap_count})"
                ));
            }
            if docker_count > 1 {
                errors.push(format!(
                    "tool '{tool_id}' ({status}) has ambiguous dockerfiles (found {docker_count})"
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

pub(in super::super) fn check_docker_arch_policy(
    workspace: &Workspace,
) -> Result<ContainerCommandOutcome> {
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
    for marker in ["build strategy", "publish strategy", "qualification criteria"] {
        if !text.to_ascii_lowercase().contains(marker) {
            errors.push(format!("policy doc missing required multiarch marker: {marker}"));
        }
    }
    for marker in ["cross-build", "buildx", "naming convention", "amd64"] {
        if !text.to_ascii_lowercase().contains(marker) {
            errors.push(format!("policy doc missing required amd64-plan marker: {marker}"));
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
                    .is_some_and(|name| name.starts_with("Dockerfile.arm64."))
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

pub(in super::super) fn check_docker_arm64_completeness(
    workspace: &Workspace,
) -> Result<ContainerCommandOutcome> {
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
    let waiver_path = workspace.path("containers/contracts/docker-waivers.toml");
    let mut waived = BTreeSet::new();
    if waiver_path.exists() {
        let data = load_toml(&waiver_path)?;
        for row in data.get("waiver").and_then(toml::Value::as_array).cloned().unwrap_or_default() {
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

pub(in super::super) fn check_docker_context(
    workspace: &Workspace,
) -> Result<ContainerCommandOutcome> {
    let mut errors = Vec::new();
    let scan_roots = [workspace.path("makes"), workspace.path("crates/bijux-dna-dev/src")];
    let broad_build_re = Regex::new(r"\bdocker\s+build\b.*\s\.\s*$")?;
    let host_copy_re = Regex::new(r"\b(COPY|ADD)\s+(\.\./|/Users/|~/)")?;
    let broad_context_copy_re = Regex::new(r"^(COPY|ADD)\s+\.\s")?;
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
            let ext = path.extension().and_then(|ext| ext.to_str()).unwrap_or_default();
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
    let dockerignore = workspace.path("containers/docker/.dockerignore");
    if dockerignore.exists() {
        let dockerignore_text = read_utf8(&dockerignore)?;
        for pattern in [".git", "artifacts", "assets", "**/*.pem", "**/*.key", ".env"] {
            if !dockerignore_text.contains(pattern) {
                errors
                    .push(format!("containers/docker/.dockerignore: missing pattern '{pattern}'"));
            }
        }
    } else {
        errors.push(
            "containers/docker/.dockerignore: missing (required for context minimization)"
                .to_string(),
        );
    }
    for path in dockerfile_paths(workspace)? {
        let text = read_utf8(&path)?;
        let lines = text.lines().collect::<Vec<_>>();
        for (index, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.ends_with('\\')
                && lines.get(index + 1).is_none_or(|next| next.trim().is_empty())
            {
                errors.push(format!(
                    "{}:{}: dangling Dockerfile line continuation before blank line or EOF",
                    workspace.rel(&path).display(),
                    index + 1
                ));
            }
            if broad_context_copy_re.is_match(trimmed) {
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

pub(in super::super) fn check_docker_hardening(
    workspace: &Workspace,
) -> Result<ContainerCommandOutcome> {
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
    let row_re = Regex::new(r"\|\s*`([^`]+)`\s*\|")?;
    let allowed = row_re
        .captures_iter(&read_utf8(&exceptions_doc)?)
        .filter_map(|captures| captures.get(1).map(|value| value.as_str().to_string()))
        .collect::<BTreeSet<_>>();
    let entrypoint_allowed = row_re
        .captures_iter(&read_utf8(&entrypoint_doc)?)
        .filter_map(|captures| captures.get(1).map(|value| value.as_str().to_string()))
        .collect::<BTreeSet<_>>();
    let required_labels = canonical_container_label_keys();
    let entrypoint_re = Regex::new(r"^ENTRYPOINT\s+\[")?;
    let cmd_re = Regex::new(r"^CMD\s+\[")?;
    let cmd_line_re = Regex::new(r"^CMD\s+\[(.+)\]\s*$")?;
    let user_re = Regex::new(r"^USER\s+(.+)$")?;
    let healthcheck_re = Regex::new(r"^HEALTHCHECK\s+(.+)$")?;
    let sh_entrypoint_re = Regex::new(r#"^ENTRYPOINT\s+\["/bin/sh",\s*"-c""#)?;
    let pipe_shell_re =
        Regex::new(r"curl\s+[^|\n]*\|\s*(bash|sh)\b|wget\s+[^|\n]*\|\s*(bash|sh)\b")?;
    let mut errors = Vec::new();
    for path in dockerfile_paths(workspace)? {
        let tool_id = path
            .file_name()
            .and_then(|name| name.to_str())
            .and_then(|name| name.strip_prefix("Dockerfile.arm64."))
            .unwrap_or_default()
            .to_string();
        let text = read_utf8(&path)?;
        let rel = workspace.rel(&path).display().to_string();
        for label in required_labels {
            if !text.contains(label) {
                errors.push(format!("{rel}: missing label {label}"));
            }
        }
        if pipe_shell_re.is_match(&text) {
            errors.push(format!("{rel}: forbidden curl|bash or wget|sh pattern"));
        }
        if recipe_ignores_build_failure(&text) {
            errors.push(format!(
                "{rel}: build commands must not convert failures into successful images"
            ));
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
                    captures.get(1).map(|value| value.as_str().to_ascii_lowercase())
                })
                .unwrap_or_default();
            if !entrypoint_exempt
                && !["--help", "-h", "--version"].iter().any(|needle| cmd_text.contains(needle))
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
            text.lines().find(|line| line.trim().starts_with("ENTRYPOINT")).unwrap_or_default(),
        ) && !entrypoint_exempt
        {
            errors.push(format!("{rel}: ENTRYPOINT must not use /bin/sh -c wrapper"));
        }
        let nonroot = text
            .lines()
            .filter_map(|line| user_re.captures(line.trim()))
            .filter_map(|captures| captures.get(1).map(|value| value.as_str().trim().to_string()))
            .any(|user| user != "root" && user != "0");
        if !nonroot && !allowed.contains(&tool_id) && !allowed.contains("*") {
            errors.push(format!("{rel}: no non-root USER and not listed in NONROOT_EXCEPTIONS.md"));
        }
        if text.contains("HEALTHCHECK") {
            let line = text
                .lines()
                .find_map(|line| healthcheck_re.captures(line.trim()))
                .and_then(|captures| captures.get(1).map(|value| value.as_str().to_string()))
                .unwrap_or_default();
            if !line.contains("--interval=") || !line.contains("--timeout=") {
                errors.push(format!("{rel}: HEALTHCHECK must define --interval and --timeout"));
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

fn recipe_ignores_build_failure(recipe: &str) -> bool {
    recipe.lines().any(|line| {
        let command = line.split('#').next().unwrap_or_default();
        command.contains("|| true")
            && ["make ", "make -", "make --", "cmake --build", "cargo build"]
                .iter()
                .any(|build| command.contains(build))
    })
}

pub(in super::super) fn check_docker_labels(
    workspace: &Workspace,
) -> Result<ContainerCommandOutcome> {
    let required = [
        "org.opencontainers.image.title",
        "org.opencontainers.image.version",
        "org.opencontainers.image.source",
        "org.opencontainers.image.licenses",
    ];
    let tool_re = Regex::new(r#"org\.opencontainers\.image\.tool="?([A-Za-z0-9_.-]+)"?"#)?;
    let version_re = Regex::new(r#"org\.opencontainers\.image\.version="?([A-Za-z0-9_.:-]+)"?"#)?;
    let apptainer_version_re = Regex::new(r"org\.opencontainers\.image\.version\s+([^\s]+)")?;
    let mut docker_versions = BTreeMap::new();
    let mut errors = Vec::new();
    for path in dockerfile_paths(workspace)? {
        let text = read_utf8(&path)?;
        let rel = workspace.rel(&path).display().to_string();
        let missing =
            required.iter().filter(|label| !text.contains(**label)).copied().collect::<Vec<_>>();
        if !missing.is_empty() {
            errors.push(format!("{rel} missing labels: {}", missing.join(", ")));
        }
        let tool_id = path
            .file_name()
            .and_then(|name| name.to_str())
            .and_then(|name| name.strip_prefix("Dockerfile.arm64."))
            .unwrap_or_default()
            .to_string();
        if let Some(captures) = tool_re.captures(&text) {
            let label = captures.get(1).map(|value| value.as_str()).unwrap_or_default();
            if label != tool_id {
                errors.push(format!("{rel} tool label mismatch: {label} != {tool_id}"));
            }
        }
        if let Some(captures) = version_re.captures(&text) {
            docker_versions.insert(
                tool_id,
                captures.get(1).map(|value| value.as_str().to_string()).unwrap_or_default(),
            );
        }
        if text.contains("/opt/bijux/VERSION.json") || text.contains("bijux-tool-info") {
            errors.push(format!(
                "{rel}: duplicate in-image self-report metadata is forbidden; publish metadata must flow through OCI labels"
            ));
        }
    }
    for path in apptainer_def_paths(workspace) {
        let tool_id =
            path.file_stem().and_then(|name| name.to_str()).unwrap_or_default().to_string();
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

pub(in super::super) fn check_docker_unpinned_apt(
    workspace: &Workspace,
) -> Result<ContainerCommandOutcome> {
    let mut errors = Vec::new();
    for dockerfile in dockerfile_paths(workspace)? {
        let rel = workspace.rel(&dockerfile).display().to_string();
        errors.extend(docker_apt_policy_violations(&rel, &read_utf8(&dockerfile)?)?);
    }
    if errors.is_empty() {
        return success_line("docker apt pin check: OK");
    }
    failure_lines("docker apt pin check: FAILED", &errors)
}

fn docker_apt_policy_violations(rel: &str, dockerfile: &str) -> Result<Vec<String>> {
    let immutable_snapshot_re = Regex::new(
        r"https://snapshot\.(?:ubuntu\.com/ubuntu|debian\.org/archive/(?:debian|debian-security))/[0-9]{8}T[0-9]{6}Z/?",
    )?;
    let apt_install_re = Regex::new(r"\bapt(?:-get)?\b[^;&|]*?\binstall\b")?;
    let option_re = Regex::new(r"--[a-zA-Z0-9-]+(?:=[^\s]+)?")?;
    let logical_lines =
        dockerfile.lines().map(str::trim).fold(Vec::<String>::new(), |mut lines, line| {
            if let Some(previous) = lines.last_mut().filter(|previous| previous.ends_with('\\')) {
                previous.pop();
                previous.push(' ');
                previous.push_str(line);
            } else {
                lines.push(line.to_string());
            }
            lines
        });
    let governed_helper_source = "containers/contracts/configure-apt-snapshot";
    let governed_helper_target = "/usr/local/sbin/bijux-configure-apt-snapshot";
    let mut snapshot_active = false;
    let mut governed_helper_available = false;
    let mut errors = Vec::new();
    if dockerfile.contains("Acquire::https::Verify-Peer=false") {
        errors.push(format!(
            "{rel}: disabling apt TLS peer verification is forbidden; use the governed snapshot bootstrap"
        ));
    }
    for instruction in logical_lines {
        if instruction.starts_with("FROM ") {
            snapshot_active = false;
            governed_helper_available = false;
            continue;
        }
        if instruction.starts_with("COPY ")
            && instruction.contains(governed_helper_source)
            && instruction.contains(governed_helper_target)
        {
            governed_helper_available = true;
        }
        let helper_offset = instruction.find(governed_helper_target);
        if helper_offset.is_some() && !governed_helper_available {
            errors.push(format!(
                "{rel}: governed apt snapshot helper is invoked before its canonical source is copied into the build stage"
            ));
        }
        let snapshot_offsets = immutable_snapshot_re
            .find_iter(&instruction)
            .map(|matched| matched.start())
            .collect::<Vec<_>>();
        for command in apt_install_re.find_iter(&instruction) {
            let governed_helper_precedes_command = governed_helper_available
                && helper_offset.is_some_and(|offset| offset < command.start());
            let snapshot_precedes_command = snapshot_active
                || governed_helper_precedes_command
                || snapshot_offsets.iter().any(|offset| *offset < command.start());
            let tail = &instruction[command.end()..];
            let end = tail.find("&&").or_else(|| tail.find("||")).or_else(|| tail.find(';'));
            let package_segment = end.map_or(tail, |index| &tail[..index]);
            let package_segment = option_re.replace_all(package_segment, " ");
            let unpinned = package_segment
                .split_whitespace()
                .filter(|token| {
                    !token.is_empty()
                        && !token.starts_with('-')
                        && !token.starts_with('$')
                        && !token.starts_with('"')
                        && !token.starts_with('/')
                        && !token.contains('=')
                })
                .collect::<Vec<_>>();
            if !snapshot_precedes_command {
                errors.extend(unpinned.into_iter().map(|package| {
                    format!(
                        "{rel}: apt package '{package}' requires an exact version or an immutable snapshot configured earlier in its build stage"
                    )
                }));
            }
        }
        if !snapshot_offsets.is_empty() {
            snapshot_active = true;
        }
        if governed_helper_available && helper_offset.is_some() {
            snapshot_active = true;
        }
    }
    Ok(errors)
}

pub(in super::super) fn check_docker_version_sync(
    workspace: &Workspace,
) -> Result<ContainerCommandOutcome> {
    let versions = tool_versions(workspace)?;
    let arg_re = Regex::new(r"^ARG\s+TOOL_VERSION\s*=\s*([^\s#]+)\s*$")?;
    let mut errors = Vec::new();
    for dockerfile in dockerfile_paths(workspace)? {
        let tool = dockerfile
            .file_name()
            .and_then(|name| name.to_str())
            .and_then(|name| name.strip_prefix("Dockerfile.arm64."))
            .unwrap_or_default()
            .to_string();
        let text = read_utf8(&dockerfile)?;
        let Some(docker_version) =
            text.lines().find_map(|line| arg_re.captures(line.trim())).and_then(|captures| {
                captures.get(1).map(|value| {
                    value.as_str().trim().trim_matches('"').trim_matches('\'').to_string()
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
        if docker_version != registry_version {
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

pub(in super::super) fn check_dockerfiles_built(
    workspace: &Workspace,
) -> Result<ContainerCommandOutcome> {
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
                .and_then(|name| name.strip_prefix("Dockerfile.arm64."))
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
            row.get("manifest").and_then(serde_json::Value::as_str).unwrap_or_default(),
        );
        if !manifest_path.exists() {
            errors.push(format!("{tool}: manifest missing at {}", manifest_path.display()));
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

pub(in super::super) fn check_no_secrets(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let mut scan = Vec::new();
    scan.extend(apptainer_def_paths(workspace));
    scan.extend(dockerfile_paths(workspace)?);
    let patterns = [
        Regex::new(r"AKIA[0-9A-Z]{16}")?,
        Regex::new(r#"(?i)(secret|token|password)\s*[:=]\s*['"]?[A-Za-z0-9_\-]{8,}"#)?,
        Regex::new(r"ghp_[A-Za-z0-9]{20,}")?,
        Regex::new(r"github_pat_[A-Za-z0-9_]{20,}")?,
        Regex::new(r"xox[baprs]-[A-Za-z0-9-]{10,}")?,
        Regex::new(r"AIza[0-9A-Za-z\-_]{35}")?,
        Regex::new(r#"(?i)aws_secret_access_key\s*[:=]\s*['"]?[A-Za-z0-9/+=]{30,}"#)?,
        Regex::new(r"(?i)-----BEGIN (?:RSA|OPENSSH|EC) PRIVATE KEY-----")?,
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

pub(in super::super) fn check_runtime_downloads(
    workspace: &Workspace,
) -> Result<ContainerCommandOutcome> {
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
                    entry.file_stem().and_then(|name| name.to_str()).unwrap_or_default()
                })
                .trim()
                .to_string();
            runtime_allowed.insert(
                tool_id,
                value.get("runtime_network").and_then(toml::Value::as_bool).unwrap_or(false),
            );
        }
    }
    let download_re = Regex::new(r"\b(curl|wget)\b")?;
    let mut errors = Vec::new();
    for path in apptainer_def_paths(workspace) {
        let text = read_utf8(&path)?;
        let tool = path.file_stem().and_then(|name| name.to_str()).unwrap_or_default().to_string();
        let mut chunks = Vec::new();
        if let Some(runscript) =
            text.split("%runscript").nth(1).and_then(|body| body.split("\n%").next())
        {
            chunks.push(runscript.to_string());
        }
        if let Some(environment) =
            text.split("%environment").nth(1).and_then(|body| body.split("\n%").next())
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
            .and_then(|name| name.strip_prefix("Dockerfile.arm64."))
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

pub(in super::super) fn check_vuln_allowlist(
    workspace: &Workspace,
) -> Result<ContainerCommandOutcome> {
    let path = std::env::var("ALLOWLIST")
        .map_or_else(|_| workspace.path("configs/ci/tools/vuln_allowlist.toml"), PathBuf::from);
    if !path.exists() {
        return Ok(ContainerCommandOutcome::failure(format!(
            "vuln allowlist: missing {}\n",
            path.display()
        )));
    }
    let data = load_toml(&path)?;
    let cve_re = Regex::new(r"^CVE-\d{4}-\d{4,}$")?;
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

pub(in super::super) fn check_vuln_hook(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
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
        env_or_default("RETAINED_ONLY", "1") == "1",
    )?;
    if !out.is_file() {
        return Ok(ContainerCommandOutcome::failure(format!(
            "vuln hook: missing report {}\n",
            out.display()
        )));
    }
    let payload = read_json(&out)?;
    let items =
        payload.get("items").and_then(serde_json::Value::as_array).cloned().unwrap_or_default();
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
    let retained = container_support_summary(workspace)?
        .into_iter()
        .filter_map(|(tool, (disposition, _))| (disposition == "retained").then_some(tool))
        .collect::<Vec<_>>();
    if rows.is_empty() && env_or_empty("CI").is_empty() {
        return success_line("vuln hook: SKIP (no local vuln scan items)");
    }
    let retained_only = matches!(
        env_or_default("RETAINED_ONLY", "1").trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes"
    );
    let mut errors = Vec::new();
    if retained_only && !retained.is_empty() {
        for tool in retained {
            let Some(row) = rows.get(&tool) else {
                errors.push(format!("{tool}: missing vuln scan item for retained tool"));
                continue;
            };
            let status = row.get("status").and_then(serde_json::Value::as_str).unwrap_or_default();
            if !matches!(status, "ok" | "not_scanned") {
                errors.push(format!("{tool}: vuln scan status is {status}"));
            }
            let per_tool = workspace.path(&format!("artifacts/containers/vuln/{tool}.json"));
            if !per_tool.exists() {
                errors
                    .push(format!("{tool}: missing per-tool vuln summary {}", per_tool.display()));
            }
        }
    }
    if errors.is_empty() {
        return success_line("vuln hook: OK");
    }
    failure_lines("vuln hook: FAILED", &errors)
}

pub(in super::super) fn check_sbom_artifacts(
    workspace: &Workspace,
) -> Result<ContainerCommandOutcome> {
    let manifest_root = workspace.path("artifacts/containers");
    if !manifest_root.exists() {
        if !env_or_empty("CI").is_empty() {
            return Ok(ContainerCommandOutcome::failure(
                "sbom artifacts: missing artifacts/containers\n",
            ));
        }
        return success_line("sbom artifacts: SKIP (no artifacts/containers)");
    }
    let strict_retained =
        !env_or_empty("CI").is_empty() || env_or_default("REQUIRE_RETAINED_SBOM", "0") == "1";
    let retained = container_support_summary(workspace)?
        .into_iter()
        .filter_map(|(tool, (disposition, _))| (disposition == "retained").then_some(tool))
        .collect::<BTreeSet<_>>();
    let mut manifests = BTreeMap::<String, Vec<(PathBuf, serde_json::Value)>>::new();
    for path in fs::read_dir(&manifest_root)
        .with_context(|| format!("read {}", manifest_root.display()))?
        .filter_map(std::result::Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("json"))
    {
        let name = path.file_name().and_then(|name| name.to_str()).unwrap_or_default();
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
    let tools_to_check = if strict_retained {
        retained.iter().cloned().collect::<Vec<_>>()
    } else {
        let shared =
            manifests.keys().filter(|tool| retained.contains(*tool)).cloned().collect::<Vec<_>>();
        if shared.is_empty() {
            manifests.keys().cloned().collect::<Vec<_>>()
        } else {
            shared
        }
    };
    if tools_to_check.is_empty() && !strict_retained {
        return success_line("sbom artifacts: SKIP (no local build manifests)");
    }
    let mut seen = 0;
    let mut errors = Vec::new();
    for tool in tools_to_check {
        let rows = manifests.get(&tool).cloned().unwrap_or_default();
        if rows.is_empty() {
            errors
                .push(format!("{tool}: missing smoke/build manifest under artifacts/containers/"));
            continue;
        }
        let ok_rows = rows
            .into_iter()
            .filter(|(_, data)| {
                data.get("status").and_then(serde_json::Value::as_str) == Some("ok")
            })
            .collect::<Vec<_>>();
        if ok_rows.is_empty() {
            errors.push(format!("{tool}: has manifests but no successful status=ok result"));
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
                errors
                    .push(format!("{}: sbom_path does not exist: {sbom}", manifest_path.display()));
            } else if sbom_path.metadata().map_or(0, |meta| meta.len()) == 0 {
                errors.push(format!("{}: sbom_path is empty: {sbom}", manifest_path.display()));
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
    if errors.is_empty() && seen == 0 {
        return success_line("sbom artifacts: SKIP (no successful local build manifests)");
    }
    if errors.is_empty() {
        return success_line(format!("sbom artifacts: OK ({seen} manifests)"));
    }
    failure_lines("sbom artifacts: FAILED", &errors)
}

pub(in super::super) fn check_time_locale_determinism(
    workspace: &Workspace,
) -> Result<ContainerCommandOutcome> {
    let mut errors = Vec::new();
    for path in apptainer_def_paths(workspace) {
        let text = read_utf8(&path)?;
        let env = text
            .split("%environment")
            .nth(1)
            .and_then(|body| body.split("\n%").next())
            .unwrap_or_default();
        if !env.contains("TZ=UTC") {
            errors
                .push(format!("{}: %environment must set TZ=UTC", workspace.rel(&path).display()));
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

pub(in super::super) fn check_tool_invocation_normalization(
    workspace: &Workspace,
) -> Result<ContainerCommandOutcome> {
    let mut errors = Vec::new();
    for row in registry_tool_rows(workspace)? {
        let runtimes = table_array_strings(&row, "runtimes");
        if !runtimes.iter().any(|runtime| runtime == "apptainer" || runtime == "docker") {
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

pub(in super::super) fn check_smoke_inputs_policy(
    workspace: &Workspace,
) -> Result<ContainerCommandOutcome> {
    let policy = workspace.path("configs/ci/tools/smoke_inputs_policy.toml");
    if !policy.is_file() {
        return Ok(ContainerCommandOutcome::failure(format!(
            "smoke-inputs policy: missing {}\n",
            policy.display()
        )));
    }
    let data = load_toml(&policy)?;
    let entries =
        data.get("tool_inputs").and_then(toml::Value::as_table).cloned().unwrap_or_default();
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
        if path.metadata().map_or(0, |meta| meta.len()) == 0 {
            errors.push(format!("{tool}: input file is empty {rel}"));
        }
    }
    if errors.is_empty() {
        return success_line(format!("smoke-inputs policy: OK ({})", entries.len()));
    }
    failure_lines("smoke-inputs policy: FAILED", &errors)
}

#[cfg(test)]
mod tests {
    use super::{docker_apt_policy_violations, recipe_ignores_build_failure};

    #[test]
    fn hardening_policy_rejects_ignored_build_failures() {
        assert!(recipe_ignores_build_failure("RUN make -j2 || true"));
        assert!(recipe_ignores_build_failure("RUN cmake --build build || true"));
        assert!(!recipe_ignores_build_failure("RUN make -j2"));
        assert!(!recipe_ignores_build_failure("RUN tool --help >/dev/null 2>&1 || true"));
    }

    #[test]
    fn apt_policy_accepts_exact_bootstrap_before_snapshot_activation() {
        let dockerfile = r"
FROM ubuntu:24.04
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates=20240203 && \
    sed -i 's#archive.ubuntu.com#https://snapshot.ubuntu.com/ubuntu/20260802T000000Z/#' /etc/apt/sources.list && \
    apt-get update && \
    apt-get install -y curl zlib1g
";

        let violations = docker_apt_policy_violations("Dockerfile", dockerfile).unwrap();

        assert!(violations.is_empty(), "{violations:?}");
    }

    #[test]
    fn apt_policy_accepts_debian_snapshot_packages() {
        let dockerfile = r"
FROM debian:bookworm
RUN sed -i 's#deb.debian.org/debian#https://snapshot.debian.org/archive/debian/20260802T000000Z#' /etc/apt/sources.list && \
    apt-get update && \
    apt-get install -y curl zlib1g
";

        let violations = docker_apt_policy_violations("Dockerfile", dockerfile).unwrap();

        assert!(violations.is_empty(), "{violations:?}");
    }

    #[test]
    fn apt_policy_accepts_canonical_snapshot_helper_per_stage() {
        let dockerfile = r"
FROM ubuntu:24.04
COPY --chmod=0555 containers/contracts/configure-apt-snapshot /usr/local/sbin/bijux-configure-apt-snapshot
RUN /usr/local/sbin/bijux-configure-apt-snapshot && apt-get install -y curl zlib1g
";

        let violations = docker_apt_policy_violations("Dockerfile", dockerfile).unwrap();

        assert!(violations.is_empty(), "{violations:?}");
    }

    #[test]
    fn apt_policy_rejects_uncopied_snapshot_helper() {
        let dockerfile = r"
FROM ubuntu:24.04
RUN /usr/local/sbin/bijux-configure-apt-snapshot && apt-get install -y curl
";

        let violations = docker_apt_policy_violations("Dockerfile", dockerfile).unwrap();

        assert_eq!(violations.len(), 2);
        assert!(violations.iter().any(|row| row.contains("before its canonical source")));
        assert!(violations.iter().any(|row| row.contains("'curl'")));
    }

    #[test]
    fn apt_policy_rejects_mutable_unversioned_packages() {
        let dockerfile = r"
FROM ubuntu:24.04
RUN apt-get update && \
    apt-get install -y --no-install-recommends curl zlib1g
";

        let violations = docker_apt_policy_violations("Dockerfile", dockerfile).unwrap();

        assert_eq!(violations.len(), 2);
        assert!(violations.iter().any(|row| row.contains("'curl'")));
        assert!(violations.iter().any(|row| row.contains("'zlib1g'")));
    }

    #[test]
    fn apt_policy_rejects_disabled_tls_verification() {
        let dockerfile = r"
FROM ubuntu:24.04
RUN apt-get -o Acquire::https::Verify-Peer=false update
";

        let violations = docker_apt_policy_violations("Dockerfile", dockerfile).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].contains("TLS peer verification"));
    }

    #[test]
    fn apt_policy_resets_snapshot_authority_between_build_stages() {
        let dockerfile = r"
FROM ubuntu:24.04 AS builder
RUN sed -i 's#archive.ubuntu.com#https://snapshot.ubuntu.com/ubuntu/20260802T000000Z/#' /etc/apt/sources.list && \
    apt-get update && apt-get install -y build-essential
FROM ubuntu:24.04
RUN apt-get update && apt-get install -y zlib1g
";

        let violations = docker_apt_policy_violations("Dockerfile", dockerfile).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].contains("'zlib1g'"));
    }
}
