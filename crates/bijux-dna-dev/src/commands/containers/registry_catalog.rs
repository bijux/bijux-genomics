use super::{
    anyhow, fs, json_string_pretty, load_toml, out_path_arg, read_json, read_utf8,
    run_program_with_env, sha256_hex, table_array_strings, table_bool, table_string, write_utf8,
    BTreeMap, BTreeSet, ContainerCommandOutcome, Context, PathBuf, Result, WalkDir, Workspace,
};

const REGISTRY_PATHS: [&str; 5] = [
    "configs/ci/registry/tool_registry.toml",
    "configs/ci/registry/tool_registry_experimental.toml",
    "configs/ci/registry/tool_registry_container_experimental.toml",
    "configs/ci/registry/tool_registry_vcf.toml",
    "configs/ci/registry/tool_registry_vcf_downstream.toml",
];

fn expected_source_identity(row: &toml::map::Map<String, toml::Value>) -> String {
    let source_sha256 = table_string(row, "source_sha256");
    if !source_sha256.is_empty() {
        return format!("sha256:{source_sha256}");
    }
    let pinned_commit = table_string(row, "pinned_commit");
    if !pinned_commit.is_empty() {
        return format!("git:{pinned_commit}");
    }
    "unresolved".to_string()
}

fn expected_pin_strategy(row: &toml::map::Map<String, toml::Value>) -> &'static str {
    if !table_string(row, "source_sha256").is_empty() {
        "source_sha256"
    } else if !table_string(row, "pinned_commit").is_empty() {
        "source_commit"
    } else {
        "unresolved"
    }
}

fn expected_runtimes(row: &toml::map::Map<String, toml::Value>) -> Vec<String> {
    let mut runtimes = Vec::new();
    if table_bool(row, "docker_arm64_supported") {
        runtimes.push("docker".to_string());
    }
    if table_bool(row, "apptainer_supported") {
        runtimes.push("apptainer".to_string());
    }
    runtimes
}

fn compare_text_field(
    violations: &mut Vec<String>,
    registry_path: &str,
    tool_id: &str,
    registry: &toml::map::Map<String, toml::Value>,
    registry_field: &str,
    expected: &str,
) {
    let actual = table_string(registry, registry_field);
    if actual != expected {
        violations.push(format!(
            "{registry_path}: {tool_id} {registry_field} expected {expected:?}, observed {actual:?}"
        ));
    }
}

fn compare_registry_row(
    violations: &mut Vec<String>,
    registry_path: &str,
    registry: &toml::map::Map<String, toml::Value>,
    catalog: &toml::map::Map<String, toml::Value>,
) {
    let tool_id = table_string(registry, "id");
    for (registry_field, catalog_field) in [
        ("version", "version"),
        ("default_version", "version"),
        ("upstream", "upstream"),
        ("license", "license"),
        ("expected_bin", "expected_bin"),
        ("version_cmd", "version_command"),
        ("smoke_version_cmd", "version_command"),
        ("help_cmd", "help_command"),
        ("smoke_help_cmd", "help_command"),
        ("expected_version_regex", "expected_version_regex"),
        ("dockerfile", "dockerfile"),
        ("apptainer_def", "apptainer_definition"),
    ] {
        compare_text_field(
            violations,
            registry_path,
            &tool_id,
            registry,
            registry_field,
            &table_string(catalog, catalog_field),
        );
    }

    let runtimes = expected_runtimes(catalog);
    let actual_runtimes = table_array_strings(registry, "runtimes");
    if actual_runtimes != runtimes {
        violations.push(format!(
            "{registry_path}: {tool_id} runtimes expected {runtimes:?}, observed {actual_runtimes:?}"
        ));
    }
    let expected_container = !runtimes.is_empty();
    if table_bool(registry, "container") != expected_container {
        violations
            .push(format!("{registry_path}: {tool_id} container must be {expected_container}"));
    }
    let expected_ref = if expected_container {
        format!("bijuxdna/{tool_id}:{}", table_string(catalog, "version"))
    } else {
        "external@licensed-installation".to_string()
    };
    compare_text_field(
        violations,
        registry_path,
        &tool_id,
        registry,
        "container_ref",
        &expected_ref,
    );
    compare_text_field(
        violations,
        registry_path,
        &tool_id,
        registry,
        "pinned_commit",
        &expected_source_identity(catalog),
    );
    compare_text_field(
        violations,
        registry_path,
        &tool_id,
        registry,
        "pin_strategy",
        expected_pin_strategy(catalog),
    );

    let disposition = table_string(catalog, "support_disposition");
    if disposition != "retained" {
        compare_text_field(violations, registry_path, &tool_id, registry, "status", &disposition);
        if table_bool(registry, "container")
            || !table_string(registry, "dockerfile").is_empty()
            || !table_string(registry, "apptainer_def").is_empty()
        {
            violations.push(format!(
                "{registry_path}: {tool_id} {disposition} tool claims a container recipe"
            ));
        }
    }
}

