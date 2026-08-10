use super::{
    anyhow, fs, load_toml, out_path_arg, read_json, read_utf8, run_program_with_env, write_utf8,
    BTreeMap, BTreeSet, ContainerCommandOutcome, Context, PathBuf, Result, WalkDir, Workspace,
};

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
    let report =
        bijux_dna_containers::api::derive_container_registry_projection_report(&workspace.root)?;
    write_utf8(&output, &format!("{}\n", serde_json::to_string_pretty(&report)?))?;
    if report.passed {
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
) -> Result<BTreeMap<String, ContainerSupportSummary>> {
    let catalog = bijux_dna_containers::evidence::load_container_catalog(&workspace.root)?;
    let mut support = BTreeMap::new();
    for row in catalog.tools {
        support.insert(
            row.id,
            ContainerSupportSummary {
                product_support: row.product_support,
                artifact_delivery: row.artifact_delivery,
                redistribution_permitted: row.redistribution_permitted,
                qualification: String::new(),
            },
        );
    }

    let report = read_json(&workspace.path("containers/reports/qualification.json"))?;
    for row in report.get("tools").and_then(serde_json::Value::as_array).into_iter().flatten() {
        let tool_id = row.get("id").and_then(serde_json::Value::as_str).unwrap_or_default();
        let qualification =
            row.get("qualification").and_then(serde_json::Value::as_str).unwrap_or_default();
        let Some(summary) = support.get_mut(tool_id) else {
            return Err(anyhow!("qualification report contains unknown tool: {tool_id}"));
        };
        if qualification.is_empty() {
            return Err(anyhow!("qualification report omits qualification for: {tool_id}"));
        }
        summary.qualification = qualification.to_string();
    }
    let missing = support
        .iter()
        .filter_map(|(tool_id, summary)| {
            summary.qualification.is_empty().then_some(tool_id.as_str())
        })
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(anyhow!("qualification report omits catalog tools: {}", missing.join(", ")));
    }
    Ok(support)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ContainerSupportSummary {
    pub(crate) product_support: bijux_dna_containers::evidence::ContainerProductSupport,
    pub(crate) artifact_delivery: bijux_dna_containers::evidence::ContainerArtifactDelivery,
    pub(crate) redistribution_permitted: bool,
    pub(crate) qualification: String,
}

impl ContainerSupportSummary {
    pub(crate) fn is_product_supported(&self) -> bool {
        self.product_support == bijux_dna_containers::evidence::ContainerProductSupport::Supported
    }

    pub(crate) fn is_redistributable_image(&self) -> bool {
        self.artifact_delivery
            == bijux_dna_containers::evidence::ContainerArtifactDelivery::RedistributableImage
    }
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
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() == 4 {
            statuses.insert(fields[0].to_string(), fields[1].to_string());
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
