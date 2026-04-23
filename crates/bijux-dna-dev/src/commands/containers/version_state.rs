use super::{
    anyhow, load_toml, read_utf8, write_utf8, BTreeMap, Context, Digest, NaiveDate, PathBuf,
    Result, Serialize, Workspace,
};

pub(super) fn tool_versions(
    workspace: &Workspace,
) -> Result<BTreeMap<String, toml::map::Map<String, toml::Value>>> {
    let value = load_toml(&workspace.path("containers/versions/versions.toml"))?;
    let Some(table) = value.as_table() else {
        return Ok(BTreeMap::new());
    };
    let mut rows = BTreeMap::new();
    for (tool, entry) in table {
        if let Some(entry_table) = entry.as_table() {
            rows.insert(tool.clone(), entry_table.clone());
        }
    }
    Ok(rows)
}

pub(super) fn versions_toml_path(workspace: &Workspace) -> PathBuf {
    workspace.path("containers/versions/versions.toml")
}

pub(super) fn container_version_deprecations_path(workspace: &Workspace) -> PathBuf {
    workspace.path("containers/versions/deprecations.toml")
}

pub(super) fn registry_deprecations_path(workspace: &Workspace) -> PathBuf {
    workspace.path("configs/ci/registry/deprecations.toml")
}

pub(super) fn lock_json_path(workspace: &Workspace) -> PathBuf {
    workspace.path("containers/versions/lock.json")
}

pub(super) fn production_registry_paths(workspace: &Workspace) -> Vec<PathBuf> {
    vec![
        workspace.path("configs/ci/registry/tool_registry.toml"),
        workspace.path("configs/ci/registry/tool_registry_vcf.toml"),
        workspace.path("configs/ci/registry/tool_registry_vcf_downstream.toml"),
    ]
}

pub(super) fn all_registry_paths(workspace: &Workspace) -> Vec<PathBuf> {
    vec![
        workspace.path("configs/ci/registry/tool_registry.toml"),
        workspace.path("configs/ci/registry/tool_registry_experimental.toml"),
        workspace.path("configs/ci/registry/tool_registry_vcf.toml"),
        workspace.path("configs/ci/registry/tool_registry_vcf_downstream.toml"),
    ]
}

pub(super) fn read_lock_json(workspace: &Workspace) -> Result<serde_json::Value> {
    serde_json::from_str(&read_utf8(&lock_json_path(workspace))?)
        .with_context(|| "parse lock.json".to_string())
}

pub(super) fn lock_items_by_tool(
    workspace: &Workspace,
) -> Result<BTreeMap<String, serde_json::Value>> {
    let mut rows = BTreeMap::new();
    if let Some(items) = read_lock_json(workspace)?
        .get("items")
        .and_then(serde_json::Value::as_array)
    {
        for row in items {
            let tool = row
                .get("tool")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .trim()
                .to_string();
            if !tool.is_empty() {
                rows.insert(tool, row.clone());
            }
        }
    }
    Ok(rows)
}

pub(super) fn parse_date(value: &str, field_name: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .with_context(|| format!("invalid {field_name}: {value}"))
}

pub(super) fn update_status_in_table_file(
    path: &std::path::Path,
    tool: &str,
    to_status: &str,
) -> Result<bool> {
    let text = read_utf8(path)?;
    let mut updated = false;
    let mut out = Vec::new();
    let chunks = text.split("[[tools]]").collect::<Vec<_>>();
    if let Some(head) = chunks.first() {
        out.push((*head).to_string());
    }
    for chunk in chunks.iter().skip(1) {
        let mut block = format!("[[tools]]{chunk}");
        if block.contains(&format!("id = \"{tool}\""))
            || block.contains(&format!("tool_id = \"{tool}\""))
        {
            let mut lines = block.lines().map(ToOwned::to_owned).collect::<Vec<_>>();
            if let Some(index) = lines
                .iter()
                .position(|line| line.trim_start().starts_with("status = "))
            {
                lines[index] = format!("status = \"{to_status}\"");
                updated = true;
            }
            block = format!("{}\n", lines.join("\n"));
        }
        out.push(block);
    }
    write_utf8(path, &out.concat())?;
    Ok(updated)
}

pub(super) fn set_registry_status(paths: &[PathBuf], tool: &str, to_status: &str) -> Result<()> {
    let mut updated_any = false;
    for path in paths {
        updated_any |= update_status_in_table_file(path, tool, to_status)?;
    }
    if !updated_any {
        return Err(anyhow!("tool not found: {tool}"));
    }
    Ok(())
}

pub(super) fn set_versions_status(
    workspace: &Workspace,
    tool: &str,
    to_status: &str,
) -> Result<()> {
    let path = versions_toml_path(workspace);
    let text = read_utf8(&path)?;
    let mut updated = false;
    let mut out = Vec::new();
    let chunks = text.split('[').collect::<Vec<_>>();
    if let Some(head) = chunks.first() {
        out.push((*head).to_string());
    }
    for chunk in chunks.iter().skip(1) {
        let block = format!("[{chunk}");
        let Some(table_end) = block.find(']') else {
            out.push(block);
            continue;
        };
        let table_name = block[1..table_end].trim();
        if table_name != tool {
            out.push(block);
            continue;
        }
        let mut lines = block.lines().map(ToOwned::to_owned).collect::<Vec<_>>();
        if let Some(index) = lines
            .iter()
            .position(|line| line.trim_start().starts_with("status = "))
        {
            lines[index] = format!("status = \"{to_status}\"");
        } else {
            lines.insert(1, format!("status = \"{to_status}\""));
        }
        updated = true;
        out.push(format!("{}\n", lines.join("\n")));
    }
    if !updated {
        return Err(anyhow!("missing versions entry for {tool}"));
    }
    write_utf8(&path, &out.concat())
}

pub(super) fn append_toml_table(
    path: &std::path::Path,
    content: &str,
    new_file_header: &str,
) -> Result<()> {
    let body = if path.exists() {
        format!("{}\n\n{}", read_utf8(path)?.trim_end(), content.trim_end())
    } else {
        format!("{}{}", new_file_header, content.trim_end())
    };
    write_utf8(path, &format!("{body}\n"))
}

#[derive(Serialize)]
pub(super) struct VersionMapItem {
    pub(super) tool: String,
    pub(super) version: String,
    pub(super) status: String,
    pub(super) source: String,
    pub(super) source_sha256: String,
    pub(super) pinned_commit: String,
    pub(super) date_pinned: String,
}