pub(super) fn check_registry_drift(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    let output = out_path_arg(
        workspace,
        args,
        "artifacts/containers/registry-drift-report.json",
        "Usage: cargo run -p bijux-dna-dev -- containers run check-registry-drift -- [<output-path>]",
    )?;
    let catalog_path = workspace.path("containers/contracts/catalog.toml");
    let catalog_raw = read_utf8(&catalog_path)?;
    let catalog_document: toml::Value = toml::from_str(&catalog_raw)?;
    let catalog_rows =
        catalog_document.get("tools").and_then(toml::Value::as_array).cloned().unwrap_or_default();
    let catalog_order = catalog_rows
        .iter()
        .filter_map(toml::Value::as_table)
        .map(|row| table_string(row, "id"))
        .collect::<Vec<_>>();
    let catalog = catalog_rows
        .into_iter()
        .filter_map(|row| row.as_table().cloned())
        .map(|row| (table_string(&row, "id"), row))
        .collect::<BTreeMap<_, _>>();

    let mut violations = Vec::new();
    let mut registry_summaries = Vec::new();
    let mut projected_ids = BTreeSet::new();
    for relative_path in REGISTRY_PATHS {
        let raw = read_utf8(&workspace.path(relative_path))?;
        let document: toml::Value = toml::from_str(&raw)?;
        let rows =
            document.get("tools").and_then(toml::Value::as_array).cloned().unwrap_or_default();
        let mut registry_ids = BTreeSet::new();
        for row in &rows {
            let Some(registry) = row.as_table() else {
                violations.push(format!("{relative_path}: non-table tool row"));
                continue;
            };
            let tool_id = table_string(registry, "id");
            if !registry_ids.insert(tool_id.clone()) {
                violations.push(format!("{relative_path}: duplicate tool {tool_id:?}"));
                continue;
            }
            let Some(contract) = catalog.get(&tool_id) else {
                violations.push(format!("{relative_path}: unknown tool {tool_id:?}"));
                continue;
            };
            projected_ids.insert(tool_id);
            compare_registry_row(&mut violations, relative_path, registry, contract);
        }
        registry_summaries.push(serde_json::json!({
            "path": relative_path,
            "sha256": sha256_hex(raw.as_bytes()),
            "tool_count": rows.len(),
        }));
    }

    let unprojected_tool_ids = catalog
        .keys()
        .filter(|tool_id| !projected_ids.contains(*tool_id))
        .cloned()
        .collect::<Vec<_>>();
    for tool_id in &unprojected_tool_ids {
        let disposition = catalog.get(tool_id).map(|row| table_string(row, "support_disposition"));
        if disposition.as_deref() == Some("retained") {
            violations
                .push(format!("retained catalog tool {tool_id:?} has no registry projection"));
        }
    }

    let manifest_order = read_utf8(&workspace.path("containers/TOOL_IDS.txt"))?
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
        .filter_map(|line| line.split_once('\t').map(|(tool_id, _)| tool_id.to_string()))
        .collect::<Vec<_>>();
    if manifest_order != catalog_order {
        violations
            .push("containers/TOOL_IDS.txt does not preserve exact catalog order".to_string());
    }

    let passed = violations.is_empty();
    let report = serde_json::json!({
        "schema_version": "bijux.container.registry_drift.v1",
        "catalog": {
            "path": "containers/contracts/catalog.toml",
            "sha256": sha256_hex(catalog_raw.as_bytes()),
            "tool_count": catalog.len(),
        },
        "tool_id_manifest_matches_catalog_order": manifest_order == catalog_order,
        "projected_tool_count": projected_ids.len(),
        "unprojected_tool_ids": unprojected_tool_ids,
        "registries": registry_summaries,
        "violation_count": violations.len(),
        "violations": violations,
        "passed": passed,
    });
    write_utf8(&output, &json_string_pretty(&report)?)?;
    if report.get("passed").and_then(serde_json::Value::as_bool) == Some(true) {
        return Ok(ContainerCommandOutcome::success(format!(
            "registry drift: OK ({})\n",
            output.display()
        )));
    }
    Ok(ContainerCommandOutcome::failure(format!(
        "registry drift: FAILED; inspect {}\n",
        output.display()
    )))
}

pub(super) fn canonical_container_label_keys() -> [&'static str; 8] {
    [
        "org.opencontainers.image.source",
        "org.opencontainers.image.revision",
        "org.opencontainers.image.created",
        "org.opencontainers.image.licenses",
        "org.opencontainers.image.version",
        "io.bijux.tool.version",
        "org.opencontainers.image.tool",
        "org.opencontainers.image.title",
    ]
}

pub(super) fn missing_container_label_markers(text: &str) -> Vec<&'static str> {
    canonical_container_label_keys().into_iter().filter(|label| !text.contains(label)).collect()
}

pub(super) fn docker_image_labels(
    workspace: &Workspace,
    image: &str,
) -> Result<BTreeMap<String, String>> {
    let inspect = run_program_with_env(
        workspace,
        "docker",
        &[
            "image".to_string(),
            "inspect".to_string(),
            image.to_string(),
            "--format".to_string(),
            "{{json .Config.Labels}}".to_string(),
        ],
        &[],
    )?;
    if !inspect.is_success() {
        return Err(anyhow!("docker image inspect failed for {image}: {}", inspect.stderr.trim()));
    }
    let stdout = inspect.stdout.trim();
    if stdout.is_empty() || stdout == "null" {
        return Ok(BTreeMap::new());
    }
    serde_json::from_str(stdout).with_context(|| format!("parse docker labels for {image}"))
}

pub(super) fn canonical_metadata_labels(
    labels: &BTreeMap<String, String>,
) -> BTreeMap<&'static str, String> {
    canonical_container_label_keys()
        .into_iter()
        .filter_map(|key| labels.get(key).cloned().map(|value| (key, value)))
        .collect()
}

pub(super) fn registry_tool_rows(
    workspace: &Workspace,
) -> Result<Vec<toml::map::Map<String, toml::Value>>> {
    let mut rows = Vec::new();
    for rel in [
        "configs/ci/registry/tool_registry.toml",
        "configs/ci/registry/tool_registry_vcf.toml",
        "configs/ci/registry/tool_registry_container_experimental.toml",
        "configs/ci/registry/tool_registry_experimental.toml",
        "configs/ci/registry/tool_registry_vcf_downstream.toml",
    ] {
        let value = load_toml(&workspace.path(rel))?;
        if let Some(entries) = value.get("tools").and_then(toml::Value::as_array) {
            for entry in entries {
                if let Some(table) = entry.as_table() {
                    rows.push(table.clone());
                }
            }
        }
    }
    Ok(rows)
}

pub(super) fn registry_tool_map(
    workspace: &Workspace,
) -> Result<BTreeMap<String, toml::map::Map<String, toml::Value>>> {
    let mut rows = BTreeMap::new();
    for row in registry_tool_rows(workspace)? {
        let tool_id = row
            .get("id")
            .or_else(|| row.get("tool_id"))
            .and_then(toml::Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_string();
        if !tool_id.is_empty() {
            rows.insert(tool_id, row);
        }
    }
    Ok(rows)
}

pub(super) fn governed_container_file_ids(workspace: &Workspace) -> Result<BTreeSet<String>> {
    let mut ids = BTreeSet::new();
    for entry in fs::read_dir(workspace.path("containers/docker"))
        .with_context(|| format!("read {}", workspace.path("containers/docker").display()))?
        .filter_map(std::result::Result::ok)
    {
        if let Some(tool_id) =
            entry.file_name().to_str().and_then(|name| name.strip_prefix("Dockerfile.arm64."))
        {
            ids.insert(tool_id.to_string());
        }
    }
    for entry in fs::read_dir(workspace.path("containers/apptainer"))
        .with_context(|| format!("read {}", workspace.path("containers/apptainer").display()))?
        .filter_map(std::result::Result::ok)
    {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("def") {
            if let Some(tool_id) = path.file_stem().and_then(|name| name.to_str()) {
                ids.insert(tool_id.to_string());
            }
        }
    }
    Ok(ids)
}

pub(crate) fn container_support_summary(
    workspace: &Workspace,
) -> Result<BTreeMap<String, (String, String)>> {
    let catalog = load_toml(&workspace.path("containers/contracts/catalog.toml"))?;
    let mut support = BTreeMap::new();
    for row in catalog
        .get("tools")
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(toml::Value::as_table)
    {
        let tool_id = table_string(row, "id");
        let disposition = table_string(row, "support_disposition");
        if !tool_id.is_empty() {
            support.insert(tool_id, (disposition, String::new()));
        }
    }

    let report = read_json(&workspace.path("containers/reports/qualification.json"))?;
    for row in report.get("tools").and_then(serde_json::Value::as_array).into_iter().flatten() {
        let tool_id = row.get("id").and_then(serde_json::Value::as_str).unwrap_or_default();
        let qualification =
            row.get("qualification").and_then(serde_json::Value::as_str).unwrap_or_default();
        let Some((_, recorded_qualification)) = support.get_mut(tool_id) else {
            return Err(anyhow!("qualification report contains unknown tool: {tool_id}"));
        };
        if qualification.is_empty() {
            return Err(anyhow!("qualification report omits qualification for: {tool_id}"));
        }
        *recorded_qualification = qualification.to_string();
    }
    let missing = support
        .iter()
        .filter_map(|(tool_id, (_, qualification))| {
            qualification.is_empty().then_some(tool_id.as_str())
        })
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(anyhow!("qualification report omits catalog tools: {}", missing.join(", ")));
    }
    Ok(support)
}

pub(super) fn apptainer_def_paths(workspace: &Workspace) -> Vec<PathBuf> {
    let mut paths = WalkDir::new(workspace.path("containers/apptainer"))
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(walkdir::DirEntry::into_path)
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("def"))
        .collect::<Vec<_>>();
    paths.sort();
    paths
}

pub(super) fn tool_status_manifest(workspace: &Workspace) -> Result<BTreeMap<String, String>> {
    let mut statuses = BTreeMap::new();
    for raw in read_utf8(&workspace.path("containers/TOOL_IDS.txt"))?.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((tool_id, status)) = line.split_once('\t') {
            statuses.insert(tool_id.to_string(), status.to_string());
        }
    }
    Ok(statuses)
}

pub(super) fn images_metadata(
    workspace: &Workspace,
) -> Result<toml::map::Map<String, toml::Value>> {
    load_toml(&workspace.path("configs/ci/tools/images.toml"))?
        .as_table()
        .cloned()
        .ok_or_else(|| anyhow!("images.toml must be a TOML table"))
}

pub(super) fn toolkit_bundles(
    workspace: &Workspace,
) -> Result<BTreeMap<String, toml::map::Map<String, toml::Value>>> {
    let value = load_toml(&workspace.path("configs/ci/tools/toolkit_bundles.toml"))?;
    let mut rows = BTreeMap::new();
    if let Some(table) = value.get("bundles").and_then(toml::Value::as_table) {
        for (bundle, row) in table {
            if let Some(row) = row.as_table() {
                rows.insert(bundle.clone(), row.clone());
            }
        }
    }
    Ok(rows)
}

pub(super) fn docker_tool_ids(workspace: &Workspace) -> Result<BTreeSet<String>> {
    let mut ids = BTreeSet::new();
    for entry in fs::read_dir(workspace.path("containers/docker"))
        .with_context(|| format!("read {}", workspace.path("containers/docker").display()))?
        .filter_map(std::result::Result::ok)
    {
        if let Some(tool) =
            entry.file_name().to_str().and_then(|name| name.strip_prefix("Dockerfile.arm64."))
        {
            ids.insert(tool.to_string());
        }
    }
    Ok(ids)
}

pub(super) fn dockerfile_paths(workspace: &Workspace) -> Result<Vec<PathBuf>> {
    let mut paths = fs::read_dir(workspace.path("containers/docker"))
        .with_context(|| format!("read {}", workspace.path("containers/docker").display()))?
        .filter_map(std::result::Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("Dockerfile.arm64."))
        })
        .collect::<Vec<_>>();
    paths.sort();
    Ok(paths)
}

pub(super) fn apptainer_tool_ids(workspace: &Workspace) -> BTreeSet<String> {
    apptainer_def_paths(workspace)
        .into_iter()
        .filter_map(|path| path.file_stem().and_then(|name| name.to_str()).map(ToOwned::to_owned))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table(raw: &str) -> Result<toml::map::Map<String, toml::Value>> {
        toml::from_str::<toml::Value>(raw)?
            .as_table()
            .cloned()
            .ok_or_else(|| anyhow!("test fixture must be a TOML table"))
    }

    #[test]
    fn registry_drift_reports_catalog_version_divergence() -> Result<()> {
        let catalog = table(
            r#"
version = "1.0"
support_disposition = "retained"
source_sha256 = "abcd"
docker_arm64_supported = false
apptainer_supported = false
"#,
        )?;
        let registry = table(
            r#"
id = "tool"
version = "2.0"
container = false
runtimes = []
"#,
        )?;
        let mut violations = Vec::new();

        compare_registry_row(&mut violations, "registry.toml", &registry, &catalog);

        assert!(violations.iter().any(|line| {
            line.contains("registry.toml: tool version expected \"1.0\", observed \"2.0\"")
        }));
        Ok(())
    }

    #[test]
    fn registry_drift_rejects_unsupported_recipe_claims() -> Result<()> {
        let catalog = table(
            r#"
version = "1.0"
support_disposition = "unsupported"
source_sha256 = "abcd"
docker_arm64_supported = false
apptainer_supported = false
"#,
        )?;
        let registry = table(
            r#"
id = "tool"
status = "unsupported"
version = "1.0"
container = true
runtimes = ["docker"]
dockerfile = "containers/docker/Dockerfile.arm64.tool"
"#,
        )?;
        let mut violations = Vec::new();

        compare_registry_row(&mut violations, "registry.toml", &registry, &catalog);

        assert!(violations
            .iter()
            .any(|line| line.contains("unsupported tool claims a container recipe")));
        Ok(())
    }
}
